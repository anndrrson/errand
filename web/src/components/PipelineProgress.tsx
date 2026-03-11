"use client";

import { CheckCircle2, Circle, Loader2 } from "lucide-react";
import type { PipelineStep, TaskStatus } from "@/lib/types";

interface PipelineProgressProps {
  steps: PipelineStep[];
  stepsCompleted: number;
  status: TaskStatus;
  onApprove: () => void;
  approving: boolean;
}

export function PipelineProgress({
  steps,
  stepsCompleted,
  status,
  onApprove,
  approving,
}: PipelineProgressProps) {
  return (
    <div className="rounded-xl border border-border bg-surface-raised p-6">
      <h2 className="text-sm font-semibold text-white mb-4">
        Pipeline Progress
      </h2>

      <div className="space-y-0">
        {steps.map((step, i) => {
          const isCompleted = i < stepsCompleted;
          const isCurrent = i === stepsCompleted;
          const isWaiting =
            isCurrent &&
            status === "waiting_approval" &&
            step.requires_approval;
          const isRunning = isCurrent && status === "running";

          return (
            <div key={i} className="flex gap-3">
              {/* Timeline line + dot */}
              <div className="flex flex-col items-center">
                <div className="mt-0.5">
                  {isCompleted ? (
                    <CheckCircle2 size={18} className="text-success" />
                  ) : isRunning ? (
                    <Loader2
                      size={18}
                      className="text-brand animate-spin"
                    />
                  ) : isWaiting ? (
                    <Circle
                      size={18}
                      className="text-warning fill-warning/20"
                    />
                  ) : (
                    <Circle size={18} className="text-text-muted" />
                  )}
                </div>
                {i < steps.length - 1 && (
                  <div
                    className={`w-px flex-1 min-h-[24px] my-1 ${
                      isCompleted ? "bg-success/40" : "bg-border"
                    }`}
                  />
                )}
              </div>

              {/* Content */}
              <div className="pb-4 min-w-0 flex-1">
                <p
                  className={`text-sm font-medium ${
                    isCompleted
                      ? "text-text-secondary"
                      : isCurrent
                        ? "text-white"
                        : "text-text-muted"
                  }`}
                >
                  <span className="text-xs text-text-muted font-mono mr-2">
                    {i + 1}.
                  </span>
                  {step.description}
                </p>

                {step.requires_approval && (
                  <span className="text-xs text-warning mt-1 inline-block">
                    Requires approval
                  </span>
                )}

                {isWaiting && (
                  <button
                    onClick={onApprove}
                    disabled={approving}
                    className="mt-2 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-success text-white text-xs font-medium hover:bg-success/90 transition-colors disabled:opacity-40"
                  >
                    {approving ? (
                      <Loader2 size={12} className="animate-spin" />
                    ) : (
                      <CheckCircle2 size={12} />
                    )}
                    Approve & Continue
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
