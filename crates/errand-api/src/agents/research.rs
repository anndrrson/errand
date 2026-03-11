use std::time::Duration;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use serde::Deserialize;

use errand_types::{AgentResult, Task, TaskKind};

pub struct ResearchAgent {
    client: reqwest::Client,
    anthropic_key: String,
    tavily_key: String,
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

impl ResearchAgent {
    pub fn new(anthropic_key: &str, tavily_key: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("failed to build HTTP client"),
            anthropic_key: anthropic_key.to_string(),
            tavily_key: tavily_key.to_string(),
        }
    }

    pub async fn execute(&self, task: &Task) -> Result<AgentResult> {
        // Step 1: Search the web via Tavily
        let search_results = self.search(&task.title, &task.description).await?;

        // Step 2: Synthesize with Claude
        let context = search_results
            .iter()
            .enumerate()
            .map(|(i, r)| format!("[{}] {} ({})\n{}\n", i + 1, r.title, r.url, r.content))
            .collect::<Vec<_>>()
            .join("\n---\n");

        let sources: Vec<String> = search_results.iter().map(|r| r.url.clone()).collect();

        // Adjust system prompt based on task kind
        let kind_context = match &task.kind {
            TaskKind::Recurring { .. } => {
                "\n\nThis is a recurring research task. Focus on what's NEW and CHANGED since this topic was last researched. Highlight recent developments."
            }
            TaskKind::Pipeline { .. } => {
                "\n\nThis is part of a multi-step research pipeline. Be thorough and structured — your output feeds into subsequent steps."
            }
            _ => "",
        };

        let system_prompt = format!(
            r#"You are an expert research agent. Your task is to produce thorough, well-structured research reports based on the provided search results and the task requirements.

Guidelines:
- Cite sources using [N] notation
- Structure the output clearly with headers and sections
- Be factual and precise — do not hallucinate information
- If the search results are insufficient, clearly state what could not be verified{kind_context}"#
        );

        let user_prompt = format!(
            "RESEARCH TASK:\nTitle: {}\nDescription: {}\n\nSEARCH RESULTS:\n{}\n\nProduce a thorough research report.",
            task.title, task.description, context
        );

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

    async fn search(&self, title: &str, description: &str) -> Result<Vec<TavilyResult>> {
        let query = format!("{title} {description}");

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
