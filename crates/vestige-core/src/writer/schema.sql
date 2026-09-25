CREATE TABLE IF NOT EXISTS writer_roles (
    id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
    active_version INTEGER NOT NULL DEFAULT 0, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS writer_versions (
    role_id TEXT NOT NULL REFERENCES writer_roles(id) ON DELETE CASCADE,
    version INTEGER NOT NULL, base_version INTEGER NOT NULL, status TEXT NOT NULL,
    rules_json TEXT NOT NULL, summary TEXT NOT NULL, task_id TEXT,
    created_at TEXT NOT NULL, PRIMARY KEY(role_id, version)
);
CREATE TABLE IF NOT EXISTS writer_sources (
    id TEXT PRIMARY KEY, role_id TEXT NOT NULL REFERENCES writer_roles(id) ON DELETE CASCADE,
    title TEXT NOT NULL, filename TEXT NOT NULL, format TEXT NOT NULL, sha256 TEXT NOT NULL,
    raw_bytes BLOB NOT NULL, text TEXT NOT NULL, metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL, UNIQUE(role_id, sha256)
);
CREATE TABLE IF NOT EXISTS writer_segments (
    id TEXT PRIMARY KEY, source_id TEXT NOT NULL REFERENCES writer_sources(id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL, text TEXT NOT NULL, start_line INTEGER NOT NULL, end_line INTEGER NOT NULL,
    UNIQUE(source_id, ordinal)
);
CREATE TABLE IF NOT EXISTS writer_messages (
    id TEXT PRIMARY KEY, role_id TEXT NOT NULL REFERENCES writer_roles(id) ON DELETE CASCADE,
    speaker TEXT NOT NULL, content TEXT NOT NULL, task_id TEXT, created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS writer_projects (
    id TEXT PRIMARY KEY, role_id TEXT NOT NULL REFERENCES writer_roles(id) ON DELETE RESTRICT,
    role_version INTEGER NOT NULL, name TEXT NOT NULL, format TEXT NOT NULL, brief TEXT NOT NULL,
    canon_json TEXT NOT NULL DEFAULT '{}', revision INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS writer_tasks (
    id TEXT PRIMARY KEY, role_id TEXT NOT NULL REFERENCES writer_roles(id) ON DELETE CASCADE,
    project_id TEXT REFERENCES writer_projects(id) ON DELETE CASCADE,
    kind TEXT NOT NULL, base_version INTEGER NOT NULL, input_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'queued', agent_id TEXT, lease_token TEXT,
    lease_until INTEGER, result_json TEXT, result_hash TEXT, error TEXT,
    created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS writer_tasks_queue ON writer_tasks(status, created_at);
CREATE TABLE IF NOT EXISTS writer_drafts (
    id TEXT NOT NULL, revision INTEGER NOT NULL,
    project_id TEXT NOT NULL REFERENCES writer_projects(id) ON DELETE CASCADE,
    role_version INTEGER NOT NULL, kind TEXT NOT NULL, title TEXT NOT NULL, content TEXT NOT NULL,
    task_id TEXT, created_at TEXT NOT NULL, PRIMARY KEY(id, revision)
);
CREATE INDEX IF NOT EXISTS writer_drafts_project ON writer_drafts(project_id, created_at);
CREATE TABLE IF NOT EXISTS writer_reviews (
    id TEXT PRIMARY KEY, draft_id TEXT NOT NULL, draft_revision INTEGER NOT NULL,
    role_version INTEGER NOT NULL, summary TEXT NOT NULL, findings_json TEXT NOT NULL,
    task_id TEXT, created_at TEXT NOT NULL,
    FOREIGN KEY(draft_id, draft_revision) REFERENCES writer_drafts(id, revision) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS writer_audit (
    id TEXT PRIMARY KEY, action TEXT NOT NULL, entity_id TEXT NOT NULL, created_at TEXT NOT NULL
);
UPDATE schema_version SET version = 37, applied_at = datetime('now');
