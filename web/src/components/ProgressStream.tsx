"use client";

import { useEffect, useState, useRef } from "react";
import { Loader2, AlertCircle, RefreshCw } from "lucide-react";
import { connectTaskStream } from "@/lib/api";
import type { ProgressEvent } from "@/lib/types";

interface ProgressStreamProps {
  taskId: string;
  active: boolean;
}

export function ProgressStream({ taskId, active }: ProgressStreamProps) {
  const [events, setEvents] = useState<ProgressEvent[]>([]);
  const [error, setError] = useState(false);
  const [reconnecting, setReconnecting] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!active) return;

    const close = connectTaskStream(
      taskId,
      (event) => {
        setEvents((prev) => [...prev, event]);
        setError(false);
        setReconnecting(false);
      },
      () => {
        setError(true);
        setReconnecting(false);
      },
      () => {
        setReconnecting(true);
        setError(false);
      }
    );

    return close;
  }, [taskId, active]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [events]);

  if (!active && events.length === 0) return null;

  return (
    <div className="rounded-xl border border-border bg-surface-raised overflow-hidden">
      <div className="px-4 py-3 border-b border-border flex items-center justify-between">
        <h3 className="text-sm font-semibold text-text-primary">
          Agent Progress
        </h3>
        {active && !error && !reconnecting && (
          <span className="flex items-center gap-1.5 text-xs text-brand-light">
            <Loader2 size={12} className="animate-spin" />
            Live
          </span>
        )}
        {reconnecting && (
          <span className="flex items-center gap-1.5 text-xs text-warning">
            <RefreshCw size={12} className="animate-spin" />
            Reconnecting...
          </span>
        )}
        {error && (
          <span className="flex items-center gap-1.5 text-xs text-error">
            <AlertCircle size={12} />
            Disconnected
          </span>
        )}
      </div>

      <div className="max-h-80 overflow-y-auto p-4 space-y-3">
        {events.length === 0 && (
          <p className="text-xs text-text-muted text-center py-4">
            Waiting for agent to begin work...
          </p>
        )}

        {events.map((event, i) => {
          const isLast = i === events.length - 1;
          return (
            <div key={i} className="flex gap-3">
              <div className="flex flex-col items-center">
                <div
                  className={`w-2 h-2 rounded-full mt-1.5 ${
                    isLast && active
                      ? "bg-brand animate-pulse"
                      : "bg-text-muted"
                  }`}
                />
                {i < events.length - 1 && (
                  <div className="w-px flex-1 bg-border mt-1" />
                )}
              </div>
              <div className="pb-2 min-w-0">
                <p className="text-xs font-medium text-text-secondary">
                  {event.step}
                </p>
                <p className="text-xs text-text-muted mt-0.5">
                  {event.message}
                </p>
                {event.progress_pct !== null && (
                  <div className="mt-1.5 h-1 w-32 rounded-full bg-surface-overlay overflow-hidden">
                    <div
                      className="h-full rounded-full bg-brand transition-all duration-500"
                      style={{ width: `${event.progress_pct}%` }}
                    />
                  </div>
                )}
              </div>
            </div>
          );
        })}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}
