use std::time::Duration;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use serde::Deserialize;

use errand_types::{AgentResult, Task, TaskKind};

/// Monitoring agent specialized for watch/alert tasks.
/// Evaluates conditions and returns boolean + evidence.
pub struct MonitorAgent {
    client: reqwest::Client,
    anthropic_key: String,
    tavily_key: String,
    solana_rpc_url: String,
}

#[derive(Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

#[derive(Deserialize)]
struct ConditionResult {
    condition_met: bool,
    evidence: String,
    confidence: f32,
}

impl MonitorAgent {
    pub fn new(anthropic_key: &str, tavily_key: &str, solana_rpc_url: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("failed to build HTTP client"),
            anthropic_key: anthropic_key.to_string(),
            tavily_key: tavily_key.to_string(),
            solana_rpc_url: solana_rpc_url.to_string(),
        }
    }

    /// Full execution — produces a report about the monitored condition.
    pub async fn execute(&self, task: &Task) -> Result<AgentResult> {
        let (met, evidence) = self.check_condition(task).await?;

        let status = if met { "CONDITION MET" } else { "CONDITION NOT MET" };
        let result_text = format!(
            "# Monitor Report: {}\n\n**Status**: {}\n\n**Condition**: {}\n\n**Evidence**:\n{}",
            task.title,
            status,
            match &task.kind {
                TaskKind::Monitor { condition, .. } => condition.as_str(),
                _ => &task.description,
            },
            evidence
        );

        let mut hasher = Sha256::new();
        hasher.update(result_text.as_bytes());
        let result_hash = format!("{:x}", hasher.finalize());

        Ok(AgentResult {
            result_text,
            result_hash,
            sources: vec![],
        })
    }

    /// Check if the monitored condition is met. Returns (met, evidence).
    pub async fn check_condition(&self, task: &Task) -> Result<(bool, String)> {
        let condition = match &task.kind {
            TaskKind::Monitor { condition, .. } => condition.clone(),
            _ => task.description.clone(),
        };

        // Gather data: web search + optional on-chain
        let search_results = self.search(&condition).await?;
        let onchain_context = self.fetch_onchain_context(&task.description).await;

        let mut data_context = String::new();

        for (i, r) in search_results.iter().enumerate() {
            data_context.push_str(&format!(
                "[{}] {} ({})\n{}\n---\n",
                i + 1,
                r.title,
                r.url,
                r.content
            ));
        }

        if let Some(onchain) = &onchain_context {
            data_context.push_str(&format!("\nON-CHAIN DATA:\n{onchain}\n"));
        }

        let system_prompt = r#"You are a monitoring agent. Your job is to evaluate whether a specific condition is currently met based on the provided data.

Respond in this exact JSON format:
{"condition_met": true/false, "evidence": "detailed explanation of what you found", "confidence": 0.0-1.0}

Be precise. Only say condition_met=true if the data clearly supports it. If uncertain, err on the side of false with a note about what's unclear."#;

        let user_prompt = format!(
            "CONDITION TO CHECK: {}\n\nTASK CONTEXT: {}\n\nDATA:\n{}\n\nEvaluate.",
            condition, task.description, data_context
        );

        let raw = self.call_claude(system_prompt, &user_prompt).await?;
        let json_str = extract_json(&raw);

        let result: ConditionResult = serde_json::from_str(json_str).unwrap_or(ConditionResult {
            condition_met: false,
            evidence: format!("Failed to parse monitor response. Raw: {raw}"),
            confidence: 0.0,
        });

        Ok((result.condition_met, result.evidence))
    }

    async fn fetch_onchain_context(&self, description: &str) -> Option<String> {
        let address = description
            .split_whitespace()
            .find(|w| w.len() >= 32 && w.len() <= 44 && w.chars().all(|c| c.is_alphanumeric()))?;

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAccountInfo",
            "params": [address, {"encoding": "jsonParsed"}]
        });

        let resp = self
            .client
            .post(&self.solana_rpc_url)
            .json(&body)
            .send()
            .await
            .ok()?;

        let data: serde_json::Value = resp.json().await.ok()?;
        Some(serde_json::to_string_pretty(&data.get("result")?).unwrap_or_default())
    }

    async fn search(&self, query: &str) -> Result<Vec<TavilyResult>> {
        let body = serde_json::json!({
            "api_key": self.tavily_key,
            "query": query,
            "search_depth": "basic",
            "max_results": 5,
            "include_answer": false,
        });

        let resp = self
            .client
            .post("https://api.tavily.com/search")
            .json(&body)
            .send()
            .await
            .context("Tavily search request failed")?;

        let tavily_resp: TavilyResponse = resp
            .json()
            .await
            .context("Failed to parse Tavily response")?;

        Ok(tavily_resp.results)
    }

    async fn call_claude(&self, system: &str, user: &str) -> Result<String> {
        let body = serde_json::json!({
            "model": "claude-3-5-haiku-latest",
            "max_tokens": 2048,
            "system": system,
            "messages": [
                { "role": "user", "content": user }
            ]
        });

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.anthropic_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Anthropic API request failed")?;

        let api_resp: AnthropicResponse = resp
            .json()
            .await
            .context("Failed to parse Anthropic response")?;

        let text = api_resp
            .content
            .into_iter()
            .filter_map(|b| b.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(text)
    }
}

/// Extract a JSON object from text that might contain markdown code blocks.
fn extract_json(text: &str) -> &str {
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return &text[start..=end];
        }
    }
    text
}
