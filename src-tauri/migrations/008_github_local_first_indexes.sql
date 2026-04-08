CREATE INDEX IF NOT EXISTS idx_external_items_source_kind_state ON external_items(source, kind, state);
CREATE INDEX IF NOT EXISTS idx_external_items_repo_updated_ext ON external_items(repo_full, updated_at_ext);
CREATE INDEX IF NOT EXISTS idx_task_external_links_external_item ON task_external_links(external_item_id);
CREATE INDEX IF NOT EXISTS idx_task_external_links_task ON task_external_links(task_id);
CREATE INDEX IF NOT EXISTS idx_github_repo_subscriptions_account_enabled ON github_repo_subscriptions(account_id, enabled);
