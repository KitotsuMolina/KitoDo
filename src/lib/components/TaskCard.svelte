<script lang="ts">
  import { cubicOut } from 'svelte/easing';
  import { fly } from 'svelte/transition';
  import type { TaskDTO } from '$lib/api/desktop';

  export let task: TaskDTO;
  export let searchQuery = '';
  export let editingId: string | null = null;
  export let editingTitle = '';
  export let menuTaskId: string | null = null;
  export let keyboardSelectedTaskId: string | null = null;
  export let extraClasses = '';
  export let draggable = false;
  export let highlightText: (text: string, query: string) => string;
  export let formatRecurrenceLabel: (rule: string | null) => string;
  export let onToggleTask: (task: TaskDTO) => void;
  export let onStartEditing: (task: TaskDTO) => void;
  export let onSaveEdit: (taskId: string) => void;
  export let onCancelEdit: () => void;
  export let onOpenMenu: (task: TaskDTO) => void;
  export let onDragStart: ((taskId: string) => void) | null = null;
  export let onDragOver: ((event: DragEvent) => void) | null = null;
  export let onDrop: ((taskId: string) => void) | null = null;

  $: isEditing = editingId === task.id;
  $: className =
    `task ${task.status === 'done' ? 'done' : ''} ${extraClasses} ${(menuTaskId === task.id || keyboardSelectedTaskId === task.id) ? 'selected' : ''}`.trim();
</script>

<article
  class={className}
  transition:fly={{ y: 8, duration: 180, easing: cubicOut }}
  {draggable}
  on:dragstart={() => onDragStart?.(task.id)}
  on:dragover={(event) => onDragOver?.(event)}
  on:drop={() => onDrop?.(task.id)}
>
  <label class="check">
    <input type="checkbox" checked={task.status === 'done'} on:change={() => onToggleTask(task)} />
  </label>

  <div class="task-body">
    {#if isEditing}
      <input
        class="title-edit"
        bind:value={editingTitle}
        on:blur={() => onSaveEdit(task.id)}
        on:keydown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            onSaveEdit(task.id);
          }
          if (e.key === 'Escape') {
            e.preventDefault();
            onCancelEdit();
          }
        }}
      />
    {:else}
      <button class="title" on:dblclick={() => onStartEditing(task)}>{@html highlightText(task.title, searchQuery)}</button>
    {/if}

    <div class="meta-line">
      {#if task.projectName}
        <span class="chip project" class:hit-chip={searchQuery.length > 0 && (task.projectName ?? '').toLowerCase().includes(searchQuery)}>
          @{task.projectName}
        </span>
      {/if}

      {#each task.labels as label}
        <span class="chip label" class:hit-chip={searchQuery.length > 0 && label.toLowerCase().includes(searchQuery)}>#{label}</span>
      {/each}

      {#if task.dueDate}
        <span class="chip due">Vence: {task.dueDate}</span>
      {/if}

      {#if task.recurrence}
        <span class="chip recurrence">{formatRecurrenceLabel(task.recurrence)}</span>
      {/if}
    </div>
  </div>

  <span class="priority-badge">P{task.priority}</span>

  <div class="menu-wrap">
    <button class="more" on:click={() => onOpenMenu(task)}>⋯</button>
  </div>
</article>

<style>
  .task {
    position: relative;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    width: 100%;
    min-width: 0;
    align-items: center;
    gap: 10px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-left: 3px solid rgba(255, 255, 255, 0.12);
    border-radius: 12px;
    padding: 10px 52px 10px 10px;
    transition: opacity 180ms ease, transform 180ms ease, box-shadow 180ms ease;
  }

  .task.overdue {
    border-left-color: rgba(255, 97, 135, 0.7);
  }

  .task.p1 {
    border-left-color: #ff6f96;
    box-shadow: 0 0 18px rgba(255, 111, 150, 0.18);
  }

  .task.p2 {
    border-left-color: #ffae62;
    box-shadow: 0 0 12px rgba(255, 174, 98, 0.15);
  }

  .task.p3 {
    border-left-color: #89c6ff;
    box-shadow: 0 0 8px rgba(137, 198, 255, 0.13);
  }

  .task.done {
    opacity: 0.62;
  }

  .task.selected {
    border-color: rgba(192, 75, 255, 0.58);
    box-shadow: 0 0 0 1px rgba(192, 75, 255, 0.25);
  }

  .check input {
    width: 16px;
    height: 16px;
    accent-color: var(--k-purple-2);
    transition: transform 150ms ease;
  }

  .check input:checked {
    transform: scale(1.08);
  }

  .task-body {
    min-width: 0;
    overflow: hidden;
  }

  .title {
    border: none;
    background: transparent;
    text-align: left;
    width: 100%;
    padding: 2px;
    min-height: 28px;
    color: var(--k-text);
    transition: color 180ms ease;
    border-radius: 10px;
    cursor: pointer;
  }

  .task.done .title {
    text-decoration: line-through;
    color: var(--k-muted);
  }

  .meta-line {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-top: 3px;
    font-size: 0.74rem;
  }

  .chip {
    border-radius: 999px;
    padding: 3px 7px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: var(--k-muted);
    background: rgba(255, 255, 255, 0.03);
  }

  .chip.project {
    border-color: rgba(192, 75, 255, 0.4);
    color: #dcb4ff;
  }

  .chip.label {
    border-color: rgba(137, 198, 255, 0.35);
    color: #b9dfff;
  }

  .chip.due {
    border-color: rgba(255, 174, 98, 0.35);
    color: #ffd7b0;
  }

  .chip.recurrence {
    border-color: rgba(155, 255, 184, 0.35);
    color: #bcffd0;
  }

  .chip.hit-chip {
    border-color: rgba(255, 222, 117, 0.6);
    box-shadow: 0 0 10px rgba(255, 222, 117, 0.18);
  }

  .title-edit {
    width: 100%;
    border-radius: 8px;
    border: 1px solid rgba(192, 75, 255, 0.7);
    background: rgba(20, 12, 26, 0.85);
    color: var(--k-text);
    padding: 6px 8px;
  }

  .priority-badge {
    border-radius: 999px;
    font-size: 0.75rem;
    padding: 4px 8px;
    border: 1px solid rgba(255, 255, 255, 0.24);
    color: var(--k-muted);
  }

  .menu-wrap {
    position: absolute;
    top: 10px;
    right: 10px;
    display: grid;
    justify-items: end;
    min-width: 32px;
  }

  .more {
    width: 32px;
    height: 32px;
    font-size: 1rem;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.04);
    color: var(--k-text);
    cursor: pointer;
  }

  @media (max-width: 1280px) {
    .task {
      grid-template-columns: auto minmax(0, 1fr);
      grid-template-areas:
        'check body'
        'check badge';
      padding-right: 52px;
    }

    .check {
      grid-area: check;
      align-self: start;
      margin-top: 4px;
    }

    .task-body {
      grid-area: body;
    }

    .title {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .meta-line {
      overflow: hidden;
    }

    .priority-badge {
      grid-area: badge;
      justify-self: start;
    }

    .menu-wrap {
      top: 8px;
      right: 8px;
    }
  }

  @media (max-width: 460px) {
    .task {
      padding: 9px;
      gap: 8px;
    }

    .title {
      font-size: 0.95rem;
    }

    .meta-line {
      gap: 5px;
    }

    .chip {
      max-width: 100%;
      overflow-wrap: anywhere;
    }
  }
</style>
