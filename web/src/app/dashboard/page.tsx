"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import {
  Coins,
  Plus,
  Clock,
  CheckCircle2,
  AlertCircle,
  Loader2,
  ArrowUpRight,
} from "lucide-react";
import { useAuth } from "@/lib/auth";
import { getTasks, getCreditBalance } from "@/lib/api";
import type { Task, CreditBalance } from "@/lib/types";
import { StatusBadge } from "@/components/StatusBadge";
import { TaskTypeIcon } from "@/components/TaskTypeIcon";
import { Button } from "@/components/ui/Button";
import { Card } from "@/components/ui/Card";

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}

export default function DashboardPage() {
  const { isAuthenticated, isLoading: authLoading } = useAuth();
  const [tasks, setTasks] = useState<Task[]>([]);
  const [credits, setCredits] = useState<CreditBalance>({
    balance: 10,
    lifetime_earned: 10,
    lifetime_spent: 0,
  });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!isAuthenticated) return;

    async function load() {
      try {
        const [tasksRes, creditsRes] = await Promise.all([
          getTasks(),
          getCreditBalance(),
        ]);
        setTasks(tasksRes.tasks);
        setCredits(creditsRes);
      } catch {
        // Use defaults
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [isAuthenticated]);

  if (authLoading) {
    return (
      <div className="mx-auto max-w-6xl px-6 py-24 text-center">
        <Loader2 size={24} className="mx-auto text-text-muted animate-spin" />
      </div>
    );
  }

  if (!isAuthenticated) {
    return (
      <div className="mx-auto max-w-6xl px-6 py-24 text-center">
        <h1 className="text-2xl font-bold text-white mb-3">Dashboard</h1>
        <p className="text-text-secondary mb-6">
          Login to view your tasks and credit balance.
        </p>
        <Button href="/login">Login</Button>
      </div>
    );
  }

  const activeTasks = tasks.filter(
    (t) =>
      t.status === "running" ||
      t.status === "waiting_approval" ||
      t.status === "pending"
  );

  const recentRuns = tasks
    .flatMap((t) =>
      t.runs.map((r) => ({ ...r, taskTitle: t.title, taskKind: t.kind }))
    )
    .sort(
      (a, b) =>
        new Date(b.started_at).getTime() - new Date(a.started_at).getTime()
    )
    .slice(0, 5);

  return (
    <div className="mx-auto max-w-6xl px-6 py-10">
      <div className="flex items-center justify-between mb-8">
        <h1 className="text-2xl font-bold tracking-tight text-white">
          Dashboard
        </h1>
        <Button href="/tasks/new">
          <Plus size={16} />
          New Task
        </Button>
      </div>

      {/* Credit balance card */}
      <Card className="mb-8">
        <div className="flex items-center gap-3 mb-1">
          <Coins size={18} className="text-warning" />
          <span className="text-sm text-text-muted">Credit Balance</span>
        </div>
        <p className="text-4xl font-bold text-white mt-2">
          {credits.balance}
        </p>
        <p className="text-sm text-text-muted mt-1">
          {credits.balance} credits remaining
        </p>
      </Card>

      {/* Onboarding — shown when user has zero tasks */}
      {tasks.length === 0 && !loading && (
        <Card className="mb-8">
          <h2 className="text-lg font-semibold text-white mb-4">
            How it works
          </h2>
          <div className="grid sm:grid-cols-3 gap-6 mb-6">
            <div>
              <div className="w-8 h-8 rounded-full bg-brand/20 text-brand-light flex items-center justify-center text-sm font-bold mb-2">1</div>
              <p className="text-sm text-text-primary font-medium">Describe a task</p>
              <p className="text-xs text-text-muted mt-1">Tell the agent what to research, monitor, or summarize.</p>
            </div>
            <div>
              <div className="w-8 h-8 rounded-full bg-brand/20 text-brand-light flex items-center justify-center text-sm font-bold mb-2">2</div>
              <p className="text-sm text-text-primary font-medium">Agent does the work</p>
              <p className="text-xs text-text-muted mt-1">It searches the web, analyzes data, and writes a report.</p>
            </div>
            <div>
              <div className="w-8 h-8 rounded-full bg-brand/20 text-brand-light flex items-center justify-center text-sm font-bold mb-2">3</div>
              <p className="text-sm text-text-primary font-medium">Get results</p>
              <p className="text-xs text-text-muted mt-1">Read the output here, or get notified via webhook.</p>
            </div>
          </div>
          <p className="text-xs text-text-muted mb-3">Try one of these:</p>
          <div className="flex flex-wrap gap-2">
            {[
              "Research the latest AI agent frameworks",
              "Summarize this week's crypto market trends",
              "Monitor when SOL price crosses $200",
            ].map((example) => (
              <Link
                key={example}
                href={`/tasks/new?prefill=${encodeURIComponent(example)}`}
                className="text-xs px-3 py-1.5 rounded-full border border-border text-text-secondary hover:border-brand hover:text-brand-light transition-colors"
              >
                {example}
              </Link>
            ))}
          </div>
        </Card>
      )}

      {/* Active tasks */}
      <div className="mb-8">
        <h2 className="text-lg font-semibold text-white mb-4">
          Active Tasks
        </h2>
        {activeTasks.length === 0 ? (
          <Card className="text-center py-10">
            <Clock size={28} className="mx-auto text-text-muted mb-3" />
            <p className="text-text-secondary mb-4">
              No active tasks right now.
            </p>
            <Button href="/tasks/new">
              Create your first agent task
            </Button>
          </Card>
        ) : (
          <div className="space-y-2">
            {activeTasks.map((task) => (
              <Link
                key={task.id}
                href={`/tasks/${task.id}`}
              >
              <Card
                interactive
                padding="sm"
                className="group flex items-center gap-4"
              >
                <TaskTypeIcon kind={task.kind} size={18} />
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-3 mb-1">
                    <h3 className="text-sm font-medium text-text-primary group-hover:text-white truncate">
                      {task.title}
                    </h3>
                    <StatusBadge status={task.status} />
                  </div>
                  {task.next_run_at && (
                    <p className="text-xs text-text-muted">
                      Next run: {formatDate(task.next_run_at)}
                    </p>
                  )}
                </div>
                <ArrowUpRight
                  size={14}
                  className="text-text-muted group-hover:text-text-secondary shrink-0"
                />
              </Card>
              </Link>
            ))}
          </div>
        )}
      </div>

      {/* Recent results */}
      <div>
        <h2 className="text-lg font-semibold text-white mb-4">
          Recent Results
        </h2>
        {recentRuns.length === 0 ? (
          <Card className="text-center py-10">
            <CheckCircle2
              size={28}
              className="mx-auto text-text-muted mb-3"
            />
            <p className="text-text-secondary">
              Results from your tasks will appear here.
            </p>
          </Card>
        ) : (
          <div className="space-y-2">
            {recentRuns.map((run) => (
              <Link
                key={run.id}
                href={`/tasks/${run.task_id}`}
              >
                <Card interactive padding="sm" className="group flex items-center gap-4">
                  <div className="flex-1 min-w-0">
                    <h3 className="text-sm font-medium text-text-primary group-hover:text-white truncate">
                      {run.taskTitle}
                    </h3>
                    <div className="flex items-center gap-3 text-xs text-text-muted mt-1">
                      <span
                        className={`flex items-center gap-1 ${
                          run.status === "completed"
                            ? "text-success"
                            : run.status === "failed"
                              ? "text-error"
                              : "text-brand-light"
                        }`}
                      >
                        {run.status === "completed" ? (
                          <CheckCircle2 size={12} />
                        ) : run.status === "failed" ? (
                          <AlertCircle size={12} />
                        ) : (
                          <Loader2 size={12} className="animate-spin" />
                        )}
                        {run.status}
                      </span>
                      <span>{formatDate(run.started_at)}</span>
                      <span>{run.cost_credits} credits</span>
                    </div>
                    {run.result && (
                      <p className="text-xs text-text-secondary mt-1.5 line-clamp-1">
                        {run.result}
                      </p>
                    )}
                  </div>
                  <ArrowUpRight
                    size={14}
                    className="text-text-muted group-hover:text-text-secondary shrink-0"
                  />
                </Card>
              </Link>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
