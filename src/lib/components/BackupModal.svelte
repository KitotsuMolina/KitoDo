<script lang="ts">
  import { Download, Upload, X } from 'lucide-svelte';

  export let open = false;
  export let busy = false;
  export let draft = '';
  export let notice = '';
  export let fileInputRef: HTMLInputElement | null = null;
  export let onClose: () => void;
  export let onExport: () => void;
  export let onTriggerImportFile: () => void;
  export let onImportFile: (event: Event) => void;
  export let onImportDraft: () => void;
</script>

{#if open}
  <div class="modal-backdrop">
    <div class="confirm-modal backup-modal">
      <div class="modal-head">
        <h4>Exportar / importar tareas</h4>
        <button class="icon-button" aria-label="Cerrar respaldo" on:click={onClose}>
          <X size={18} strokeWidth={2.1} />
        </button>
      </div>
      <p>Genera un JSON con tus tareas activas y completadas, o importa un backup existente haciendo merge por ID.</p>
      <div class="backup-actions">
        <button class="button-primary button-with-icon" disabled={busy} on:click={onExport}>
          <Download size={16} strokeWidth={2.1} />
          <span>Exportar JSON</span>
        </button>
        <button class="button-secondary button-with-icon" disabled={busy} on:click={onTriggerImportFile}>
          <Upload size={16} strokeWidth={2.1} />
          <span>Importar archivo</span>
        </button>
        <input
          bind:this={fileInputRef}
          class="hidden-file"
          type="file"
          accept=".json,application/json"
          on:change={onImportFile}
        />
      </div>
      <textarea
        class="backup-textarea"
        bind:value={draft}
        placeholder="Pega aquí un backup JSON para importarlo"
      ></textarea>
      {#if notice}
        <p class="backup-notice">{notice}</p>
      {/if}
      <div class="confirm-actions">
        <button class="button-secondary button-with-icon" on:click={onClose}>
          <X size={16} strokeWidth={2.1} />
          <span>Cerrar</span>
        </button>
        <button class="button-primary button-with-icon" disabled={busy || !draft.trim()} on:click={onImportDraft}>
          <Upload size={16} strokeWidth={2.1} />
          <span>Importar JSON pegado</span>
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

  .backup-modal {
    width: min(680px, 94vw);
  }

  .modal-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  .button-primary,
  .button-secondary {
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

  .button-primary:hover,
  .button-secondary:hover {
    transform: translateY(-1px);
  }

  .button-primary:hover {
    box-shadow: 0 0 18px rgba(192, 75, 255, 0.22);
  }

  .button-secondary:hover {
    border-color: rgba(192, 75, 255, 0.45);
    background: rgba(192, 75, 255, 0.1);
  }

  .button-primary:disabled,
  .button-secondary:disabled {
    opacity: 0.55;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
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

  .backup-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .backup-textarea {
    width: 100%;
    min-height: 220px;
    resize: vertical;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: var(--k-text);
    padding: 10px 12px;
    font: inherit;
    scrollbar-color: rgba(192, 75, 255, 0.56) rgba(255, 255, 255, 0.05);
  }

  .backup-textarea:focus {
    outline: none;
    border-color: rgba(192, 75, 255, 0.72);
    box-shadow: 0 0 0 2px rgba(166, 12, 219, 0.2);
  }

  .backup-notice {
    color: #dcb4ff !important;
  }

  .hidden-file {
    display: none;
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

    .backup-modal {
      width: min(94vw, 680px);
    }

    .backup-actions,
    .confirm-actions {
      grid-template-columns: 1fr;
      flex-direction: column;
      align-items: stretch;
    }

    .backup-textarea {
      min-height: 180px;
    }
  }

  @media (max-width: 460px) {
    .backup-actions button,
    .confirm-actions button {
      width: 100%;
    }
  }
</style>
