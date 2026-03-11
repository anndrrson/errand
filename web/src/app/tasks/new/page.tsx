"use client";

import { useState, useEffect, Suspense } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import {
  Zap,
  Calendar,
  Eye,
  GitBranch,
  ArrowRight,
  ArrowLeft,
  Plus,
  Trash2,
} from "lucide-react";
import { createTask, getCreditBalance } from "@/lib/api";
import type { CreditBalance } from "@/lib/types";
import type { TaskKind, TaskCategory, PipelineStep } from "@/lib/types";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Textarea } from "@/components/ui/Textarea";
import { Select } from "@/components/ui/Select";
import { Card } from "@/components/ui/Card";

type TaskType = "one_shot" | "recurring" | "monitor" | "pipeline";

const typeOptions: {
  value: TaskType;
  label: string;
  desc: string;
  icon: typeof Zap;
  color: string;
  bg: string;
  creditCost: string;
}[] = [
  {
    value: "one_shot",
    label: "One-Shot",
    desc: "Do something once",
    icon: Zap,
    color: "text-brand-light",
    bg: "border-brand/40 bg-brand/10",
    creditCost: "1 credit",
  },
  {
    value: "recurring",
    label: "Recurring",
    desc: "Run on a schedule",
    icon: Calendar,
    color: "text-success",
    bg: "border-success/40 bg-success/10",
    creditCost: "2 credits/run",
  },
  {
    value: "monitor",
    label: "Monitor",
    desc: "Watch and alert",
    icon: Eye,
    color: "text-purple",
    bg: "border-purple/40 bg-purple/10",
    creditCost: "1 credit/check",
  },
  {
    value: "pipeline",
    label: "Pipeline",
    desc: "Multi-step workflow",
    icon: GitBranch,
    color: "text-warning",
    bg: "border-warning/40 bg-warning/10",
    creditCost: "1 credit/step",
  },
];

const categoryOptions: { value: TaskCategory; label: string }[] = [
  { value: "research", label: "Research" },
  { value: "content", label: "Content" },
  { value: "data", label: "Data" },
  { value: "crypto", label: "Crypto" },
  { value: "monitor", label: "Monitor" },
];

const cronPresets = [
  { label: "Every day at 9am", value: "0 9 * * *" },
  { label: "Every Monday at 9am", value: "0 9 * * 1" },
  { label: "Every weekday at 9am", value: "0 9 * * 1-5" },
  { label: "Every hour", value: "0 * * * *" },
  { label: "Twice daily (9am, 5pm)", value: "0 9,17 * * *" },
  { label: "Every Sunday at 6pm", value: "0 18 * * 0" },
];

const checkIntervalOptions = [
  { label: "Every 5 minutes", value: 300 },
  { label: "Every 15 minutes", value: 900 },
  { label: "Every hour", value: 3600 },
  { label: "Every 6 hours", value: 21600 },
  { label: "Daily", value: 86400 },
];

export default function NewTaskPage() {
  return (
    <Suspense fallback={<div className="min-h-screen bg-surface" />}>
      <NewTaskPageInner />
    </Suspense>
  );
}

function NewTaskPageInner() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const prefill = searchParams.get("prefill") || "";

  const [step, setStep] = useState(1);
  const [taskType, setTaskType] = useState<TaskType | null>(null);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState(prefill);
  const [category, setCategory] = useState<TaskCategory>("research");

  // Recurring fields
  const [cron, setCron] = useState(cronPresets[0].value);

  // Monitor fields
  const [condition, setCondition] = useState("");
  const [checkInterval, setCheckInterval] = useState(3600);

  // Pipeline fields
  const [pipelineSteps, setPipelineSteps] = useState<PipelineStep[]>([
    { description: "", requires_approval: false },
  ]);

  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState("");
  const [credits, setCredits] = useState<CreditBalance | null>(null);

  useEffect(() => {
    getCreditBalance().then(setCredits).catch(() => {});
  }, []);

  // Auto-advance if prefill provided
  useEffect(() => {
    if (prefill) {
      setTaskType("one_shot");
      setTitle(prefill);
      setStep(2);
    }
  }, [prefill]);

  function buildKind(): TaskKind {
    switch (taskType) {
      case "recurring":
        return { type: "recurring", cron };
      case "monitor":
        return {
          type: "monitor",
          condition,
          check_interval_seconds: checkInterval,
        };
      case "pipeline":
        return {
          type: "pipeline",
          steps: pipelineSteps.filter((s) => s.description.trim()),
        };
      default:
        return { type: "one_shot" };
    }
  }

  function getCreditEstimate(): string {
    switch (taskType) {
      case "one_shot":
        return "1 credit";
      case "recurring":
        return "2 credits per run";
      case "monitor":
        return "1 credit per check";
      case "pipeline": {
        const count = pipelineSteps.filter((s) => s.description.trim()).length;
        return `~${count} credit${count !== 1 ? "s" : ""}`;
      }
      default:
        return "--";
    }
  }

  async function handleSubmit() {
    setError("");

    if (!taskType || !title.trim() || !description.trim()) {
      setError("Please fill in all required fields.");
      return;
    }

    if (taskType === "monitor" && !condition.trim()) {
      setError("Please specify a condition to monitor.");
      return;
    }

    if (taskType === "pipeline") {
      const validSteps = pipelineSteps.filter((s) => s.description.trim());
      if (validSteps.length === 0) {
        setError("Add at least one pipeline step.");
        return;
      }
    }

    setSubmitting(true);
    try {
      const task = await createTask({
        title: title.trim(),
        description: description.trim(),
        kind: buildKind(),
        category,
      });
      router.push(`/tasks/${task.id}`);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to create task."
      );
    } finally {
      setSubmitting(false);
    }
  }

  function addPipelineStep() {
    setPipelineSteps([
      ...pipelineSteps,
      { description: "", requires_approval: false },
    ]);
  }

  function removePipelineStep(index: number) {
    setPipelineSteps(pipelineSteps.filter((_, i) => i !== index));
  }

  function updatePipelineStep(
    index: number,
    updates: Partial<PipelineStep>
  ) {
    setPipelineSteps(
      pipelineSteps.map((s, i) => (i === index ? { ...s, ...updates } : s))
    );
  }

  return (
    <div className="mx-auto max-w-3xl px-6 py-10">
      <h1 className="text-2xl font-bold tracking-tight text-white mb-1">
        New Task
      </h1>
      <p className="text-sm text-text-secondary mb-8">
        Set up an agent task. Pick a type, configure, and let it run.
      </p>

      {/* Step indicators */}
      <div className="flex items-center gap-2 mb-8">
        {[1, 2, 3].map((s) => (
          <div key={s} className="flex items-center gap-2">
            <div
              className={`w-7 h-7 rounded-full flex items-center justify-center text-xs font-medium ${
                step >= s
                  ? "bg-brand text-white"
                  : "bg-surface-overlay text-text-muted border border-border"
              }`}
            >
              {s}
            </div>
            {s < 3 && (
              <div
                className={`w-12 h-px ${
                  step > s ? "bg-brand" : "bg-border"
                }`}
              />
            )}
          </div>
        ))}
      </div>

      {/* Step 1: Pick type */}
      {step === 1 && (
        <div>
          <h2 className="text-lg font-semibold text-white mb-4">
            Pick a task type
          </h2>
          <div className="grid grid-cols-2 gap-3">
            {typeOptions.map((opt) => {
              const Icon = opt.icon;
              const isSelected = taskType === opt.value;
              return (
                <button
                  key={opt.value}
                  type="button"
                  onClick={() => setTaskType(opt.value)}
                  className={`rounded-xl border p-5 text-left transition-all ${
                    isSelected
                      ? opt.bg
                      : "border-border bg-surface-raised hover:border-text-muted"
                  }`}
                >
                  <Icon
                    size={20}
                    className={isSelected ? opt.color : "text-text-muted"}
                  />
                  <p
                    className={`text-sm font-semibold mt-3 ${
                      isSelected ? "text-white" : "text-text-primary"
                    }`}
                  >
                    {opt.label}
                  </p>
                  <p className="text-xs text-text-muted mt-0.5">{opt.desc}</p>
                  <p className="text-xs text-text-muted mt-2">
                    {opt.creditCost}
                  </p>
                </button>
              );
            })}
          </div>
          <div className="mt-6">
            <Button
              disabled={!taskType}
              onClick={() => setStep(2)}
            >
              Continue
              <ArrowRight size={16} />
            </Button>
          </div>
        </div>
      )}

      {/* Step 2: Configure */}
      {step === 2 && (
        <div className="space-y-6">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setStep(1)}
          >
            <ArrowLeft size={14} />
            Back
          </Button>

          <h2 className="text-lg font-semibold text-white">
            Configure your task
          </h2>

          {/* Title */}
          <div>
            <label className="block text-sm font-medium text-text-primary mb-2">
              Title
            </label>
            <Input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="e.g. Weekly competitor analysis"
            />
          </div>

          {/* Description */}
          <div>
            <label className="block text-sm font-medium text-text-primary mb-2">
              Description
            </label>
            <Textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={5}
              placeholder="Describe what the agent should do..."
            />
          </div>

          {/* Category */}
          <div>
            <label className="block text-sm font-medium text-text-primary mb-2">
              Category
            </label>
            <Select
              value={category}
              onChange={(e) => setCategory(e.target.value as TaskCategory)}
            >
              {categoryOptions.map((c) => (
                <option key={c.value} value={c.value}>
                  {c.label}
                </option>
              ))}
            </Select>
          </div>

          {/* Recurring: cron */}
          {taskType === "recurring" && (
            <div>
              <label className="block text-sm font-medium text-text-primary mb-2">
                Schedule
              </label>
              <Select
                fullWidth
                value={cron}
                onChange={(e) => setCron(e.target.value)}
              >
                {cronPresets.map((p) => (
                  <option key={p.value} value={p.value}>
                    {p.label}
                  </option>
                ))}
              </Select>
            </div>
          )}

          {/* Monitor: condition + interval */}
          {taskType === "monitor" && (
            <>
              <div>
                <label className="block text-sm font-medium text-text-primary mb-2">
                  Condition to watch
                </label>
                <Input
                  type="text"
                  value={condition}
                  onChange={(e) => setCondition(e.target.value)}
                  placeholder='e.g. "SOL price > $200" or "wallet 0x... moves funds"'
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-text-primary mb-2">
                  Check interval
                </label>
                <Select
                  fullWidth
                  value={checkInterval}
                  onChange={(e) =>
                    setCheckInterval(parseInt(e.target.value, 10))
                  }
                >
                  {checkIntervalOptions.map((opt) => (
                    <option key={opt.value} value={opt.value}>
                      {opt.label}
                    </option>
                  ))}
                </Select>
              </div>
            </>
          )}

          {/* Pipeline: step builder */}
          {taskType === "pipeline" && (
            <div>
              <label className="block text-sm font-medium text-text-primary mb-3">
                Pipeline Steps
              </label>
              <div className="space-y-3">
                {pipelineSteps.map((ps, i) => (
                  <Card
                    key={i}
                    padding="sm"
                    className="flex items-start gap-3"
                  >
                    <span className="text-xs font-mono text-text-muted mt-2.5 shrink-0 w-6">
                      {i + 1}.
                    </span>
                    <div className="flex-1 space-y-2">
                      <Input
                        type="text"
                        value={ps.description}
                        onChange={(e) =>
                          updatePipelineStep(i, {
                            description: e.target.value,
                          })
                        }
                        placeholder="Describe this step..."
                        className="!px-3 !py-2"
                      />
                      <label className="flex items-center gap-2 text-xs text-text-secondary cursor-pointer">
                        <input
                          type="checkbox"
                          checked={ps.requires_approval}
                          onChange={(e) =>
                            updatePipelineStep(i, {
                              requires_approval: e.target.checked,
                            })
                          }
                          className="rounded border-border"
                        />
                        Requires my approval before continuing
                      </label>
                    </div>
                    {pipelineSteps.length > 1 && (
                      <button
                        type="button"
                        onClick={() => removePipelineStep(i)}
                        className="text-text-muted hover:text-error transition-colors mt-2"
                      >
                        <Trash2 size={14} />
                      </button>
                    )}
                  </Card>
                ))}
              </div>
              <Button
                variant="ghost"
                size="sm"
                onClick={addPipelineStep}
                className="mt-3"
              >
                <Plus size={14} />
                Add step
              </Button>
            </div>
          )}

          <div className="flex items-center gap-3 pt-2">
            <Button
              disabled={!title.trim() || !description.trim()}
              onClick={() => setStep(3)}
            >
              Review
              <ArrowRight size={16} />
            </Button>
          </div>
        </div>
      )}

      {/* Step 3: Review + confirm */}
      {step === 3 && (
        <div className="space-y-6">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setStep(2)}
          >
            <ArrowLeft size={14} />
            Back
          </Button>

          <h2 className="text-lg font-semibold text-white">
            Review your task
          </h2>

          <Card className="space-y-4">
            <div>
              <span className="text-xs text-text-muted uppercase tracking-wider">
                Type
              </span>
              <p className="text-sm text-text-primary mt-1 capitalize">
                {taskType?.replace("_", "-")}
              </p>
            </div>
            <div>
              <span className="text-xs text-text-muted uppercase tracking-wider">
                Title
              </span>
              <p className="text-sm text-text-primary mt-1">{title}</p>
            </div>
            <div>
              <span className="text-xs text-text-muted uppercase tracking-wider">
                Description
              </span>
              <p className="text-sm text-text-secondary mt-1 whitespace-pre-wrap">
                {description}
              </p>
            </div>
            <div>
              <span className="text-xs text-text-muted uppercase tracking-wider">
                Category
              </span>
              <p className="text-sm text-text-primary mt-1 capitalize">
                {category}
              </p>
            </div>

            {taskType === "recurring" && (
              <div>
                <span className="text-xs text-text-muted uppercase tracking-wider">
                  Schedule
                </span>
                <p className="text-sm text-text-primary mt-1">
                  {cronPresets.find((p) => p.value === cron)?.label || cron}
                </p>
              </div>
            )}

            {taskType === "monitor" && (
              <>
                <div>
                  <span className="text-xs text-text-muted uppercase tracking-wider">
                    Condition
                  </span>
                  <p className="text-sm text-text-primary mt-1">{condition}</p>
                </div>
                <div>
                  <span className="text-xs text-text-muted uppercase tracking-wider">
                    Check Interval
                  </span>
                  <p className="text-sm text-text-primary mt-1">
                    {checkIntervalOptions.find(
                      (o) => o.value === checkInterval
                    )?.label || `${checkInterval}s`}
                  </p>
                </div>
              </>
            )}

            {taskType === "pipeline" && (
              <div>
                <span className="text-xs text-text-muted uppercase tracking-wider">
                  Steps
                </span>
                <div className="mt-2 space-y-2">
                  {pipelineSteps
                    .filter((s) => s.description.trim())
                    .map((s, i) => (
                      <div
                        key={i}
                        className="flex items-center gap-2 text-sm"
                      >
                        <span className="text-xs text-text-muted font-mono">
                          {i + 1}.
                        </span>
                        <span className="text-text-primary">
                          {s.description}
                        </span>
                        {s.requires_approval && (
                          <span className="text-xs text-warning bg-warning/10 px-1.5 py-0.5 rounded">
                            approval required
                          </span>
                        )}
                      </div>
                    ))}
                </div>
              </div>
            )}

            <div className="pt-2 border-t border-border">
              <span className="text-xs text-text-muted uppercase tracking-wider">
                Estimated Cost
              </span>
              <p className="text-sm text-text-primary mt-1 font-medium">
                {getCreditEstimate()}
              </p>
            </div>

            {credits !== null && (
              <div className="pt-2 border-t border-border">
                <span className="text-xs text-text-muted uppercase tracking-wider">
                  Your Balance
                </span>
                <p className={`text-sm mt-1 font-medium ${credits.balance <= 0 ? "text-error" : "text-text-primary"}`}>
                  {credits.balance} credit{credits.balance !== 1 ? "s" : ""} remaining
                </p>
                {credits.balance <= 0 && (
                  <p className="text-xs text-error mt-1">
                    Insufficient credits to create this task.
                  </p>
                )}
              </div>
            )}
          </Card>

          {error && <p className="text-sm text-error">{error}</p>}

          <Button
            onClick={handleSubmit}
            loading={submitting}
            disabled={credits !== null && credits.balance <= 0}
            size="lg"
          >
            <ArrowRight size={16} />
            Create Task
          </Button>
        </div>
      )}
    </div>
  );
}
