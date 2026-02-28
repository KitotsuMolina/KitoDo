PRAGMA foreign_keys=OFF;

CREATE TABLE IF NOT EXISTS tasks_new (
  id TEXT PRIMARY KEY,
  project_id TEXT NULL,
  title TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('todo', 'done')),
  priority INTEGER NOT NULL CHECK (priority BETWEEN 1 AND 4),
  due_date TEXT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  completed_at TEXT NULL,
  deleted_at TEXT NULL,
  sort_index INTEGER NULL,
  recurrence TEXT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE SET NULL
);

INSERT INTO tasks_new (
  id, project_id, title, status, priority, due_date,
  created_at, updated_at, completed_at, deleted_at, sort_index, recurrence
)
SELECT
  id, project_id, title, status, priority, due_date,
  created_at, updated_at, completed_at, deleted_at, sort_index, recurrence
FROM tasks;

DROP TABLE tasks;
ALTER TABLE tasks_new RENAME TO tasks;

PRAGMA foreign_keys=ON;
