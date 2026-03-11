"use client";

import type { TaskStatus } from "@/lib/types";

const statusConfig: Record<
  TaskStatus,
  { label: string; className: string }
> = {
  pending: {
    label: "Pending",
    className: "bg-text-muted/15 text-text-secondary border-text-muted/30",
  },
  running: {
    label: "Running",
    className: "bg-brand/15 text-brand-light border-brand/30",
  },
  waiting_approval: {
    label: "Waiting Approval",
    className: "bg-warning/15 text-warning border-warning/30",
  },
  paused: {
    label: "Paused",
    className: "bg-text-muted/15 text-text-muted border-text-muted/30",
  },
  completed: {
    label: "Completed",
    className: "bg-success/15 text-success border-success/30",
  },
  failed: {
    label: "Failed",
    className: "bg-error/15 text-error border-error/30",
  },
  cancelled: {
    label: "Cancelled",
    className: "bg-text-muted/15 text-text-muted border-text-muted/30",
  },
};

export function StatusBadge({ status }: { status: TaskStatus }) {
  const config = statusConfig[status];
  return (
    <span
      className={`inline-flex items-center px-2.5 py-0.5 text-xs font-medium rounded-full border ${config.className}`}
    >
      {config.label}
    </span>
  );
}
