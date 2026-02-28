CREATE INDEX IF NOT EXISTS idx_tasks_project_sort_index ON tasks(project_id, sort_index);
CREATE INDEX IF NOT EXISTS idx_tasks_project_due_date ON tasks(project_id, due_date);
CREATE INDEX IF NOT EXISTS idx_tasks_due_date_v2 ON tasks(due_date);
