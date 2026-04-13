<script lang="ts">
  import { ExternalLink, Plus, RefreshCw, Trash2, Unplug, X } from 'lucide-svelte';
  import type { GithubAccountDTO, GithubSettingsDTO, GithubStatusDTO, ProjectDTO, RepoSubDTO } from '$lib/api/desktop';
  import type { GithubDocLink, GithubSetupItem } from '$lib/constants/app-content';

  export let open = false;
  export let busy = false;
  export let message = '';
  export let githubAccounts: GithubAccountDTO[] = [];
  export let githubSettings: GithubSettingsDTO | null = null;
  export let githubRepoSubs: RepoSubDTO[] = [];
  export let githubStatus: GithubStatusDTO | null = null;
  export let selectedGithubAccountId: string | null = null;
  export let selectedGithubAccountLabel = 'Seleccionar cuenta';
  export let selectedGithubIntervalLabel = '5m';
  export let selectedGithubProjectLabel = 'Bandeja GitHub (auto)';
  export let githubTokenKindLabel = 'Sin token detectado';
  export let githubNotificationsReady = false;
  export let githubTokenInput = '';
  export let githubRepoInput = '';
  export let githubAccountDropdownOpen = false;
  export let githubIntervalDropdownOpen = false;
  export let githubProjectDropdownOpen = false;
  export let githubAccountMenuOpenUp = false;
  export let githubIntervalMenuOpenUp = false;
  export let githubProjectMenuOpenUp = false;
  export let githubSetupItems: GithubSetupItem[] = [];
  export let githubDocLinks: GithubDocLink[] = [];
  export let projects: ProjectDTO[] = [];
  export let githubAccountDropdownRef: HTMLDivElement | null = null;
  export let githubIntervalDropdownRef: HTMLDivElement | null = null;
  export let githubProjectDropdownRef: HTMLDivElement | null = null;
  export let githubAccountMenuRef: HTMLDivElement | null = null;
  export let githubIntervalMenuRef: HTMLDivElement | null = null;
  export let githubProjectMenuRef: HTMLDivElement | null = null;
  export let onClose: () => void;
  export let onOpenExternalUrl: (url: string) => void;
  export let onConnect: () => void;
  export let onDisconnect: () => void;
  export let onSyncNow: () => void;
  export let onAddRepo: () => void;
  export let onToggleRepo: (sub: RepoSubDTO) => void;
  export let onRemoveRepo: (sub: RepoSubDTO) => void;
  export let onUpdateSettings: (patch: {
    enabled?: boolean;
    syncIntervalSec?: number;
    importPrReviews?: boolean;
    importAssignedIssues?: boolean;
    importNotifications?: boolean;
    defaultProjectId?: string | null;
  }) => void;
  export let onSelectAccount: (account: GithubAccountDTO) => void;
  export let onToggleAccountDropdown: () => void;
  export let onToggleIntervalDropdown: () => void;
  export let onToggleProjectDropdown: () => void;
</script>

{#if open}
  <div class="modal-backdrop">
    <div class="confirm-modal github-modal">
      <div class="modal-head">
        <h4>Configuración de GitHub</h4>
        <button class="icon-button" aria-label="Cerrar configuración de GitHub" on:click={onClose}>
          <X size={18} strokeWidth={2.1} />
        </button>
      </div>

      <p>Usa este panel para conectar tu cuenta, elegir qué importar y activar notificaciones de GitHub dentro de KitoDo.</p>

      <section class="github-setup-card">
        <h5>Antes de conectar</h5>
        <div class="github-setup-list">
          {#each githubSetupItems as item}
            <div class="github-setup-item">
              <strong>{item.title}</strong>
              <span>{item.description}</span>
            </div>
          {/each}
        </div>
        <div class="github-doc-links">
          {#each githubDocLinks as link}
            <button class="ghost-trigger compact button-with-icon" on:click={() => onOpenExternalUrl(link.url)}>
              <ExternalLink size={16} strokeWidth={2.1} />
              <span>{link.label}</span>
            </button>
          {/each}
        </div>
      </section>

      {#if githubAccounts.length === 0}
        <section class="github-config-grid">
          <label class="github-field">
            <span>Personal access token</span>
            <input placeholder="ghp_..." bind:value={githubTokenInput} type="password" />
          </label>
          <div class="github-status-line">
            <small>
              Para importar notificaciones usa un token classic. Si solo conectas un token fine-grained, KitoDo podrá quedar limitado.
            </small>
          </div>
          <div class="confirm-actions">
            <button class="button-primary" disabled={busy || !githubTokenInput.trim()} on:click={onConnect}>Conectar GitHub</button>
          </div>
        </section>
      {:else}
        <section class="github-config-grid">
          <div class="github-status-card">
            <div class="github-status-head">
              <strong>{selectedGithubAccountLabel}</strong>
              <div class="status-badges">
                <span class="status-badge" class:ok={githubTokenKindLabel === 'Token classic'}>
                  {githubTokenKindLabel}
                </span>
                <span class="status-badge" class:ok={githubSettings?.enabled}>
                  {githubSettings?.enabled ? 'Sync activa' : 'Sync desactivada'}
                </span>
                <span class="status-badge" class:ok={githubNotificationsReady}>
                  {githubNotificationsReady ? 'Notificaciones listas' : 'Notificaciones limitadas'}
                </span>
              </div>
            </div>
            <div class="github-status-list">
              <span>
                {githubNotificationsReady
                  ? 'Tu token permite usar la API de notifications.'
                  : 'Para importar notificaciones necesitas un personal access token classic.'}
              </span>
              <span>
                {githubStatus?.lastSyncAt
                  ? `Última sincronización: ${githubStatus.lastSyncAt}`
                  : 'Todavía no se ha ejecutado ninguna sincronización.'}
              </span>
              {#if githubStatus?.lastError}
                <span class="status-error">Último error: {githubStatus.lastError}</span>
              {/if}
            </div>
          </div>

          <div class="context-dropdown" bind:this={githubAccountDropdownRef}>
            <button class="context-trigger" class:open={githubAccountDropdownOpen} on:click={onToggleAccountDropdown}>
              {selectedGithubAccountLabel}
              <span class="order-chevron">▾</span>
            </button>
            {#if githubAccountDropdownOpen}
              <div class="context-menu" class:open-up={githubAccountMenuOpenUp} bind:this={githubAccountMenuRef}>
                {#each githubAccounts as account}
                  <button class:active={selectedGithubAccountId === account.accountId} on:click={() => onSelectAccount(account)}>
                    {account.username} ({account.tokenKind})
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          {#if githubSettings}
            <label class="gh-inline">
              <input
                type="checkbox"
                checked={githubSettings.enabled}
                on:change={(e) => onUpdateSettings({ enabled: (e.currentTarget as HTMLInputElement).checked })}
              />
              Sincronización automática
            </label>
            <label class="gh-inline">
              <span>Intervalo</span>
              <div class="context-dropdown" bind:this={githubIntervalDropdownRef}>
                <button class="context-trigger" class:open={githubIntervalDropdownOpen} on:click={onToggleIntervalDropdown}>
                  {selectedGithubIntervalLabel}
                  <span class="order-chevron">▾</span>
                </button>
                {#if githubIntervalDropdownOpen}
                  <div class="context-menu" class:open-up={githubIntervalMenuOpenUp} bind:this={githubIntervalMenuRef}>
                    {#each [
                      { value: 60, label: '1m' },
                      { value: 300, label: '5m' },
                      { value: 600, label: '10m' },
                      { value: 1800, label: '30m' }
                    ] as opt}
                      <button
                        class:active={githubSettings.syncIntervalSec === opt.value}
                        on:click={() => onUpdateSettings({ syncIntervalSec: opt.value })}
                      >
                        {opt.label}
                      </button>
                    {/each}
                  </div>
                {/if}
              </div>
            </label>
            <label class="gh-inline">
              <input
                type="checkbox"
                checked={githubSettings.importPrReviews}
                on:change={(e) => onUpdateSettings({ importPrReviews: (e.currentTarget as HTMLInputElement).checked })}
              />
              Revisiones de PR
            </label>
            <label class="gh-inline">
              <input
                type="checkbox"
                checked={githubSettings.importAssignedIssues}
                on:change={(e) => onUpdateSettings({ importAssignedIssues: (e.currentTarget as HTMLInputElement).checked })}
              />
              Issues asignadas
            </label>
            <label class="gh-inline">
              <input
                type="checkbox"
                checked={githubSettings.importNotifications}
                on:change={(e) => onUpdateSettings({ importNotifications: (e.currentTarget as HTMLInputElement).checked })}
              />
              Notificaciones de GitHub
            </label>

            <label class="gh-inline">
              <span>Proyecto destino</span>
              <div class="context-dropdown" bind:this={githubProjectDropdownRef}>
                <button class="context-trigger" class:open={githubProjectDropdownOpen} on:click={onToggleProjectDropdown}>
                  {selectedGithubProjectLabel}
                  <span class="order-chevron">▾</span>
                </button>
                {#if githubProjectDropdownOpen}
                  <div class="context-menu" class:open-up={githubProjectMenuOpenUp} bind:this={githubProjectMenuRef}>
                    <button class:active={!githubSettings.defaultProjectId} on:click={() => onUpdateSettings({ defaultProjectId: null })}>
                      Bandeja GitHub (auto)
                    </button>
                    {#each projects as project}
                      <button
                        class:active={githubSettings.defaultProjectId === project.id}
                        on:click={() => onUpdateSettings({ defaultProjectId: project.id })}
                      >
                        @{project.name}
                      </button>
                    {/each}
                  </div>
                {/if}
              </div>
            </label>
          {/if}

          <div class="gh-actions">
            <button class="button-primary button-with-icon" disabled={busy} on:click={onSyncNow}>
              <RefreshCw size={16} strokeWidth={2.1} />
              <span>Sincronizar ahora</span>
            </button>
            <button class="button-secondary button-with-icon" disabled={busy} on:click={onDisconnect}>
              <Unplug size={16} strokeWidth={2.1} />
              <span>Desconectar</span>
            </button>
          </div>

          <div class="gh-add-repo">
            <input class="github-repo-input" placeholder="owner/repo" bind:value={githubRepoInput} />
            <button class="button-primary button-with-icon" disabled={busy} on:click={onAddRepo}>
              <Plus size={16} strokeWidth={2.1} />
              <span>Añadir</span>
            </button>
          </div>
          <div class="gh-repos">
            {#each githubRepoSubs as sub}
              <div class="gh-repo-item">
                <span>{sub.owner}/{sub.repo}</span>
                <div class="gh-actions">
                  <button class="button-secondary compact-action" on:click={() => onToggleRepo(sub)}>
                    {sub.enabled ? 'Desactivar' : 'Activar'}
                  </button>
                  <button class="button-danger compact-action button-with-icon" on:click={() => onRemoveRepo(sub)}>
                    <Trash2 size={14} strokeWidth={2.1} />
                    <span>Quitar</span>
                  </button>
                </div>
              </div>
            {/each}
          </div>
        </section>
      {/if}

      {#if message}
        <p class="backup-notice">{message}</p>
      {/if}

      <div class="confirm-actions">
        <button class="button-secondary button-with-icon" on:click={onClose}>
          <X size={16} strokeWidth={2.1} />
          <span>Cerrar</span>
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(2, 2, 5, 0.45);
    display: grid;
    place-items: center;
    z-index: 220;
  }

  .confirm-modal {
    width: min(420px, 92vw);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    background: rgba(20, 16, 30, 0.96);
    padding: 12px;
    display: grid;
    gap: 10px;
  }

  .confirm-modal h4 {
    margin: 0;
    font-size: 0.95rem;
  }

  .confirm-modal p {
    margin: 0;
    font-size: 0.86rem;
    color: var(--k-muted);
  }

  .modal-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  .github-modal {
    width: min(760px, 94vw);
    max-height: min(88vh, 920px);
    overflow: auto;
    overscroll-behavior: contain;
    scrollbar-color: rgba(192, 75, 255, 0.56) rgba(255, 255, 255, 0.05);
  }

  .icon-button {
    min-width: 34px;
    min-height: 34px;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.05);
    color: var(--k-text);
    cursor: pointer;
    display: inline-grid;
    place-items: center;
    padding: 0;
    flex: 0 0 auto;
  }

  .ghost-trigger {
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.05);
    color: var(--k-text);
    padding: 8px 12px;
    cursor: pointer;
    white-space: nowrap;
  }

  .ghost-trigger.compact {
    padding: 8px 10px;
  }

  .button-with-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }

  .button-with-icon :global(svg) {
    flex: 0 0 auto;
  }

  .button-primary,
  .button-secondary,
  .button-danger {
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    color: var(--k-text);
    padding: 9px 12px;
    cursor: pointer;
    transition: border-color 180ms ease, box-shadow 180ms ease, background 180ms ease, transform 180ms ease;
  }

  .button-primary {
    background: linear-gradient(135deg, rgba(166, 12, 219, 0.55), rgba(192, 75, 255, 0.42));
    border-color: rgba(192, 75, 255, 0.52);
    color: #f4eaff;
    box-shadow: 0 0 16px rgba(192, 75, 255, 0.16);
  }

  .button-secondary {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.14);
  }

  .button-danger {
    background: rgba(255, 120, 145, 0.12);
    border-color: rgba(255, 120, 145, 0.35);
    color: #ffd7e1;
  }

  .button-primary:hover,
  .button-secondary:hover,
  .button-danger:hover {
    transform: translateY(-1px);
  }

  .button-primary:hover {
    box-shadow: 0 0 18px rgba(192, 75, 255, 0.22);
  }

  .button-secondary:hover {
    border-color: rgba(192, 75, 255, 0.45);
    background: rgba(192, 75, 255, 0.1);
  }

  .button-danger:hover {
    border-color: rgba(255, 120, 145, 0.55);
    background: rgba(255, 120, 145, 0.18);
  }

  .button-primary:disabled,
  .button-secondary:disabled,
  .button-danger:disabled {
    opacity: 0.55;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
  }

  .compact-action {
    padding: 7px 10px;
    min-height: 34px;
    font-size: 0.78rem;
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .context-dropdown {
    position: relative;
  }

  .context-trigger {
    width: 100%;
    display: inline-flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.05);
    color: var(--k-text);
    padding: 7px 8px;
  }

  .context-trigger.open {
    border-color: rgba(192, 75, 255, 0.75);
    box-shadow: 0 0 0 2px rgba(166, 12, 219, 0.2);
  }

  .order-chevron {
    opacity: 0.9;
    font-size: 0.74rem;
  }

  .context-menu {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    width: 100%;
    max-height: 220px;
    overflow: auto;
    padding: 6px;
    display: grid;
    gap: 5px;
    border-radius: 10px;
    border: 1px solid rgba(192, 75, 255, 0.45);
    background: rgba(19, 16, 28, 0.98);
    box-shadow: 0 14px 24px rgba(0, 0, 0, 0.35), 0 0 18px rgba(192, 75, 255, 0.16);
    z-index: 80;
    scrollbar-color: rgba(192, 75, 255, 0.56) rgba(255, 255, 255, 0.04);
  }

  .context-menu.open-up {
    top: auto;
    bottom: calc(100% + 6px);
    transform-origin: bottom left;
  }

  .context-menu button {
    text-align: left;
    border-radius: 7px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.03);
    color: var(--k-text);
    padding: 7px 8px;
  }

  .context-menu button:hover {
    border-color: rgba(192, 75, 255, 0.55);
    background: rgba(192, 75, 255, 0.14);
  }

  .context-menu button.active {
    border-color: rgba(192, 75, 255, 0.8);
    background: linear-gradient(135deg, rgba(166, 12, 219, 0.45), rgba(192, 75, 255, 0.38));
    color: #fff;
  }

  .github-config-grid {
    display: grid;
    gap: 10px;
  }

  .github-field {
    display: grid;
    gap: 6px;
    color: var(--k-muted);
    font-size: 0.84rem;
  }

  .github-field input {
    width: 100%;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: var(--k-text);
    padding: 10px 12px;
  }

  .github-repo-input {
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(10, 10, 14, 0.7);
    color: var(--k-text);
    padding: 11px 12px;
    outline: none;
  }

  .github-repo-input:focus {
    border-color: rgba(192, 75, 255, 0.7);
    box-shadow: 0 0 0 3px rgba(166, 12, 219, 0.2);
  }

  .github-setup-card {
    display: grid;
    gap: 10px;
    padding: 12px;
    border-radius: 12px;
    border: 1px solid rgba(137, 198, 255, 0.22);
    background: rgba(255, 255, 255, 0.03);
  }

  .github-status-card {
    display: grid;
    gap: 10px;
    padding: 12px;
    border-radius: 12px;
    border: 1px solid rgba(192, 75, 255, 0.24);
    background:
      linear-gradient(135deg, rgba(192, 75, 255, 0.08), rgba(137, 198, 255, 0.05)),
      rgba(255, 255, 255, 0.03);
  }

  .github-status-head {
    display: grid;
    gap: 8px;
  }

  .github-status-head strong {
    font-size: 0.92rem;
  }

  .status-badges {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .status-badge {
    border-radius: 999px;
    padding: 5px 9px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.05);
    color: var(--k-muted);
    font-size: 0.78rem;
  }

  .status-badge.ok {
    border-color: rgba(155, 255, 184, 0.4);
    color: #c9ffd8;
    background: rgba(64, 120, 78, 0.2);
  }

  .github-status-list {
    display: grid;
    gap: 6px;
    color: var(--k-muted);
    font-size: 0.84rem;
    line-height: 1.45;
  }

  .status-error {
    color: #ffc4d3;
  }

  .github-setup-card h5 {
    margin: 0;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--k-muted);
  }

  .github-setup-list {
    display: grid;
    gap: 8px;
  }

  .github-setup-item {
    display: grid;
    gap: 4px;
  }

  .github-setup-item strong {
    font-size: 0.88rem;
  }

  .github-setup-item span,
  .github-status-line small {
    color: var(--k-muted);
    font-size: 0.84rem;
    line-height: 1.45;
  }

  .github-doc-links {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .gh-inline {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: 8px;
    font-size: 0.8rem;
    color: var(--k-muted);
  }

  .gh-actions {
    display: flex;
    gap: 6px;
  }

  .gh-add-repo {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 6px;
  }

  .gh-repos {
    display: grid;
    gap: 6px;
    max-height: 150px;
    overflow: auto;
    scrollbar-color: rgba(137, 198, 255, 0.46) rgba(255, 255, 255, 0.05);
  }

  .gh-repo-item {
    display: flex;
    justify-content: space-between;
    gap: 6px;
    align-items: center;
    font-size: 0.78rem;
    color: var(--k-text);
  }

  .backup-notice {
    color: #dcb4ff !important;
  }

  @media (max-width: 960px) {
    .modal-head {
      align-items: stretch;
      flex-direction: column;
    }

    .gh-inline,
    .gh-add-repo,
    .gh-repo-item,
    .confirm-actions {
      grid-template-columns: 1fr;
      flex-direction: column;
      align-items: stretch;
    }

    .gh-repo-item {
      display: grid;
    }

    .gh-actions {
      justify-content: flex-start;
      flex-wrap: wrap;
    }

    .github-modal {
      width: min(94vw, 760px);
      max-height: min(90vh, 920px);
    }

    .github-doc-links {
      flex-direction: column;
      align-items: stretch;
    }

    .status-badges {
      flex-direction: column;
    }
  }

  @media (max-width: 460px) {
    .confirm-actions button {
      width: 100%;
    }
  }
</style>
