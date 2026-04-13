<script lang="ts">
  import { X } from 'lucide-svelte';
  import { fade, fly } from 'svelte/transition';

  export let open = false;
  export let title = 'Confirmar';
  export let confirmLabel = 'Confirmar';
  export let cancelLabel = 'Cancelar';
  export let closeLabel = 'Cerrar confirmación';
  export let onClose: () => void = () => {};
  export let onConfirm: () => void = () => {};
</script>

{#if open}
  <div class="modal-backdrop" transition:fade={{ duration: 120 }}>
    <div class="confirm-modal" transition:fly={{ y: 10, duration: 160 }}>
      <div class="modal-head">
        <h4>{title}</h4>
        <button class="icon-button" aria-label={closeLabel} on:click={onClose}>
          <X size={18} strokeWidth={2.1} />
        </button>
      </div>

      <div class="content">
        <slot />
      </div>

      <div class="confirm-actions">
        <button class="button-secondary" on:click={onClose}>{cancelLabel}</button>
        <button class="button-danger" on:click={onConfirm}>{confirmLabel}</button>
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

  .modal-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  .modal-head h4 {
    margin: 0;
    font-size: 0.95rem;
  }

  .content :global(p) {
    margin: 0;
    font-size: 0.86rem;
    color: var(--k-muted);
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

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .button-secondary,
  .button-danger {
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    color: var(--k-text);
    padding: 9px 12px;
    cursor: pointer;
    transition: border-color 180ms ease, box-shadow 180ms ease, background 180ms ease, transform 180ms ease;
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

  .button-secondary:hover,
  .button-danger:hover {
    transform: translateY(-1px);
  }

  .button-secondary:hover {
    border-color: rgba(192, 75, 255, 0.45);
    background: rgba(192, 75, 255, 0.1);
  }

  .button-danger:hover {
    border-color: rgba(255, 120, 145, 0.55);
    background: rgba(255, 120, 145, 0.18);
  }

  .button-secondary:disabled,
  .button-danger:disabled {
    opacity: 0.55;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
  }

  @media (max-width: 960px) {
    .confirm-actions {
      grid-template-columns: 1fr;
      flex-direction: column;
      align-items: stretch;
    }

    .modal-head {
      align-items: stretch;
      flex-direction: column;
    }
  }

  @media (max-width: 460px) {
    .confirm-actions button {
      width: 100%;
    }
  }
</style>
