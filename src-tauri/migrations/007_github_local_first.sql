ALTER TABLE tasks ADD COLUMN external_url TEXT NULL;

CREATE TABLE IF NOT EXISTS github_accounts (
  account_id TEXT PRIMARY KEY,
  username TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  token_kind TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS github_settings (
  account_id TEXT PRIMARY KEY,
  enabled INTEGER NOT NULL DEFAULT 1,
  sync_interval_sec INTEGER NOT NULL DEFAULT 300,
  import_pr_reviews INTEGER NOT NULL DEFAULT 1,
  import_assigned_issues INTEGER NOT NULL DEFAULT 1,
  import_notifications INTEGER NOT NULL DEFAULT 0,
  default_project_id TEXT NULL,
  FOREIGN KEY(account_id) REFERENCES github_accounts(account_id) ON DELETE CASCADE,
  FOREIGN KEY(default_project_id) REFERENCES projects(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS github_repo_subscriptions (
  id TEXT PRIMARY KEY,
  account_id TEXT NOT NULL,
  owner TEXT NOT NULL,
  repo TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  last_synced_at TEXT NULL,
  last_etag_pr TEXT NULL,
  last_etag_issues TEXT NULL,
  last_etag_notifications TEXT NULL,
  FOREIGN KEY(account_id) REFERENCES github_accounts(account_id) ON DELETE CASCADE,
  UNIQUE(account_id, owner, repo)
);

CREATE TABLE IF NOT EXISTS external_items (
  id TEXT PRIMARY KEY,
  source TEXT NOT NULL,
  external_key TEXT NOT NULL,
  kind TEXT NOT NULL,
  url TEXT NOT NULL,
  title TEXT NOT NULL,
  state TEXT NOT NULL,
  repo_full TEXT NOT NULL,
  number INTEGER NULL,
  author TEXT NULL,
  assignee TEXT NULL,
  updated_at_ext TEXT NULL,
  payload_json TEXT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(source, external_key)
);

CREATE TABLE IF NOT EXISTS task_external_links (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  external_item_id TEXT NOT NULL,
  user_modified_title INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE,
  FOREIGN KEY(external_item_id) REFERENCES external_items(id) ON DELETE CASCADE,
  UNIQUE(task_id, external_item_id),
  UNIQUE(external_item_id)
);
