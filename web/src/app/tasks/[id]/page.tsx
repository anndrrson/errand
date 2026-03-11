"use client";

import { useState, useEffect } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import {
  ArrowLeft,
  Clock,
  Pause,
  Play,
  XCircle,
  CheckCircle2,
  AlertCircle,
} from "lucide-react";
import ReactMarkdown from "react-markdown";
import {
  getTask,
  pauseTask,
  resumeTask,
  cancelTask,
  approveTask,
} from "@/lib/api";
import type { Task } from "@/lib/types";
import { StatusBadge } from "@/components/StatusBadge";
import { CategoryBadge } from "@/components/CategoryIcon";
import { TaskTypeIcon } from "@/components/TaskTypeIcon";
import { ProgressStream } from "@/components/ProgressStream";
import { PipelineProgress } from "@/components/PipelineProgress";
import { RunHistory } from "@/components/RunHistory";
import { Button } from "@/components/ui/Button";
import { Card } from "@/components/ui/Card";

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleString("en-US", {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}

function getKindLabel(kind: Task["kind"]): string {
  switch (kind.type) {
    case "one_shot":
      return "One-Shot";
    case "recurring":
      return "Recurring";
    case "monitor":
      return "Monitor";
    case "pipeline":
      return "Pipeline";
  }
}

export default function TaskDetailPage() {
  const params = useParams();
  const taskId = params.id as string;

  const [task, setTask] = useState<Task | null>(null);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState(false);

  useEffect(() => {
    async function load() {
      try {
        const data = await getTask(taskId);
        setTask(data);
      } catch {
        // Task not found
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [taskId]);

  async function handleAction(
    action: (id: string) => Promise<Task>
  ) {
    setActionLoading(true);
    try {
      const updated = await action(taskId);
      setTask(updated);
    } catch (err) {
      console.error("Action failed:", err);
    } finally {
      setActionLoading(false);
    }
  }

  if (loading) {
    return (
      <div className="mx-auto max-w-4xl px-6 py-10">
        <div className="h-8 w-48 rounded bg-surface-raised animate-pulse mb-6" />
        <div className="h-64 rounded-xl bg-surface-raised animate-pulse" />
      </div>
    );
  }

  if (!task) {
    return (
      <div className="mx-auto max-w-4xl px-6 py-24 text-center">
        <AlertCircle size={32} className="mx-auto text-text-muted mb-3" />
        <p className="text-text-secondary">Task not found.</p>
        <Link
          href="/tasks"
          className="inline-flex items-center gap-1.5 text-sm text-brand-light hover:text-white transition-colors mt-4"
        >
          <ArrowLeft size={14} />
          Back to Tasks
        </Link>
      </div>
    );
  }

  const isActive = task.status === "running" || task.status === "pending";
  const latestRun =
    task.runs.length > 0 ? task.runs[task.runs.length - 1] : null;

  return (
    <div className="mx-auto max-w-4xl px-6 py-10">
      <Link
        href="/tasks"
        className="inline-flex items-center gap-1.5 text-sm text-text-muted hover:text-text-secondary transition-colors mb-6"
      >
        <ArrowLeft size={14} />
        Back to Tasks
      </Link>

      <div className="grid lg:grid-cols-[1fr_280px] gap-6">
        {/* Main column */}
        <div className="space-y-6">
          {/* Header */}
          <Card>
            <div className="flex items-start justify-between gap-4 mb-4">
              <h1 className="text-xl font-bold text-white leading-snug">
                {task.title}
              </h1>
              <StatusBadge status={task.status} />
            </div>

            <div className="flex items-center gap-4 mb-5 text-sm text-text-secondary">
              <div className="flex items-center gap-1.5">
                <TaskTypeIcon kind={task.kind} size={14} />
                <span>{getKindLabel(task.kind)}</span>
              </div>
              <CategoryBadge category={task.category} />
              <span className="flex items-center gap-1">
                <Clock size={14} />
                {formatDate(task.created_at)}
              </span>
            </div>

            <p className="text-sm text-text-secondary whitespace-pre-wrap leading-relaxed">
              {task.description}
            </p>
          </Card>

          {/* Type-specific info */}
          {task.kind.type === "recurring" && (
            <Card>
              <h2 className="text-sm font-semibold text-white mb-3">
                Schedule
              </h2>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-text-muted">Cron</span>
                  <span className="text-text-secondary font-mono text-xs">
                    {task.kind.cron}
                  </span>
                </div>
                {task.next_run_at && (
                  <div className="flex justify-between">
                    <span className="text-text-muted">Next run</span>
                    <span className="text-text-secondary">
                      {formatDate(task.next_run_at)}
                    </span>
                  </div>
                )}
              </div>
            </Card>
          )}

          {task.kind.type === "monitor" && (
            <Card>
              <h2 className="text-sm font-semibold text-white mb-3">
                Monitor Config
              </h2>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-text-muted">Condition</span>
                  <span className="text-text-secondary">
                    {task.kind.condition}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-text-muted">Check interval</span>
                  <span className="text-text-secondary">
                    {task.kind.check_interval_seconds >= 3600
                      ? `${Math.floor(task.kind.check_interval_seconds / 3600)}h`
                      : `${Math.floor(task.kind.check_interval_seconds / 60)}m`}
                  </span>
                </div>
              </div>
            </Card>
          )}

          {task.kind.type === "pipeline" && (
            <PipelineProgress
              steps={task.kind.steps}
              stepsCompleted={latestRun?.steps_completed ?? 0}
              status={task.status}
              onApprove={() => handleAction(approveTask)}
              approving={actionLoading}
            />
          )}

          {/* Progress stream for active tasks */}
          {isActive && (
            <ProgressStream taskId={taskId} active={isActive} />
          )}

          {/* Latest result */}
          {latestRun?.result && (
            <Card>
              <h2 className="text-sm font-semibold text-white mb-3 flex items-center gap-2">
                <CheckCircle2 size={16} className="text-success" />
                Latest Result
              </h2>
              <div className="rounded-lg bg-surface p-4 border border-border prose prose-invert prose-sm max-w-none">
                <ReactMarkdown>{latestRun.result}</ReactMarkdown>
              </div>
            </Card>
          )}

          {/* Run history */}
          {task.runs.length > 0 && (
            <RunHistory runs={task.runs} />
          )}

          {/* Controls */}
          {(task.status === "running" ||
            task.status === "paused" ||
            task.status === "pending" ||
            task.status === "waiting_approval") && (
            <div className="flex items-center gap-3">
              {task.status === "running" && (
                <Button
                  variant="secondary"
                  onClick={() => handleAction(pauseTask)}
                  loading={actionLoading}
                >
                  <Pause size={14} />
                  Pause
                </Button>
              )}
              {task.status === "paused" && (
                <Button
                  onClick={() => handleAction(resumeTask)}
                  loading={actionLoading}
                >
                  <Play size={14} />
                  Resume
                </Button>
              )}
              {task.status === "waiting_approval" &&
                task.kind.type !== "pipeline" && (
                  <Button
                    variant="success"
                    onClick={() => handleAction(approveTask)}
                    loading={actionLoading}
                  >
                    <CheckCircle2 size={14} />
                    Approve
                  </Button>
                )}
              <Button
                variant="danger"
                onClick={() => handleAction(cancelTask)}
                disabled={actionLoading}
              >
                <XCircle size={14} />
                Cancel
              </Button>
            </div>
          )}
        </div>

        {/* Sidebar */}
        <div className="space-y-4">
          <Card padding="sm">
            <h3 className="text-xs font-medium text-text-muted mb-3 uppercase tracking-wider">
              Timeline
            </h3>
            <div className="space-y-2.5 text-xs">
              <div className="flex justify-between">
                <span className="text-text-muted">Created</span>
                <span className="text-text-secondary">
                  {formatDate(task.created_at)}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-text-muted">Updated</span>
                <span className="text-text-secondary">
                  {formatDate(task.updated_at)}
                </span>
              </div>
              {task.next_run_at && (
                <div className="flex justify-between">
                  <span className="text-text-muted">Next run</span>
                  <span className="text-brand-light">
                    {formatDate(task.next_run_at)}
                  </span>
                </div>
              )}
            </div>
          </Card>

          <Card padding="sm">
            <h3 className="text-xs font-medium text-text-muted mb-3 uppercase tracking-wider">
              Stats
            </h3>
            <div className="space-y-2.5 text-xs">
              <div className="flex justify-between">
                <span className="text-text-muted">Total runs</span>
                <span className="text-text-secondary">
                  {task.runs.length}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-text-muted">Credits used</span>
                <span className="text-text-secondary">
                  {task.runs.reduce((sum, r) => sum + r.cost_credits, 0)}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-text-muted">Successful</span>
                <span className="text-success">
                  {task.runs.filter((r) => r.status === "completed").length}
                </span>
              </div>
            </div>
          </Card>

          {task.webhook_url && (
            <Card padding="sm">
              <h3 className="text-xs font-medium text-text-muted mb-2 uppercase tracking-wider">
                Webhook
              </h3>
              <p className="text-xs text-text-secondary font-mono break-all">
                {task.webhook_url}
              </p>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}
