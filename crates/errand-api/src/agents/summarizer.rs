use std::time::Duration;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use serde::Deserialize;

use errand_types::{AgentResult, Task, TaskKind};

pub struct SummarizerAgent {
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

impl SummarizerAgent {
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
        // Try to fetch content from any URLs in the description
        let fetched_content = self.fetch_urls(&task.description).await;

        // Search for additional context if needed
        let search_results = self.search(&task.title, &task.description).await?;

        let mut context = String::new();

        if !fetched_content.is_empty() {
            context.push_str("FETCHED CONTENT:\n");
            for (url, content) in &fetched_content {
                context.push_str(&format!("--- {} ---\n{}\n\n", url, content));
            }
        }

        if !search_results.is_empty() {
            context.push_str("SEARCH RESULTS:\n");
            for (i, r) in search_results.iter().enumerate() {
                context.push_str(&format!(
                    "[{}] {} ({})\n{}\n---\n",
                    i + 1,
                    r.title,
                    r.url,
                    r.content
                ));
            }
        }

        let sources: Vec<String> = search_results.iter().map(|r| r.url.clone()).collect();

        let pipeline_note = if matches!(task.kind, TaskKind::Pipeline { .. }) {
            "\n\nThis is part of a multi-step pipeline. Structure your output to be consumable by subsequent processing steps."
        } else {
            ""
        };

        let system_prompt = format!(
            r#"You are an expert content summarizer and writer. Your task is to produce high-quality summaries, rewrites, or content pieces based on the provided source material.

Guidelines:
- Preserve key information and nuance from the source
- Adapt tone and style to the task requirements
- Structure output clearly with appropriate formatting
- Be concise but comprehensive — don't lose critical details{pipeline_note}"#
        );

        let user_prompt = format!(
            "CONTENT TASK:\nTitle: {}\nDescription: {}\n\n{}\n\nProduce the requested deliverable.",
            task.title, task.description, context,
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

    /// Fetch and convert HTML from URLs found in text.
    async fn fetch_urls(&self, text: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();

        for word in text.split_whitespace() {
            if word.starts_with("http://") || word.starts_with("https://") {
                let url = word.trim_matches(|c: char| !c.is_alphanumeric() && c != ':' && c != '/' && c != '.' && c != '-' && c != '_' && c != '?' && c != '=' && c != '&');
                if let Ok(resp) = self.client.get(url).send().await {
                    if let Ok(html) = resp.text().await {
                        // html2text 0.14 from_read returns Result
                        let plain = match html2text::from_read(html.as_bytes(), 120) {
                            Ok(text) => text,
                            Err(_) => html[..html.len().min(8000)].to_string(),
                        };
                        // Truncate to avoid blowing up context
                        let truncated = if plain.len() > 8000 {
                            format!("{}...[truncated]", &plain[..8000])
                        } else {
                            plain
                        };
                        results.push((url.to_string(), truncated));
                    }
                }
            }
        }

        results
    }

    async fn search(&self, title: &str, description: &str) -> Result<Vec<TavilyResult>> {
        let query = format!("{title} {description}");

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
