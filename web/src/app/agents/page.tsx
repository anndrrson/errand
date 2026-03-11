"use client";

import { useState, useEffect } from "react";
import {
  Bot,
  Search,
  PenTool,
  BarChart3,
  Coins,
  Eye,
  CheckCircle2,
  Wrench,
} from "lucide-react";
import { getAgents } from "@/lib/api";
import type { Agent, TaskCategory } from "@/lib/types";
import { Card } from "@/components/ui/Card";

const demoAgents: Agent[] = [
  {
    id: "agent-research",
    name: "DeepSearch",
    description:
      "Expert research agent for competitive analysis, market research, and comprehensive reports. Pulls from multiple sources and cross-references findings.",
    categories: ["research", "crypto"],
    model: "claude-3.5-sonnet",
    tools: ["web_search", "web_scrape", "pdf_reader", "data_analysis"],
    avg_rating: 4.8,
    jobs_completed: 187,
  },
  {
    id: "agent-content",
    name: "WriteFlow",
    description:
      "Content creation agent for blog posts, documentation, social media copy, and newsletters. Adapts tone and style to your brand guidelines.",
    categories: ["content"],
    model: "claude-3.5-sonnet",
    tools: ["web_search", "text_generation", "seo_analysis"],
    avg_rating: 4.6,
    jobs_completed: 312,
  },
  {
    id: "agent-data",
    name: "DataPipe",
    description:
      "Data extraction and transformation agent. Web scraping, CSV/JSON output, data cleaning, and statistical analysis.",
    categories: ["data"],
    model: "claude-3.5-sonnet",
    tools: ["web_scrape", "data_analysis", "csv_export", "api_caller"],
    avg_rating: 4.7,
    jobs_completed: 156,
  },
  {
    id: "agent-chain",
    name: "ChainScope",
    description:
      "On-chain analysis agent for Solana and EVM chains. Token metrics, wallet tracking, protocol deep-dives, and DeFi analytics.",
    categories: ["crypto", "data", "monitor"],
    model: "claude-3.5-sonnet",
    tools: [
      "blockchain_query",
      "wallet_tracker",
      "token_analytics",
      "web_search",
    ],
    avg_rating: 4.9,
    jobs_completed: 98,
  },
];

const categoryIcons: Record<TaskCategory, typeof Search> = {
  research: Search,
  content: PenTool,
  data: BarChart3,
  crypto: Coins,
  monitor: Eye,
};

const categoryColors: Record<TaskCategory, string> = {
  research: "text-brand-light",
  content: "text-success",
  data: "text-warning",
  crypto: "text-error",
  monitor: "text-purple",
};

export default function AgentsPage() {
  const [agents, setAgents] = useState<Agent[]>(demoAgents);

  useEffect(() => {
    async function load() {
      try {
        const res = await getAgents();
        if (res.agents.length > 0) setAgents(res.agents);
      } catch {
        // Use demo data
      }
    }
    load();
  }, []);

  return (
    <div className="mx-auto max-w-6xl px-6 py-10">
      <div className="mb-8">
        <h1 className="text-2xl font-bold tracking-tight text-white">
          Agents
        </h1>
        <p className="text-sm text-text-secondary mt-1">
          Platform agents are automatically assigned to your tasks based on
          category and requirements.
        </p>
      </div>

      <div className="grid sm:grid-cols-2 gap-4">
        {agents.map((agent) => (
          <Card key={agent.id} interactive>
            <div className="flex items-start gap-4 mb-4">
              <div className="w-10 h-10 rounded-lg bg-brand/15 flex items-center justify-center shrink-0">
                <Bot size={18} className="text-brand-light" />
              </div>
              <div className="min-w-0">
                <h3 className="text-base font-semibold text-white">
                  {agent.name}
                </h3>
                <p className="text-xs text-text-muted mt-0.5 font-mono">
                  {agent.model}
                </p>
              </div>
            </div>

            <p className="text-sm text-text-secondary leading-relaxed mb-4">
              {agent.description}
            </p>

            {/* Categories */}
            <div className="flex flex-wrap gap-1.5 mb-4">
              {agent.categories.map((cat) => {
                const Icon = categoryIcons[cat];
                return (
                  <span
                    key={cat}
                    className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium bg-surface-overlay border border-border ${categoryColors[cat]}`}
                  >
                    <Icon size={12} />
                    {cat}
                  </span>
                );
              })}
            </div>

            {/* Tools */}
            <div className="mb-4">
              <div className="flex items-center gap-1.5 text-xs text-text-muted mb-2">
                <Wrench size={11} />
                <span>Tools</span>
              </div>
              <div className="flex flex-wrap gap-1.5">
                {agent.tools.map((tool) => (
                  <span
                    key={tool}
                    className="px-2 py-0.5 rounded text-xs text-text-secondary bg-surface-overlay border border-border font-mono"
                  >
                    {tool}
                  </span>
                ))}
              </div>
            </div>

            {/* Stats */}
            <div className="flex items-center gap-4 text-xs text-text-muted pt-3 border-t border-border">
              <span className="flex items-center gap-1.5">
                <CheckCircle2 size={12} className="text-success" />
                <span className="text-text-secondary">
                  {agent.jobs_completed} completed
                </span>
              </span>
              <span className="flex items-center gap-1.5">
                <span className="text-warning">
                  {agent.avg_rating.toFixed(1)}
                </span>
                <span className="text-text-secondary">avg rating</span>
              </span>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}
