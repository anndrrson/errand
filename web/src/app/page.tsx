"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import {
  ArrowRight,
  Calendar,
  Eye,
  GitBranch,
  Bot,
  Zap,
  Clock,
  Send,
} from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Card } from "@/components/ui/Card";

const features = [
  {
    icon: Calendar,
    title: "Recurring Reports",
    desc: "Get competitor analysis every Monday. Market summaries every morning. Automated, on schedule.",
    color: "text-brand-light",
    bg: "bg-brand/10",
  },
  {
    icon: Eye,
    title: "Smart Monitoring",
    desc: "Watch wallets, track prices, monitor competitors. Get alerted the moment something changes.",
    color: "text-purple",
    bg: "bg-purple/10",
  },
  {
    icon: GitBranch,
    title: "Multi-Step Pipelines",
    desc: "Chain tasks together. Research, analyze, summarize. Approve each step or let it run.",
    color: "text-success",
    bg: "bg-success/10",
  },
];

const exampleTasks = [
  {
    text: "Every Monday: Summarize what my 5 competitors shipped this week",
    icon: Calendar,
    color: "text-brand-light",
  },
  {
    text: "Alert me when any wallet sends >1000 SOL from this address",
    icon: Eye,
    color: "text-purple",
  },
  {
    text: "Research, Draft, Review pipeline for my weekly blog post",
    icon: GitBranch,
    color: "text-success",
  },
];

export default function Home() {
  const router = useRouter();
  const [quickQuery, setQuickQuery] = useState("");

  function handleQuickSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!quickQuery.trim()) return;
    router.push(
      `/tasks/new?prefill=${encodeURIComponent(quickQuery.trim())}`
    );
  }

  return (
    <div>
      {/* Hero */}
      <section className="relative overflow-hidden">
        <div className="mx-auto max-w-6xl px-6 pt-24 pb-20">
          <div className="max-w-2xl">
            <h1 className="text-4xl sm:text-5xl font-bold tracking-tight leading-[1.1] text-white">
              Agents work
              <br />
              while you sleep.
            </h1>
            <p className="mt-5 text-lg text-text-secondary leading-relaxed max-w-lg">
              Set up AI agents that research, monitor, and report — on
              autopilot. Get results when you wake up.
            </p>
            <div className="mt-8 flex items-center gap-3">
              <Button href="/signup" size="lg">
                Start free — 10 credits on signup
                <ArrowRight size={16} />
              </Button>
            </div>
          </div>
        </div>

        {/* Grid background */}
        <div
          className="absolute inset-0 -z-10 opacity-[0.03]"
          style={{
            backgroundImage:
              "linear-gradient(var(--color-border) 1px, transparent 1px), linear-gradient(90deg, var(--color-border) 1px, transparent 1px)",
            backgroundSize: "64px 64px",
          }}
        />
        {/* Subtle hero glow */}
        <div
          className="absolute top-0 left-1/4 -z-10 w-[600px] h-[400px] rounded-full blur-3xl"
          style={{
            background:
              "radial-gradient(ellipse, var(--color-brand) 0%, transparent 70%)",
            opacity: 0.06,
          }}
        />
      </section>

      {/* Quick one-shot */}
      <section className="border-y border-border bg-surface-raised">
        <div className="mx-auto max-w-6xl px-6 py-8">
          <p className="text-sm text-text-secondary mb-3">
            Or try a one-shot task
          </p>
          <form
            onSubmit={handleQuickSubmit}
            className="flex items-center gap-3"
          >
            <Input
              value={quickQuery}
              onChange={(e) => setQuickQuery(e.target.value)}
              placeholder="Ask a research question... e.g. &quot;Compare the top 5 project management tools&quot;"
              className="flex-1"
            />
            <Button type="submit" variant="secondary" size="lg">
              <Send size={14} />
              Go
            </Button>
          </form>
        </div>
      </section>

      {/* Features */}
      <section className="mx-auto max-w-6xl px-6 py-24">
        <h2 className="text-2xl font-bold tracking-tight text-white mb-2">
          What agents can do
        </h2>
        <p className="text-text-secondary mb-10">
          Set it up once. Let it run forever.
        </p>
        <div className="grid sm:grid-cols-3 gap-6">
          {features.map((feat) => {
            const Icon = feat.icon;
            return (
              <Card key={feat.title} interactive>
                <div
                  className={`w-10 h-10 rounded-lg ${feat.bg} flex items-center justify-center mb-4`}
                >
                  <Icon size={20} className={feat.color} />
                </div>
                <h3 className="text-base font-semibold text-white mb-2">
                  {feat.title}
                </h3>
                <p className="text-sm text-text-secondary leading-relaxed">
                  {feat.desc}
                </p>
              </Card>
            );
          })}
        </div>
      </section>

      {/* Example tasks */}
      <section className="mx-auto max-w-6xl px-6 pb-24">
        <h2 className="text-2xl font-bold tracking-tight text-white mb-2">
          Example tasks
        </h2>
        <p className="text-text-secondary mb-8">
          Real workflows people run with Errand agents.
        </p>
        <div className="space-y-3">
          {exampleTasks.map((task) => {
            const Icon = task.icon;
            return (
              <Link
                key={task.text}
                href={`/tasks/new?prefill=${encodeURIComponent(task.text)}`}
                className="group flex items-center gap-4 rounded-lg border border-border bg-surface-raised p-4 hover:border-text-muted hover:bg-surface-overlay transition-all"
              >
                <Icon size={18} className={task.color} />
                <span className="text-sm text-text-primary group-hover:text-white">
                  {task.text}
                </span>
                <ArrowRight
                  size={14}
                  className="ml-auto text-text-muted opacity-0 group-hover:opacity-100 transition-opacity"
                />
              </Link>
            );
          })}
        </div>
      </section>

      {/* Stats */}
      <section className="border-y border-border bg-surface-raised">
        <div className="mx-auto max-w-6xl px-6 py-5 flex items-center justify-center gap-8 sm:gap-16 text-sm">
          <div className="flex items-center gap-2 text-text-secondary">
            <Zap size={14} className="text-brand-light" />
            <span className="text-base font-semibold text-text-primary">4</span>
            <span>task types</span>
          </div>
          <div className="hidden sm:block w-px h-4 bg-border" />
          <div className="flex items-center gap-2 text-text-secondary">
            <Bot size={14} className="text-brand-light" />
            <span className="text-base font-semibold text-text-primary">4</span>
            <span>specialized agents</span>
          </div>
          <div className="hidden sm:block w-px h-4 bg-border" />
          <div className="flex items-center gap-2 text-text-secondary">
            <Clock size={14} className="text-brand-light" />
            <span className="text-base font-semibold text-text-primary">24/7</span>
            <span>always running</span>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-border">
        <div className="mx-auto max-w-6xl px-6 py-8 flex items-center justify-between text-xs text-text-muted">
          <span>errand</span>
          <span>Agents work while you sleep.</span>
        </div>
      </footer>
    </div>
  );
}
