"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import {
  SlidersHorizontal,
  Search,
  Plus,
  ArrowUpRight,
  Clock,
} from "lucide-react";
import { getTasks } from "@/lib/api";
import type { Task, TaskCategory, TaskStatus } from "@/lib/types";
import { StatusBadge } from "@/components/StatusBadge";
import { TaskTypeIcon } from "@/components/TaskTypeIcon";
import { CategoryBadge } from "@/components/CategoryIcon";
import { Button } from "@/components/ui/Button";
import { Select } from "@/components/ui/Select";
import { Card } from "@/components/ui/Card";

const kindFilters = [
  { value: "", label: "All Types" },
  { value: "one_shot", label: "One-Shot" },
  { value: "recurring", label: "Recurring" },
  { value: "monitor", label: "Monitor" },
  { value: "pipeline", label: "Pipeline" },
];

const statusFilters = [
  { value: "", label: "All Status" },
  { value: "pending", label: "Pending" },
  { value: "running", label: "Running" },
  { value: "waiting_approval", label: "Waiting Approval" },
  { value: "paused", label: "Paused" },
  { value: "completed", label: "Completed" },
  { value: "failed", label: "Failed" },
];

const categoryFilters = [
  { value: "", label: "All Categories" },
  { value: "research", label: "Research" },
  { value: "content", label: "Content" },
  { value: "data", label: "Data" },
  { value: "crypto", label: "Crypto" },
  { value: "monitor", label: "Monitor" },
];

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString("en-US", {
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

function getScheduleInfo(task: Task): string | null {
  if (task.kind.type === "recurring") {
    return `Schedule: ${task.kind.cron}`;
  }
  if (task.kind.type === "monitor") {
    return `Watching: ${task.kind.condition}`;
  }
  return null;
}

export default function TasksPage() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [kind, setKind] = useState("");
  const [status, setStatus] = useState("");
  const [category, setCategory] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      setLoading(true);
      try {
        const res = await getTasks({
          kind: kind || undefined,
          status: status || undefined,
          category: category || undefined,
        });
        if (!cancelled) setTasks(res.tasks);
      } catch {
        if (!cancelled) setTasks([]);
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    load();
    return () => {
      cancelled = true;
    };
  }, [kind, status, category]);

  const filtered = tasks
    .filter((t) => !kind || t.kind.type === kind)
    .filter((t) => !status || t.status === status)
    .filter((t) => !category || t.category === category)
    .sort(
      (a, b) =>
        new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    );

  return (
    <div className="mx-auto max-w-6xl px-6 py-10">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white">
            Tasks
          </h1>
          <p className="text-sm text-text-secondary mt-1">
            {filtered.length} task{filtered.length !== 1 ? "s" : ""}
          </p>
        </div>
        <Button href="/tasks/new">
          <Plus size={16} />
          New Task
        </Button>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap items-center gap-3 mb-8">
        <div className="flex items-center gap-2 text-text-muted">
          <SlidersHorizontal size={14} />
        </div>

        <Select
          value={kind}
          onChange={(e) => setKind(e.target.value)}
        >
          {kindFilters.map((f) => (
            <option key={f.value} value={f.value}>
              {f.label}
            </option>
          ))}
        </Select>

        <Select
          value={status}
          onChange={(e) => setStatus(e.target.value)}
        >
          {statusFilters.map((f) => (
            <option key={f.value} value={f.value}>
              {f.label}
            </option>
          ))}
        </Select>

        <Select
          value={category}
          onChange={(e) => setCategory(e.target.value)}
        >
          {categoryFilters.map((f) => (
            <option key={f.value} value={f.value}>
              {f.label}
            </option>
          ))}
        </Select>
      </div>

      {/* Task List */}
      {loading ? (
        <div className="space-y-2">
          {[...Array(4)].map((_, i) => (
            <div
              key={i}
              className="h-20 rounded-lg border border-border bg-surface-raised animate-pulse"
            />
          ))}
        </div>
      ) : filtered.length === 0 ? (
        <div className="text-center py-20">
          <Search size={32} className="mx-auto text-text-muted mb-3" />
          <p className="text-text-secondary">No tasks match your filters.</p>
        </div>
      ) : (
        <div className="space-y-2">
          {filtered.map((task) => {
            const scheduleInfo = getScheduleInfo(task);
            const lastRun =
              task.runs.length > 0
                ? task.runs[task.runs.length - 1]
                : null;

            return (
              <Link
                key={task.id}
                href={`/tasks/${task.id}`}
              >
                <Card interactive padding="sm" className="group flex items-center gap-4">
                  <TaskTypeIcon kind={task.kind} size={18} />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3 mb-1">
                      <h3 className="text-sm font-medium text-text-primary group-hover:text-white truncate">
                        {task.title}
                      </h3>
                      <StatusBadge status={task.status} />
                      <span className="text-xs text-text-muted hidden sm:inline">
                        {getKindLabel(task.kind)}
                      </span>
                    </div>
                    <div className="flex items-center gap-3 text-xs text-text-muted">
                      <CategoryBadge category={task.category} />
                      {scheduleInfo && (
                        <span className="truncate max-w-[200px]">
                          {scheduleInfo}
                        </span>
                      )}
                      {lastRun && (
                        <span className="flex items-center gap-1">
                          <Clock size={10} />
                          Last: {formatDate(lastRun.started_at)}
                        </span>
                      )}
                      {task.next_run_at && (
                        <span className="text-brand-light">
                          Next: {formatDate(task.next_run_at)}
                        </span>
                      )}
                    </div>
                  </div>
                  <ArrowUpRight
                    size={14}
                    className="text-text-muted group-hover:text-text-secondary shrink-0"
                  />
                </Card>
              </Link>
            );
          })}
        </div>
      )}
    </div>
  );
}
