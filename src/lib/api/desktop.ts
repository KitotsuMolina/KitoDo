function invoke<T>(command: string, payload: Record<string, unknown> = {}): Promise<T> {
  return window.kitodo.invoke<T>(command, payload);
}

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
  externalUrl: string | null;
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

export type GithubAccountDTO = {
  accountId: string;
  username: string;
  tokenKind: 'classic' | 'fine_grained' | 'unknown' | string;
  createdAt: string;
};

export type GithubSettingsDTO = {
  accountId: string;
  enabled: boolean;
  syncIntervalSec: number;
  importPrReviews: boolean;
  importAssignedIssues: boolean;
  importNotifications: boolean;
  defaultProjectId: string | null;
};

export type RepoDTO = {
  owner: string;
  repo: string;
  fullName: string;
  private: boolean;
};

export type RepoSubDTO = {
  id: string;
  accountId: string;
  owner: string;
  repo: string;
  enabled: boolean;
  lastSyncedAt: string | null;
};

export type GithubSyncResultDTO = {
  fetched: number;
  createdTasks: number;
  updatedTasks: number;
  closedTasks: number;
  errors: string[];
};

export type GithubStatusDTO = {
  accountId: string;
  username: string;
  enabled: boolean;
  lastSyncAt: string | null;
  lastError: string | null;
};

export type GithubExternalItemDTO = {
  id: string;
  kind: string;
  url: string;
  title: string;
  state: string;
  repoFull: string;
  number: number | null;
  updatedAtExt: string | null;
};

export type ImportResultDTO = {
  importedProjects: number;
  createdTasks: number;
  updatedTasks: number;
  linkedLabels: number;
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

export async function exportBackupJson(): Promise<string> {
  return invoke<string>('export_backup_json');
}

export async function importBackupJson(json: string): Promise<ImportResultDTO> {
  return invoke<ImportResultDTO>('import_backup_json', { json });
}

export async function githubConnect(token: string): Promise<GithubAccountDTO> {
  return invoke<GithubAccountDTO>('github_connect', { token });
}

export async function githubDisconnect(accountId: string): Promise<boolean> {
  return invoke<boolean>('github_disconnect', { accountId });
}

export async function githubListAccounts(): Promise<GithubAccountDTO[]> {
  return invoke<GithubAccountDTO[]>('github_list_accounts');
}

export async function githubGetSettings(accountId: string): Promise<GithubSettingsDTO> {
  return invoke<GithubSettingsDTO>('github_get_settings', { accountId });
}

export async function githubSetSettings(
  accountId: string,
  settingsPatch: {
    enabled?: boolean;
    syncIntervalSec?: number;
    importPrReviews?: boolean;
    importAssignedIssues?: boolean;
    importNotifications?: boolean;
    defaultProjectId?: string | null;
  }
): Promise<GithubSettingsDTO> {
  return invoke<GithubSettingsDTO>('github_set_settings', {
    accountId,
    settingsPatch: {
      enabled: settingsPatch.enabled,
      syncIntervalSec: settingsPatch.syncIntervalSec,
      importPrReviews: settingsPatch.importPrReviews,
      importAssignedIssues: settingsPatch.importAssignedIssues,
      importNotifications: settingsPatch.importNotifications,
      defaultProjectId:
        settingsPatch.defaultProjectId === undefined ? undefined : settingsPatch.defaultProjectId
    }
  });
}

export async function githubListRepos(accountId: string): Promise<RepoDTO[]> {
  return invoke<RepoDTO[]>('github_list_repos', { accountId });
}

export async function githubAddRepoSubscription(accountId: string, owner: string, repo: string): Promise<RepoSubDTO> {
  return invoke<RepoSubDTO>('github_add_repo_subscription', { accountId, owner, repo });
}

export async function githubRemoveRepoSubscription(id: string): Promise<boolean> {
  return invoke<boolean>('github_remove_repo_subscription', { id });
}

export async function githubToggleRepoSubscription(id: string, enabled: boolean): Promise<RepoSubDTO> {
  return invoke<RepoSubDTO>('github_toggle_repo_subscription', { id, enabled });
}

export async function githubListRepoSubscriptions(accountId: string): Promise<RepoSubDTO[]> {
  return invoke<RepoSubDTO[]>('github_list_repo_subscriptions', { accountId });
}

export async function githubSyncNow(accountId: string): Promise<GithubSyncResultDTO> {
  return invoke<GithubSyncResultDTO>('github_sync_now', { accountId });
}

export async function githubGetStatus(accountId: string): Promise<GithubStatusDTO> {
  return invoke<GithubStatusDTO>('github_get_status', { accountId });
}

export async function githubListExternalItems(limit = 100): Promise<GithubExternalItemDTO[]> {
  return invoke<GithubExternalItemDTO[]>('github_list_external_items', { limit });
}
