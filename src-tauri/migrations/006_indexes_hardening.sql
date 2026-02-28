CREATE INDEX IF NOT EXISTS idx_tasks_due_status_deleted ON tasks(due_date, status, deleted_at);
CREATE INDEX IF NOT EXISTS idx_tasks_project_sort_index_v2 ON tasks(project_id, sort_index);
CREATE INDEX IF NOT EXISTS idx_tasks_project_due_date_v2 ON tasks(project_id, due_date);
CREATE INDEX IF NOT EXISTS idx_tasks_due_date_v3 ON tasks(due_date);
CREATE INDEX IF NOT EXISTS idx_task_labels_task_id ON task_labels(task_id);
CREATE INDEX IF NOT EXISTS idx_task_labels_label_id ON task_labels(label_id);
