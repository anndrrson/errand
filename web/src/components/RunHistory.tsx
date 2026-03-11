"use client";

import { CheckCircle2, AlertCircle, Loader2 } from "lucide-react";
import type { TaskRun } from "@/lib/types";

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleString("en-US", {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}

export function RunHistory({ runs }: { runs: TaskRun[] }) {
  const sorted = [...runs].sort(
    (a, b) =>
      new Date(b.started_at).getTime() - new Date(a.started_at).getTime()
  );

  return (
    <div className="rounded-xl border border-border bg-surface-raised overflow-hidden">
      <div className="px-4 py-3 border-b border-border">
        <h3 className="text-sm font-semibold text-text-primary">
          Run History
        </h3>
      </div>

      <div className="divide-y divide-border">
        {sorted.map((run) => (
          <div key={run.id} className="px-4 py-3 flex items-center gap-4">
            <div className="shrink-0">
              {run.status === "completed" ? (
                <CheckCircle2 size={16} className="text-success" />
              ) : run.status === "failed" ? (
                <AlertCircle size={16} className="text-error" />
              ) : (
                <Loader2
                  size={16}
                  className="text-brand-light animate-spin"
                />
              )}
            </div>

            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-3 text-xs">
                <span
                  className={`font-medium capitalize ${
                    run.status === "completed"
                      ? "text-success"
                      : run.status === "failed"
                        ? "text-error"
                        : "text-brand-light"
                  }`}
                >
                  {run.status}
                </span>
                <span className="text-text-muted">
                  {formatDate(run.started_at)}
                </span>
                {run.completed_at && (
                  <span className="text-text-muted">
                    Finished {formatDate(run.completed_at)}
                  </span>
                )}
              </div>
              {run.result && (
                <p className="text-xs text-text-secondary mt-1 line-clamp-1">
                  {run.result}
                </p>
              )}
            </div>

            <div className="shrink-0 text-xs text-text-muted">
              {run.cost_credits} cr
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
