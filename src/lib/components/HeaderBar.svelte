<script lang="ts">
  import { BellOff, CircleHelp, Database, Filter, X } from 'lucide-svelte';

  export let pendingCount = 0;
  export let todayPendingCount = 0;
  export let todayDoneCount = 0;
  export let todayTotalCount = 0;
  export let dayProgress = 0;
  export let showDone = false;
  export let expandedMode = false;
  export let dueNotificationsEnabled = true;
  export let dueNotificationsSupported = true;
  export let onToggleShowDone: () => void;
  export let onToggleSidebar: () => void;
  export let onOpenHelp: () => void;
  export let onOpenBackup: () => void;
  export let onCloseApp: () => void;
</script>

<header class="header">
  <div class="header-left">
    <h1>KitoDo</h1>
    <p>Pendientes: {pendingCount} | Hoy: {todayPendingCount}</p>
    <label class="toggle top-left">
      <input type="checkbox" checked={showDone} on:change={onToggleShowDone} />
      <span class="toggle-switch" aria-hidden="true">
        <span class="toggle-thumb"></span>
      </span>
      <span>Mostrar completadas</span>
    </label>
  </div>
  <div class="header-actions">
    {#if dueNotificationsSupported && !dueNotificationsEnabled}
      <button class="header-status-badge" on:click={onOpenHelp} title="Las notificaciones están desactivadas. Abrir ayuda.">
        <BellOff size={14} strokeWidth={2.1} />
        <span>Notificaciones desactivadas</span>
      </button>
    {/if}
    <div class="header-tool-buttons">
      <button
        class="header-tool-button"
        aria-label={expandedMode ? 'Ocultar filtros' : 'Mostrar filtros'}
        title={expandedMode ? 'Ocultar filtros' : 'Mostrar filtros'}
        on:click={onToggleSidebar}
      >
        <Filter size={18} strokeWidth={2.1} />
      </button>
      <button class="header-tool-button" aria-label="Abrir ayuda" title="Abrir ayuda" on:click={onOpenHelp}>
        <CircleHelp size={18} strokeWidth={2.1} />
      </button>
      <button
        class="header-tool-button header-tool-button--accent"
        aria-label="Abrir respaldo"
        title="Abrir respaldo"
        on:click={onOpenBackup}
      >
        <Database size={18} strokeWidth={2.1} />
      </button>
      <button
        class="header-tool-button header-tool-button--danger"
        aria-label="Cerrar aplicación"
        title="Cerrar aplicación"
        on:click={onCloseApp}
      >
        <X size={18} strokeWidth={2.1} />
      </button>
    </div>
    <div class="progress-wrap" title="Progreso del día">
      <span>{todayDoneCount}/{todayTotalCount}</span>
      <div class="progress-bar"><div style={`width:${dayProgress}%`}></div></div>
    </div>
  </div>
</header>

<style>
  .header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 10px;
    flex-wrap: wrap;
  }

  .header-left {
    display: grid;
    gap: 6px;
    justify-items: start;
    min-width: 0;
  }

  .header-actions {
    display: grid;
    gap: 8px;
    justify-items: end;
    margin-left: auto;
  }

  .header-status-badge {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 34px;
    border-radius: 999px;
    border: 1px solid rgba(255, 190, 92, 0.28);
    background: rgba(255, 190, 92, 0.1);
    color: #ffe1a6;
    padding: 6px 10px;
    font-size: 0.78rem;
    cursor: pointer;
    transition: transform 180ms ease, border-color 180ms ease, background 180ms ease;
  }

  .header-status-badge:hover {
    transform: translateY(-1px);
    border-color: rgba(255, 190, 92, 0.5);
    background: rgba(255, 190, 92, 0.16);
  }

  .header-tool-buttons {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    flex-wrap: wrap;
  }

  .header h1 {
    margin: 0;
    font-size: 1.2rem;
    letter-spacing: 0.04em;
  }

  .header p {
    margin: 5px 0 0;
    color: var(--k-muted);
    font-size: 0.86rem;
  }

  .progress-wrap {
    min-width: 118px;
    font-size: 0.78rem;
    color: var(--k-muted);
    text-align: right;
  }

  .header-tool-button {
    width: 40px;
    height: 40px;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.05);
    color: var(--k-text);
    display: inline-grid;
    place-items: center;
    cursor: pointer;
    transition: border-color 180ms ease, box-shadow 180ms ease, background 180ms ease, transform 180ms ease;
  }

  .header-tool-button:hover {
    transform: translateY(-1px);
    border-color: rgba(192, 75, 255, 0.46);
    background: rgba(192, 75, 255, 0.1);
    box-shadow: 0 0 14px rgba(192, 75, 255, 0.14);
  }

  .header-tool-button--accent {
    border-color: rgba(192, 75, 255, 0.42);
    background: linear-gradient(135deg, rgba(166, 12, 219, 0.24), rgba(192, 75, 255, 0.18));
    color: #efd8ff;
  }

  .header-tool-button--danger {
    border-color: rgba(255, 120, 145, 0.32);
    color: #ffd7e1;
  }

  .toggle {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 0.9rem;
    color: var(--k-muted);
    cursor: pointer;
    user-select: none;
  }

  .toggle.top-left {
    margin-top: 2px;
  }

  .toggle input {
    position: absolute;
    opacity: 0;
    width: 1px;
    height: 1px;
    pointer-events: none;
  }

  .toggle-switch {
    width: 42px;
    height: 24px;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.18);
    background: rgba(255, 255, 255, 0.08);
    padding: 2px;
    display: inline-flex;
    align-items: center;
    transition: background 280ms ease, border-color 280ms ease, box-shadow 280ms ease;
  }

  .toggle-thumb {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.9);
    transform: translateX(0);
    transition: transform 280ms cubic-bezier(0.22, 1, 0.36, 1), background 280ms ease;
  }

  .toggle input:checked + .toggle-switch {
    background: linear-gradient(135deg, rgba(166, 12, 219, 0.65), rgba(192, 75, 255, 0.55));
    border-color: rgba(192, 75, 255, 0.8);
    box-shadow: 0 0 16px rgba(192, 75, 255, 0.25);
  }

  .toggle input:checked + .toggle-switch .toggle-thumb {
    transform: translateX(18px);
    background: #ffffff;
  }

  .toggle input:focus-visible + .toggle-switch {
    box-shadow: 0 0 0 3px rgba(166, 12, 219, 0.25);
  }

  .progress-bar {
    margin-top: 4px;
    width: 118px;
    height: 8px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    overflow: hidden;
  }

  .progress-bar > div {
    height: 100%;
    background: linear-gradient(90deg, var(--k-purple), var(--k-purple-2));
    transition: width 200ms ease;
  }

  @media (max-width: 960px) {
    .header-actions {
      justify-items: end;
      width: auto;
    }

    .header-status-badge {
      max-width: min(100%, 280px);
      justify-content: center;
      text-align: center;
      white-space: normal;
    }

    .header-tool-buttons {
      justify-content: flex-end;
    }

    .header-left {
      justify-items: start;
    }

    .header h1,
    .header p {
      text-align: left;
    }

    .toggle {
      width: 100%;
      align-items: flex-start;
      line-height: 1.35;
    }

    .progress-wrap {
      width: auto;
      min-width: 0;
      text-align: right;
    }
  }

  @media (max-width: 460px) {
    .header-tool-button {
      width: 38px;
      height: 38px;
    }
  }
</style>
