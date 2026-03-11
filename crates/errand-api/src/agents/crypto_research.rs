use std::time::Duration;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use serde::Deserialize;

use errand_types::{AgentResult, Task, TaskKind};

pub struct CryptoResearchAgent {
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

impl CryptoResearchAgent {
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

    pub async fn execute(&self, task: &Task) -> Result<AgentResult> {
        // Step 1: Search with crypto-focused queries
        let search_results = self.search(&task.title, &task.description).await?;

        // Step 2: Optionally fetch on-chain data if description mentions addresses
        let onchain_context = self.fetch_onchain_context(&task.description).await;

        // Step 3: Synthesize with Claude
        let context = search_results
            .iter()
            .enumerate()
            .map(|(i, r)| format!("[{}] {} ({})\n{}\n", i + 1, r.title, r.url, r.content))
            .collect::<Vec<_>>()
            .join("\n---\n");

        let sources: Vec<String> = search_results.iter().map(|r| r.url.clone()).collect();

        let kind_context = match &task.kind {
            TaskKind::Recurring { .. } => {
                "\n\nThis is a recurring report. Focus on what changed since last time: new price movements, protocol updates, governance proposals, or on-chain activity shifts."
            }
            TaskKind::Monitor { condition, .. } => {
                &format!("\n\nThis is a monitoring check for condition: \"{condition}\". Evaluate whether this condition is currently met based on the data.")
            }
            _ => "",
        };

        let system_prompt = format!(
            r#"You are an expert crypto and blockchain research agent. You specialize in:
- Token/protocol analysis and tokenomics
- DeFi protocol mechanics and risk assessment
- On-chain data interpretation
- Smart contract architecture review
- Market analysis with on-chain context

Guidelines:
- Cite sources using [N] notation
- Include relevant on-chain metrics when available (TVL, volume, holder counts)
- Clearly distinguish between verified facts and speculation
- Flag potential risks and red flags
- Structure: Overview, Key Findings, Risk Assessment, Conclusion{kind_context}"#
        );

        let mut user_prompt = format!(
            "CRYPTO RESEARCH TASK:\nTitle: {}\nDescription: {}\n\nSEARCH RESULTS:\n{}",
            task.title, task.description, context
        );

        if let Some(onchain) = &onchain_context {
            user_prompt.push_str(&format!("\n\nON-CHAIN DATA:\n{onchain}"));
        }

        user_prompt.push_str("\n\nProduce the requested deliverable.");

        let result_text = self.call_claude(&system_prompt, &user_prompt).await?;

        let mut hasher = Sha256::new();
        hasher.update(result_text.as_bytes());
        let result_hash = format!("{:x}", hasher.finalize());

        Ok(AgentResult {
            result_text,
            result_hash,
            sources,
        })
    }

    /// Attempt to fetch Solana account info if the description contains a base58 address.
    async fn fetch_onchain_context(&self, description: &str) -> Option<String> {
        // Simple heuristic: find words that look like Solana addresses (32-44 base58 chars)
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

    async fn search(&self, title: &str, description: &str) -> Result<Vec<TavilyResult>> {
        let query = format!("{title} {description} crypto blockchain");

        let body = serde_json::json!({
            "api_key": self.tavily_key,
            "query": query,
            "search_depth": "advanced",
            "max_results": 10,
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
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 4096,
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
