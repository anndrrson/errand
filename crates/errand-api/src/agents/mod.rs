pub mod crypto_research;
pub mod monitor;
pub mod research;
pub mod summarizer;

use anyhow::Result;

use errand_types::{AgentResult, Task, TaskCategory, TaskKind};

use self::crypto_research::CryptoResearchAgent;
use self::monitor::MonitorAgent;
use self::research::ResearchAgent;
use self::summarizer::SummarizerAgent;

/// Orchestrates platform AI agents, dispatching tasks to the
/// appropriate agent based on category and kind.
pub struct AgentOrchestrator {
    research: ResearchAgent,
    crypto: CryptoResearchAgent,
    summarizer: SummarizerAgent,
    monitor: MonitorAgent,
}

impl AgentOrchestrator {
    pub fn new(anthropic_key: &str, tavily_key: &str, solana_rpc_url: &str) -> Self {
        Self {
            research: ResearchAgent::new(anthropic_key, tavily_key),
            crypto: CryptoResearchAgent::new(anthropic_key, tavily_key, solana_rpc_url),
            summarizer: SummarizerAgent::new(anthropic_key, tavily_key),
            monitor: MonitorAgent::new(anthropic_key, tavily_key, solana_rpc_url),
        }
    }

    /// Execute a task using the appropriate agent.
    pub async fn execute_task(&self, task: &Task) -> Result<AgentResult> {
        // Monitor tasks always use the monitor agent
        if matches!(task.kind, TaskKind::Monitor { .. }) {
            return self.monitor.execute(task).await;
        }

        match task.category {
            TaskCategory::Research => self.research.execute(task).await,
            TaskCategory::Crypto => self.crypto.execute(task).await,
            TaskCategory::Content | TaskCategory::Data => self.summarizer.execute(task).await,
            TaskCategory::Monitor => self.monitor.execute(task).await,
        }
    }

    /// Run a monitor check — returns (condition_met, evidence).
    pub async fn check_condition(&self, task: &Task) -> Result<(bool, String)> {
        self.monitor.check_condition(task).await
    }
}
