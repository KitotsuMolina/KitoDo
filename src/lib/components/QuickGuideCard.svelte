<script lang="ts">
  import { CircleHelp, X } from 'lucide-svelte';
  import type { QuickAddExample } from '$lib/constants/app-content';

  export let examples: QuickAddExample[] = [];
  export let onHide: () => void;
  export let onOpenHelp: () => void;
  export let onSelectExample: (value: string) => void;
</script>

<section class="guide-card">
  <div class="panel-head">
    <div class="guide-copy">
      <h2>Crear tareas en una línea</h2>
      <p>
        Usa la entrada rápida para combinar título, proyecto, etiquetas, prioridad, fecha y repetición.
        Los tokens `due` y `every` siguen estando en inglés porque son la sintaxis real del parser.
      </p>
    </div>
    <button class="icon-button" aria-label="Ocultar guía rápida" on:click={onHide}>
      <X size={18} strokeWidth={2.1} />
    </button>
  </div>
  <div class="guide-examples">
    {#each examples as example}
      <button class="example-pill" on:click={() => onSelectExample(example.value)}>
        <strong>{example.label}</strong>
        <span>{example.value}</span>
      </button>
    {/each}
  </div>
  <div class="guide-actions">
    <span>Filtros: botón `Filtros` o `Shift + F`. Cierre: mismo botón o `Escape`.</span>
    <button class="ghost-trigger compact button-with-icon" on:click={onOpenHelp}>
      <CircleHelp size={16} strokeWidth={2.1} />
      <span>Ver guía rápida</span>
    </button>
  </div>
</section>

<style>
  .panel-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  .guide-card {
    display: grid;
    gap: 12px;
    padding: 12px;
    border-radius: 14px;
    border: 1px solid rgba(192, 75, 255, 0.2);
    background:
      linear-gradient(135deg, rgba(192, 75, 255, 0.08), rgba(137, 198, 255, 0.05)),
      rgba(255, 255, 255, 0.03);
  }

  .guide-copy h2 {
    margin: 0;
    font-size: 0.95rem;
  }

  .guide-copy p {
    margin: 6px 0 0;
    color: var(--k-muted);
    font-size: 0.86rem;
    line-height: 1.45;
  }

  .guide-examples {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
  }

  .example-pill {
    display: grid;
    gap: 4px;
    text-align: left;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: var(--k-text);
    padding: 10px;
  }

  .example-pill strong {
    font-size: 0.8rem;
  }

  .example-pill span {
    color: var(--k-muted);
    font-size: 0.8rem;
    overflow-wrap: anywhere;
  }

  .guide-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-wrap: wrap;
    color: var(--k-muted);
    font-size: 0.82rem;
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
    .guide-examples {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 960px) {
    .guide-actions {
      justify-content: stretch;
      flex-direction: column;
      align-items: stretch;
    }
  }

  @media (max-width: 460px) {
    .example-pill {
      width: 100%;
    }
  }
</style>
