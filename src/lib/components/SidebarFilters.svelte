<script lang="ts">
  import { ExternalLink, RotateCcw, X } from 'lucide-svelte';
  import type { GithubSettingsDTO, GithubStatusDTO, LabelDTO, ProjectDTO } from '$lib/api/desktop';

  type SidebarTab = 'inbox' | 'today' | 'upcoming';

  export let tabNames: Record<SidebarTab, string>;
  export let tabCounts: Record<SidebarTab, number>;
  export let currentTab: SidebarTab;
  export let projects: ProjectDTO[] = [];
  export let labels: LabelDTO[] = [];
  export let selectedProjectId: string | null = null;
  export let selectedLabel: string | null = null;
  export let githubAccountsCount = 0;
  export let selectedGithubAccountLabel = '';
  export let githubSettings: GithubSettingsDTO | null = null;
  export let githubStatus: GithubStatusDTO | null = null;
  export let onClose: () => void;
  export let onSelectTab: (tab: SidebarTab) => void;
  export let onSelectProject: (projectId: string | null) => void;
  export let onSelectLabel: (label: string | null) => void;
  export let onOpenGithub: () => void;
  export let onClearFilters: () => void;
</script>

<aside class="sidebar">
  <div class="sidebar-head">
    <div>
      <h2>Filtros</h2>
      <p>Abre o cierra este panel con el botón `Filtros`, `Shift + F` o `Escape`.</p>
    </div>
    <button class="icon-button" aria-label="Cerrar filtros" on:click={onClose}>
      <X size={18} strokeWidth={2.1} />
    </button>
  </div>

  <div class="side-block">
    <h3>Vistas</h3>
    {#each (Object.keys(tabNames) as SidebarTab[]) as tab}
      <button class:active={currentTab === tab} on:click={() => onSelectTab(tab)}>
        {tabNames[tab]} ({tabCounts[tab]})
      </button>
    {/each}
  </div>

  <div class="side-block">
    <h3>Proyectos</h3>
    <button class:active={selectedProjectId === null} on:click={() => onSelectProject(null)}>Todos</button>
    {#each projects as project}
      <button class:active={selectedProjectId === project.id} on:click={() => onSelectProject(project.id)}>
        @{project.name}
      </button>
    {/each}
  </div>

  <div class="side-block">
    <h3>Etiquetas</h3>
    <button class:active={selectedLabel === null} on:click={() => onSelectLabel(null)}>Todas</button>
    {#each labels as label}
      <button class:active={selectedLabel === label.name} on:click={() => onSelectLabel(label.name)}>
        #{label.name}
      </button>
    {/each}
  </div>

  <div class="side-block github-block">
    <h3>GitHub</h3>
    {#if githubAccountsCount === 0}
      <div class="sidebar-badges">
        <span class="status-badge">Sin cuenta conectada</span>
        <span class="status-badge">Abre el panel para configurar GitHub</span>
      </div>
    {:else}
      <div class="sidebar-badges">
        <span class="status-badge ok">{selectedGithubAccountLabel}</span>
        <span class="status-badge" class:ok={githubSettings?.enabled}>
          {githubSettings?.enabled ? 'Sync activa' : 'Sync desactivada'}
        </span>
        <span class="status-badge">
          {githubStatus?.lastSyncAt ? `Última sync: ${githubStatus.lastSyncAt}` : 'Sin sincronización'}
        </span>
      </div>
    {/if}
  </div>

  <button class="clear-filters button-with-icon" on:click={onOpenGithub}>
    <ExternalLink size={16} strokeWidth={2.1} />
    <span>Mostrar panel de GitHub</span>
  </button>
  <button class="clear-filters button-with-icon" on:click={onClearFilters}>
    <RotateCcw size={16} strokeWidth={2.1} />
    <span>Limpiar filtros</span>
  </button>
</aside>

<style>
  .sidebar {
    border-radius: 14px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(12, 12, 18, 0.7);
    padding: 10px;
    display: grid;
    gap: 10px;
    align-content: start;
    min-height: 0;
    overflow: auto;
    scrollbar-color: rgba(192, 75, 255, 0.62) rgba(255, 255, 255, 0.05);
  }

  .sidebar-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
    padding-bottom: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .sidebar-head h2 {
    margin: 0;
    font-size: 0.95rem;
    letter-spacing: 0.04em;
  }

  .sidebar-head p {
    margin: 4px 0 0;
    color: var(--k-muted);
    font-size: 0.8rem;
    line-height: 1.4;
  }

  .side-block {
    display: grid;
    gap: 6px;
  }

  .side-block h3 {
    margin: 0;
    color: var(--k-muted);
    font-size: 0.77rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .side-block button,
  .clear-filters {
    text-align: left;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: var(--k-text);
    padding: 6px 8px;
    cursor: pointer;
  }

  .side-block button.active,
  .clear-filters:hover {
    border-color: rgba(192, 75, 255, 0.66);
    box-shadow: 0 0 10px rgba(192, 75, 255, 0.24);
  }

  .github-block {
    gap: 8px;
  }

  .sidebar-badges {
    display: flex;
    gap: 6px;
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

  .button-with-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }

  .button-with-icon :global(svg) {
    flex: 0 0 auto;
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

  @media (max-width: 1280px) {
    .sidebar {
      order: 2;
      max-height: none;
    }
  }
</style>
