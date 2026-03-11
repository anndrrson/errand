-- Errand: Persistent AI Agent Platform
-- "Agents work while you sleep."

-- Users (email + password auth for beta)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tasks (the core unit — one-shot, recurring, monitor, or pipeline)
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    kind JSONB NOT NULL,
    category TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    webhook_url TEXT,
    email_notify TEXT,
    next_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tasks_owner ON tasks(owner_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_next_run ON tasks(next_run_at) WHERE status = 'running';

-- Task runs (each execution of a task)
CREATE TABLE task_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id),
    status TEXT NOT NULL DEFAULT 'running',
    steps_completed INT NOT NULL DEFAULT 0,
    result TEXT,
    result_hash TEXT,
    cost_credits INT NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_task_runs_task ON task_runs(task_id);

-- Credit balances
CREATE TABLE credit_balances (
    owner_id UUID PRIMARY KEY REFERENCES users(id),
    balance BIGINT NOT NULL DEFAULT 0,
    lifetime_earned BIGINT NOT NULL DEFAULT 0,
    lifetime_spent BIGINT NOT NULL DEFAULT 0
);

-- Credit transactions (audit log)
CREATE TABLE credit_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id),
    amount BIGINT NOT NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_credit_tx_owner ON credit_transactions(owner_id);

-- Agent registry
CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    categories JSONB NOT NULL DEFAULT '[]',
    model TEXT NOT NULL,
    tools JSONB NOT NULL DEFAULT '[]',
    avg_rating REAL NOT NULL DEFAULT 0,
    jobs_completed INT NOT NULL DEFAULT 0
);

-- Seed agents
INSERT INTO agents (id, name, description, categories, model, tools, avg_rating, jobs_completed)
VALUES
    (
        'research-pro',
        'Research Pro',
        'Expert research agent — web search, synthesis, structured reports.',
        '["research", "data"]',
        'claude-sonnet-4-20250514',
        '["web_search", "read_url"]',
        4.5,
        0
    ),
    (
        'crypto-research',
        'Crypto Analyst',
        'Crypto and blockchain research — on-chain data, DeFi analysis, market reports.',
        '["crypto"]',
        'claude-sonnet-4-20250514',
        '["web_search", "read_url", "solana_rpc"]',
        4.5,
        0
    ),
    (
        'summarizer',
        'Content Synthesizer',
        'Summarize URLs, PDFs, and raw text. Multi-source synthesis for content and data tasks.',
        '["content", "data"]',
        'claude-sonnet-4-20250514',
        '["web_search", "read_url", "parse_pdf"]',
        4.5,
        0
    ),
    (
        'monitor',
        'Monitor Agent',
        'Watches for conditions and alerts when met. Web monitoring, price alerts, on-chain events.',
        '["monitor", "crypto"]',
        'claude-3-5-haiku-latest',
        '["web_search", "read_url", "solana_rpc"]',
        4.5,
        0
    );
