import {
  githubAddRepoSubscription,
  githubConnect,
  githubDisconnect,
  githubGetSettings,
  githubGetStatus,
  githubListAccounts,
  githubListRepoSubscriptions,
  githubRemoveRepoSubscription,
  githubSetSettings,
  githubSyncNow,
  githubToggleRepoSubscription,
  type GithubAccountDTO,
  type GithubSettingsDTO,
  type GithubStatusDTO,
  type RepoSubDTO
} from '$lib/api/desktop';

export type GithubStateSnapshot = {
  accounts: GithubAccountDTO[];
  settings: GithubSettingsDTO | null;
  repoSubs: RepoSubDTO[];
  status: GithubStatusDTO | null;
  selectedAccountId: string | null;
};

export type GithubSettingsPatch = {
  enabled?: boolean;
  syncIntervalSec?: number;
  importPrReviews?: boolean;
  importAssignedIssues?: boolean;
  importNotifications?: boolean;
  defaultProjectId?: string | null;
};

export function parseRepoInput(value: string): { owner: string; repo: string } {
  const trimmed = value.trim();
  if (!trimmed.includes('/')) {
    throw new Error('Formato repo inválido. Usa owner/repo');
  }

  const [owner, repo] = trimmed.split('/', 2);
  if (!owner || !repo) {
    throw new Error('Formato repo inválido. Usa owner/repo');
  }

  return { owner, repo };
}

export async function loadGithubState(selectedAccountId: string | null): Promise<GithubStateSnapshot> {
  const accounts = await githubListAccounts();
  if (accounts.length === 0) {
    return {
      accounts,
      settings: null,
      repoSubs: [],
      status: null,
      selectedAccountId: null
    };
  }

  const nextSelectedAccountId =
    selectedAccountId && accounts.some((account) => account.accountId === selectedAccountId)
      ? selectedAccountId
      : accounts[0].accountId;

  const [settings, repoSubs, status] = await Promise.all([
    githubGetSettings(nextSelectedAccountId),
    githubListRepoSubscriptions(nextSelectedAccountId),
    githubGetStatus(nextSelectedAccountId)
  ]);

  return {
    accounts,
    settings,
    repoSubs,
    status,
    selectedAccountId: nextSelectedAccountId
  };
}

export async function connectGithubAccount(token: string): Promise<{ account: GithubAccountDTO; message: string }> {
  const account = await githubConnect(token.trim());
  return {
    account,
    message: `Conectado como ${account.username}`
  };
}

export async function disconnectGithubAccount(accountId: string): Promise<string> {
  await githubDisconnect(accountId);
  return 'Cuenta desconectada';
}

export async function syncGithubAccount(accountId: string): Promise<string> {
  const result = await githubSyncNow(accountId);
  return `Sincronización: +${result.createdTasks} nuevas, ${result.updatedTasks} actualizadas, ${result.closedTasks} cerradas`;
}

export async function addGithubRepo(accountId: string, repoInput: string): Promise<void> {
  const { owner, repo } = parseRepoInput(repoInput);
  await githubAddRepoSubscription(accountId, owner, repo);
}

export async function toggleGithubRepo(sub: RepoSubDTO): Promise<void> {
  await githubToggleRepoSubscription(sub.id, !sub.enabled);
}

export async function removeGithubRepo(sub: RepoSubDTO): Promise<void> {
  await githubRemoveRepoSubscription(sub.id);
}

export async function updateGithubSettings(accountId: string, patch: GithubSettingsPatch): Promise<GithubSettingsDTO> {
  return githubSetSettings(accountId, patch);
}
