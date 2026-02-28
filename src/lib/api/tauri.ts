import { invoke } from '@tauri-apps/api/core';

export type TaskDTO = {
  id: string;
  title: string;
  status: 'todo' | 'done';
  priority: number;
  dueDate: string | null;
  projectId: string | null;
  projectName: string | null;
  labels: string[];
  updatedAt: string;
  recurrence: string | null;
  sortIndex: number | null;
};

export type ToggleResultDTO = {
  updatedTask: TaskDTO;
  spawnedTask: TaskDTO | null;
};

export type ProjectDTO = {
  id: string;
  name: string;
  sortMode: 'auto' | 'manual';
};

export type LabelDTO = {
  id: string;
  name: string;
};

export async function initDb(): Promise<boolean> {
  return invoke<boolean>('init_db');
}

export async function quickAdd(input: string): Promise<TaskDTO> {
  return invoke<TaskDTO>('quick_add', { input });
}

export async function listInbox(showDone: boolean): Promise<TaskDTO[]> {
  return invoke<TaskDTO[]>('list_inbox', { showDone });
}

export async function listToday(showDone: boolean): Promise<TaskDTO[]> {
  return invoke<TaskDTO[]>('list_today', { showDone });
}

export async function listOverdue(showDone: boolean): Promise<TaskDTO[]> {
  return invoke<TaskDTO[]>('list_overdue', { showDone });
}

export async function listUpcoming(days: number, showDone: boolean): Promise<TaskDTO[]> {
  return invoke<TaskDTO[]>('list_upcoming', { days, showDone });
}

export async function listProjectTasks(projectId: string, showDone: boolean): Promise<TaskDTO[]> {
  return invoke<TaskDTO[]>('list_project_tasks', { projectId, showDone });
}

export async function listProjects(): Promise<ProjectDTO[]> {
  return invoke<ProjectDTO[]>('list_projects');
}

export async function listLabels(): Promise<LabelDTO[]> {
  return invoke<LabelDTO[]>('list_labels');
}

export async function getProjectSortMode(projectId: string): Promise<'auto' | 'manual'> {
  return invoke<'auto' | 'manual'>('get_project_sort_mode', { projectId });
}

export async function setProjectSortMode(projectId: string, mode: 'auto' | 'manual'): Promise<ProjectDTO> {
  return invoke<ProjectDTO>('set_project_sort_mode', { projectId, mode });
}

export async function reorderProjectTasks(projectId: string, orderedTaskIds: string[]): Promise<boolean> {
  return invoke<boolean>('reorder_project_tasks', { projectId, orderedTaskIds });
}

export async function resetProjectManualOrder(projectId: string): Promise<boolean> {
  return invoke<boolean>('reset_project_manual_order', { projectId });
}

export async function toggleTask(id: string): Promise<TaskDTO> {
  return invoke<TaskDTO>('toggle_task', { id });
}

export async function toggleTaskWithRecurrence(id: string): Promise<ToggleResultDTO> {
  return invoke<ToggleResultDTO>('toggle_task_with_recurrence', { id });
}

export async function updateTaskTitle(id: string, title: string): Promise<TaskDTO> {
  return invoke<TaskDTO>('update_task_title', { id, title });
}

export async function updateTaskPriority(id: string, priority: number): Promise<TaskDTO> {
  return invoke<TaskDTO>('update_task_priority', { id, priority });
}

export async function updateTaskDueDate(id: string, dueDate: string | null): Promise<TaskDTO> {
  return invoke<TaskDTO>('update_task_due_date', { id, dueDate });
}

export async function updateTaskRecurrence(id: string, recurrence: string | null): Promise<TaskDTO> {
  return invoke<TaskDTO>('update_task_recurrence', { id, recurrence });
}

export async function moveTaskToProject(params: {
  id: string;
  projectId?: string | null;
  projectNameToCreate?: string | null;
}): Promise<TaskDTO> {
  return invoke<TaskDTO>('move_task_to_project', {
    id: params.id,
    projectId: params.projectId ?? null,
    projectNameToCreate: params.projectNameToCreate ?? null
  });
}

export async function softDeleteTask(id: string): Promise<boolean> {
  return invoke<boolean>('soft_delete_task', { id });
}

export async function restoreTask(id: string): Promise<TaskDTO> {
  return invoke<TaskDTO>('restore_task', { id });
}
