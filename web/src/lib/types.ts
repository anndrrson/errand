export type TaskKind =
  | { type: "one_shot" }
  | { type: "recurring"; cron: string }
  | { type: "monitor"; condition: string; check_interval_seconds: number }
  | { type: "pipeline"; steps: PipelineStep[] };

export interface PipelineStep {
  description: string;
  requires_approval: boolean;
  depends_on?: number;
}

export type TaskStatus =
  | "pending"
  | "running"
  | "waiting_approval"
  | "paused"
  | "completed"
  | "failed"
  | "cancelled";

export type TaskCategory = "research" | "content" | "data" | "crypto" | "monitor";

export interface Task {
  id: string;
  title: string;
  description: string;
  kind: TaskKind;
  category: TaskCategory;
  status: TaskStatus;
  webhook_url?: string;
  email_notify?: string;
  next_run_at?: string;
  created_at: string;
  updated_at: string;
  runs: TaskRun[];
}

export interface TaskRun {
  id: string;
  task_id: string;
  status: "running" | "completed" | "failed";
  steps_completed: number;
  result?: string;
  cost_credits: number;
  started_at: string;
  completed_at?: string;
}

export interface Agent {
  id: string;
  name: string;
  description: string;
  categories: TaskCategory[];
  model: string;
  tools: string[];
  avg_rating: number;
  jobs_completed: number;
}

export interface CreditBalance {
  balance: number;
  lifetime_earned: number;
  lifetime_spent: number;
}

export interface CreditTransaction {
  id: string;
  amount: number;
  reason: string;
  created_at: string;
}

export interface ProgressEvent {
  step: string;
  message: string;
  timestamp: string;
  progress_pct: number | null;
}

export interface CreateTaskRequest {
  title: string;
  description: string;
  kind: TaskKind;
  category: TaskCategory;
  webhook_url?: string;
  email_notify?: string;
}

export interface AuthUser {
  id: string;
  email: string;
}

export interface AuthResponse {
  token: string;
  user: AuthUser;
}
