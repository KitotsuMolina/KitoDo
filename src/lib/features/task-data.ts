import {
  listInbox,
  listLabels,
  listOverdue,
  listProjectTasks,
  listProjects,
  listToday,
  listUpcoming,
  quickAdd,
  restoreTask,
  softDeleteTask,
  toggleTaskWithRecurrence,
  updateTaskTitle,
  type LabelDTO,
  type ProjectDTO,
  type TaskDTO
} from '$lib/api/desktop';

export type TaskDataSnapshot = {
  tasksByTab: {
    inbox: TaskDTO[];
    today: TaskDTO[];
    upcoming: TaskDTO[];
  };
  overdueTasks: TaskDTO[];
  projectTasks: TaskDTO[];
  projects: ProjectDTO[];
  labels: LabelDTO[];
  todayAll: TaskDTO[];
};

export async function loadTaskData(showDone: boolean, selectedProjectId: string | null): Promise<TaskDataSnapshot> {
  const [inbox, todayVisible, upcoming, overdue, projects, labels, todayAll] = await Promise.all([
    listInbox(showDone),
    listToday(showDone),
    listUpcoming(7, showDone),
    listOverdue(showDone),
    listProjects(),
    listLabels(),
    listToday(true)
  ]);

  return {
    tasksByTab: {
      inbox,
      today: todayVisible,
      upcoming
    },
    overdueTasks: overdue,
    projectTasks: selectedProjectId ? await listProjectTasks(selectedProjectId, showDone) : [],
    projects,
    labels,
    todayAll
  };
}

export async function loadProjectTaskData(selectedProjectId: string | null, showDone: boolean): Promise<TaskDTO[]> {
  if (!selectedProjectId) {
    return [];
  }
  return listProjectTasks(selectedProjectId, showDone);
}

export async function createTaskFromQuickInput(value: string): Promise<void> {
  await quickAdd(value.trim());
}

export async function toggleTask(taskId: string) {
  return toggleTaskWithRecurrence(taskId);
}

export async function saveTaskTitle(taskId: string, title: string): Promise<void> {
  await updateTaskTitle(taskId, title.trim());
}

export async function deleteTaskById(taskId: string): Promise<void> {
  await softDeleteTask(taskId);
}

export async function restoreTaskById(taskId: string): Promise<void> {
  await restoreTask(taskId);
}
