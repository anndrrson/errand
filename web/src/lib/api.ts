import type {
  Task,
  Agent,
  CreateTaskRequest,
  ProgressEvent,
  CreditBalance,
  CreditTransaction,
  AuthResponse,
} from "./types";

const BASE_URL =
  process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001/api";

function getAuthHeader(): Record<string, string> {
  if (typeof window === "undefined") return {};
  const token = localStorage.getItem("errand_jwt");
  if (!token) return {};
  return { Authorization: `Bearer ${token}` };
}

async function request<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...getAuthHeader(),
      ...options.headers,
    },
  });

  if (!res.ok) {
    // Handle 401 — clear auth and redirect to login
    if (res.status === 401) {
      localStorage.removeItem("errand_jwt");
      localStorage.removeItem("errand_user");
      if (typeof window !== "undefined") {
        window.location.href = "/login";
      }
      throw new Error("Session expired. Please log in again.");
    }

    // Try to parse structured error from server
    const body = await res.text();
    let message = `API ${res.status}: ${body}`;
    try {
      const parsed = JSON.parse(body);
      if (parsed.error) {
        message = parsed.error;
      }
    } catch {
      // Fall back to raw text
    }
    throw new Error(message);
  }

  return res.json();
}

// Auth
export async function signup(
  email: string,
  password: string
): Promise<AuthResponse> {
  return request("/auth/signup", {
    method: "POST",
    body: JSON.stringify({ email, password }),
  });
}

export async function login(
  email: string,
  password: string
): Promise<AuthResponse> {
  return request("/auth/login", {
    method: "POST",
    body: JSON.stringify({ email, password }),
  });
}

// Tasks
export async function getTasks(params?: {
  kind?: string;
  status?: string;
  category?: string;
}): Promise<{ tasks: Task[] }> {
  const query = new URLSearchParams();
  if (params?.kind) query.set("kind", params.kind);
  if (params?.status) query.set("status", params.status);
  if (params?.category) query.set("category", params.category);
  const qs = query.toString();
  return request(`/tasks${qs ? `?${qs}` : ""}`);
}

export async function getTask(id: string): Promise<Task> {
  return request(`/tasks/${id}`);
}

export async function createTask(data: CreateTaskRequest): Promise<Task> {
  return request("/tasks", {
    method: "POST",
    body: JSON.stringify(data),
  });
}

export async function pauseTask(id: string): Promise<Task> {
  return request(`/tasks/${id}/pause`, { method: "POST" });
}

export async function resumeTask(id: string): Promise<Task> {
  return request(`/tasks/${id}/resume`, { method: "POST" });
}

export async function cancelTask(id: string): Promise<Task> {
  return request(`/tasks/${id}/cancel`, { method: "POST" });
}

export async function approveTask(id: string): Promise<Task> {
  return request(`/tasks/${id}/approve`, { method: "POST" });
}

// Agents
export async function getAgents(): Promise<{ agents: Agent[] }> {
  return request("/agents");
}

// Credits
export async function getCreditBalance(): Promise<CreditBalance> {
  return request("/credits/balance");
}

export async function getCreditHistory(): Promise<{
  transactions: CreditTransaction[];
}> {
  return request("/credits/history");
}

// SSE Progress Stream with reconnection
export function connectTaskStream(
  taskId: string,
  onEvent: (event: ProgressEvent) => void,
  onError?: (error: Event) => void,
  onReconnecting?: () => void
): () => void {
  const token = localStorage.getItem("errand_jwt") || "";
  const url = `${BASE_URL}/tasks/${taskId}/stream?token=${encodeURIComponent(token)}`;

  let retries = 0;
  const MAX_RETRIES = 3;
  const RETRY_DELAY = 2000;
  let source: EventSource | null = null;
  let retryTimeout: ReturnType<typeof setTimeout> | null = null;
  let cancelled = false;

  function connect() {
    if (cancelled) return;

    source = new EventSource(url);

    source.onmessage = (event) => {
      retries = 0; // Reset on successful message
      try {
        const data: ProgressEvent = JSON.parse(event.data);
        onEvent(data);
      } catch {
        // skip malformed events
      }
    };

    source.onerror = (event) => {
      source?.close();
      source = null;

      if (cancelled) return;

      if (retries < MAX_RETRIES) {
        retries++;
        onReconnecting?.();
        retryTimeout = setTimeout(connect, RETRY_DELAY);
      } else {
        onError?.(event);
      }
    };
  }

  connect();

  return () => {
    cancelled = true;
    if (retryTimeout) clearTimeout(retryTimeout);
    source?.close();
  };
}
