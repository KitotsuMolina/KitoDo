<script lang="ts">
  import { Eye, X } from 'lucide-svelte';
  import type { ShortcutSection } from '$lib/constants/app-content';

  export let open = false;
  export let sections: ShortcutSection[] = [];
  export let dueNotificationsEnabled = true;
  export let dueNotificationsSupported = true;
  export let onToggleDueNotifications: () => void;
  export let onClose: () => void;
  export let onShowGuide: () => void;
</script>

{#if open}
  <div class="modal-backdrop">
    <div class="confirm-modal help-modal">
      <div class="modal-head">
        <h4>Guía rápida de KitoDo</h4>
        <button class="icon-button" aria-label="Cerrar ayuda" on:click={onClose}>
          <X size={18} strokeWidth={2.1} />
        </button>
      </div>
      <p>
        KitoDo está pensado para capturar tareas rápido y luego refinarlas desde filtros y el panel lateral.
        Los atajos globales no se disparan mientras escribes en un campo de texto.
      </p>
      <section class="preferences-card">
        <div class="preferences-copy">
          <strong>Notificaciones de tareas con fecha</strong>
          <span>
            {#if dueNotificationsSupported}
              Recibe avisos locales para tareas de hoy y tareas vencidas.
            {:else}
              Este sistema no soporta notificaciones nativas desde Electron.
            {/if}
          </span>
        </div>
        <label class="toggle-row" class:disabled={!dueNotificationsSupported}>
          <input
            type="checkbox"
            checked={dueNotificationsEnabled}
            disabled={!dueNotificationsSupported}
            on:change={onToggleDueNotifications}
          />
          <span>{dueNotificationsEnabled ? 'Activadas' : 'Desactivadas'}</span>
        </label>
      </section>
      <div class="shortcut-sections">
        {#each sections as section}
          <section class="shortcut-section">
            <h5>{section.title}</h5>
            <div class="shortcut-list">
              {#each section.items as item}
                <div class="shortcut-row">
                  <div class="shortcut-keys">
                    {#each item.keys as key}
                      <kbd>{key}</kbd>
                    {/each}
                  </div>
                  <span>{item.description}</span>
                </div>
              {/each}
            </div>
          </section>
        {/each}
      </div>
      <div class="confirm-actions">
        <button class="ghost-trigger compact button-with-icon" on:click={onShowGuide}>
          <Eye size={16} strokeWidth={2.1} />
          <span>Mostrar guía</span>
        </button>
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

  .help-modal {
    width: min(760px, 94vw);
    max-height: min(88vh, 860px);
    overflow: auto;
    overscroll-behavior: contain;
    scrollbar-color: rgba(192, 75, 255, 0.56) rgba(255, 255, 255, 0.05);
  }

  .modal-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
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

  .button-secondary {
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.14);
    color: var(--k-text);
    padding: 9px 12px;
    cursor: pointer;
    transition: border-color 180ms ease, box-shadow 180ms ease, background 180ms ease, transform 180ms ease;
  }

  .button-secondary:hover {
    transform: translateY(-1px);
    border-color: rgba(192, 75, 255, 0.45);
    background: rgba(192, 75, 255, 0.1);
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

  .shortcut-sections {
    display: grid;
    gap: 12px;
  }

  .preferences-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-radius: 12px;
    border: 1px solid rgba(192, 75, 255, 0.2);
    background: rgba(255, 255, 255, 0.04);
  }

  .preferences-copy {
    display: grid;
    gap: 4px;
  }

  .preferences-copy strong {
    font-size: 0.88rem;
  }

  .preferences-copy span {
    color: var(--k-muted);
    font-size: 0.82rem;
    line-height: 1.4;
  }

  .toggle-row {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    white-space: nowrap;
    color: var(--k-text);
    font-size: 0.84rem;
  }

  .toggle-row.disabled {
    opacity: 0.55;
  }

  .toggle-row input {
    accent-color: #c04bff;
  }

  .shortcut-section {
    display: grid;
    gap: 8px;
  }

  .shortcut-section h5 {
    margin: 0;
    font-size: 0.8rem;
    color: var(--k-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .shortcut-list {
    display: grid;
    gap: 8px;
  }

  .shortcut-row {
    display: grid;
    grid-template-columns: minmax(180px, 220px) minmax(0, 1fr);
    gap: 12px;
    align-items: center;
  }

  .shortcut-row span {
    color: var(--k-text);
    font-size: 0.9rem;
  }

  .shortcut-keys {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  kbd {
    border-radius: 8px;
    border: 1px solid rgba(192, 75, 255, 0.32);
    background: linear-gradient(180deg, rgba(40, 26, 54, 0.96), rgba(21, 16, 30, 0.96));
    color: #f3e8ff;
    padding: 5px 8px;
    font: inherit;
    font-size: 0.8rem;
    line-height: 1;
    box-shadow: inset 0 -1px 0 rgba(255, 255, 255, 0.08);
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  @media (max-width: 960px) {
    .modal-head {
      align-items: stretch;
      flex-direction: column;
    }

    .help-modal {
      width: min(94vw, 760px);
    }

    .shortcut-row {
      grid-template-columns: 1fr;
      gap: 6px;
    }

    .preferences-card {
      flex-direction: column;
      align-items: stretch;
    }

    .confirm-actions {
      grid-template-columns: 1fr;
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
