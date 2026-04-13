<script lang="ts">
  import { ExternalLink, Trash2, X } from 'lucide-svelte';
  import type { ProjectDTO, TaskDTO } from '$lib/api/desktop';

  type RecurrenceOption = { value: string | null; label: string };
  type CalendarCell = { label: string; iso: string | null; selected: boolean; isToday: boolean };

  export let selectedTask: TaskDTO | null = null;
  export let variant: 'sidebar' | 'modal' = 'sidebar';
  export let selectedRecurrenceLabel = 'Sin repetición';
  export let selectedProjectLabel = 'Bandeja';
  export let recurrenceOptions: RecurrenceOption[] = [];
  export let projects: ProjectDTO[] = [];
  export let dueDateDraft = '';
  export let projectDraft = '';
  export let calendarMonthLabel = '';
  export let calendarCells: CalendarCell[] = [];
  export let recurrenceDropdownOpen = false;
  export let projectDropdownOpen = false;
  export let datePickerOpen = false;
  export let recurrenceMenuOpenUp = false;
  export let projectMenuOpenUp = false;
  export let datePickerOpenUp = false;
  export let datePickerRef: HTMLDivElement | null = null;
  export let datePickerMenuRef: HTMLDivElement | null = null;
  export let recurrenceDropdownRef: HTMLDivElement | null = null;
  export let recurrenceMenuRef: HTMLDivElement | null = null;
  export let projectDropdownRef: HTMLDivElement | null = null;
  export let projectMenuRef: HTMLDivElement | null = null;
  export let onClose: () => void;
  export let onUpdatePriority: (taskId: string, priority: number) => void;
  export let onPickDueShortcut: (taskId: string, kind: 'today' | 'tomorrow' | null) => void;
  export let onToggleDatePicker: () => void;
  export let onShiftCalendarMonth: (offset: number) => void;
  export let onApplyDateFromPicker: (taskId: string, isoDate: string) => void;
  export let onApplyManualDate: (taskId: string) => void;
  export let onToggleRecurrenceDropdown: () => void;
  export let onUpdateRecurrence: (taskId: string, recurrence: string | null) => void;
  export let onToggleProjectDropdown: () => void;
  export let onMoveProject: (taskId: string, projectId: string | null) => void;
  export let onCreateAndMoveProject: (taskId: string) => void;
  export let onRequestDeleteTask: (taskId: string) => void;
  export let onOpenExternalUrl: (url: string) => void;

  $: isModal = variant === 'modal';
</script>

{#if selectedTask}
  <div class:is-modal-root={isModal}>
    <div class="panel-head" class:task-modal-head={isModal}>
      {#if isModal}
        <div class="task-modal-title-block">
          <h4>Acciones</h4>
          <p class="context-title task-modal-title">{selectedTask.title}</p>
        </div>
      {:else}
        <h3>Acciones</h3>
      {/if}
      <button class="icon-button" aria-label="Cerrar panel de tarea" on:click={onClose}>
        <X size={18} strokeWidth={2.1} />
      </button>
    </div>

    {#if !isModal}
      <p class="context-title">{selectedTask.title}</p>
    {/if}

    <div class="menu-group">
      <p>Prioridad</p>
      <div class="menu-inline" class:task-modal-actions={isModal}>
        {#each [1, 2, 3, 4] as level}
          <button class:button-secondary={isModal} class:compact-action={isModal} on:click={() => onUpdatePriority(selectedTask.id, level)}>
            P{level}
          </button>
        {/each}
      </div>
    </div>

    <div class="menu-group">
      <p>Fecha</p>
      <div class="menu-inline" class:task-modal-actions={isModal}>
        <button class:button-secondary={isModal} class:compact-action={isModal} on:click={() => onPickDueShortcut(selectedTask.id, 'today')}>Hoy</button>
        <button class:button-secondary={isModal} class:compact-action={isModal} on:click={() => onPickDueShortcut(selectedTask.id, 'tomorrow')}>Mañana</button>
        <button class:button-secondary={isModal} class:compact-action={isModal} on:click={() => onPickDueShortcut(selectedTask.id, null)}>Quitar</button>
      </div>
      <div class="date-picker" bind:this={datePickerRef}>
        <button class="context-trigger date-trigger" class:open={datePickerOpen} on:click={onToggleDatePicker}>
          {dueDateDraft || 'Seleccionar fecha'}
          <span class="order-chevron">▾</span>
        </button>
        {#if datePickerOpen}
          <div class="date-picker-menu" class:open-up={datePickerOpenUp} bind:this={datePickerMenuRef}>
            <div class="calendar-head">
              <button class="nav-btn" on:click={() => onShiftCalendarMonth(-1)} aria-label="Mes anterior">‹</button>
              <strong>{calendarMonthLabel}</strong>
              <button class="nav-btn" on:click={() => onShiftCalendarMonth(1)} aria-label="Mes siguiente">›</button>
            </div>
            <div class="calendar-weekdays">
              <span>Do</span><span>Lu</span><span>Ma</span><span>Mi</span><span>Ju</span><span>Vi</span><span>Sá</span>
            </div>
            <div class="calendar-grid">
              {#each calendarCells as cell}
                {#if cell.iso}
                  <button
                    class="day-btn"
                    class:selected={cell.selected}
                    class:today={cell.isToday}
                    on:click={() => onApplyDateFromPicker(selectedTask.id, cell.iso!)}
                  >
                    {cell.label}
                  </button>
                {:else}
                  <span class="day-empty"></span>
                {/if}
              {/each}
            </div>
            <div class="menu-inline date-manual" class:task-modal-actions={isModal}>
              <input class="date-field" class:task-modal-input={isModal} placeholder="YYYY-MM-DD" bind:value={dueDateDraft} />
              <button class:button-primary={isModal} class:compact-action={isModal} on:click={() => onApplyManualDate(selectedTask.id)}>Aplicar</button>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <div class="menu-group">
      <p>Recurrencia</p>
      <div class="context-dropdown" bind:this={recurrenceDropdownRef}>
        <button class="context-trigger" class:open={recurrenceDropdownOpen} on:click={onToggleRecurrenceDropdown}>
          {selectedRecurrenceLabel}
          <span class="order-chevron">▾</span>
        </button>
        {#if recurrenceDropdownOpen}
          <div class="context-menu" class:open-up={recurrenceMenuOpenUp} bind:this={recurrenceMenuRef}>
            {#each recurrenceOptions as option}
              <button class:active={(selectedTask.recurrence ?? null) === option.value} on:click={() => onUpdateRecurrence(selectedTask.id, option.value)}>
                {option.label}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <div class="menu-group">
      <p>Proyecto</p>
      <div class="context-dropdown" bind:this={projectDropdownRef}>
        <button class="context-trigger" class:open={projectDropdownOpen} on:click={onToggleProjectDropdown}>
          {selectedProjectLabel}
          <span class="order-chevron">▾</span>
        </button>
        {#if projectDropdownOpen}
          <div class="context-menu" class:open-up={projectMenuOpenUp} bind:this={projectMenuRef}>
            <button class:active={selectedTask.projectId === null} on:click={() => onMoveProject(selectedTask.id, null)}>Bandeja</button>
            {#each projects as project}
              <button class:active={selectedTask.projectId === project.id} on:click={() => onMoveProject(selectedTask.id, project.id)}>
                @{project.name}
              </button>
            {/each}
          </div>
        {/if}
      </div>
      <div class="menu-inline" class:task-modal-actions={isModal}>
        <input class:task-modal-input={isModal} placeholder="Crear proyecto..." bind:value={projectDraft} />
        <button class:button-primary={isModal} class:compact-action={isModal} on:click={() => onCreateAndMoveProject(selectedTask.id)}>Crear</button>
      </div>
    </div>

    {#if isModal}
      <div class="confirm-actions">
        <button class="button-danger button-with-icon" on:click={() => onRequestDeleteTask(selectedTask.id)}>
          <Trash2 size={16} strokeWidth={2.1} />
          <span>Eliminar tarea</span>
        </button>
        {#if selectedTask.externalUrl}
          <button class="button-secondary button-with-icon" on:click={() => onOpenExternalUrl(selectedTask.externalUrl!)}>
            <ExternalLink size={16} strokeWidth={2.1} />
            <span>Abrir en GitHub</span>
          </button>
        {/if}
        <button class="button-secondary button-with-icon" on:click={onClose}>
          <X size={16} strokeWidth={2.1} />
          <span>Cerrar</span>
        </button>
      </div>
    {:else}
      <button class="danger" on:click={() => onRequestDeleteTask(selectedTask.id)}>Eliminar</button>
      {#if selectedTask.externalUrl}
        <button on:click={() => onOpenExternalUrl(selectedTask.externalUrl!)}>Abrir en GitHub</button>
      {/if}
    {/if}
  </div>
{:else if !isModal}
  <div>
    <h3>Panel de tarea</h3>
    <p class="context-hint">
      Selecciona una tarea con `⋯` para editar prioridad, fecha, recurrencia o proyecto. Cuando abras una tarea, podrás cerrar este panel con `✕` o `Escape`.
    </p>
  </div>
{/if}

<style>
  .panel-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  .context-title {
    margin: 0;
    font-size: 0.9rem;
    color: var(--k-text);
  }

  .context-hint {
    margin: 0;
    color: var(--k-muted);
    font-size: 0.86rem;
  }

  .menu-group {
    display: grid;
    gap: 6px;
  }

  .menu-group p {
    margin: 0;
    color: var(--k-muted);
    font-size: 0.75rem;
    text-transform: uppercase;
  }

  .menu-inline {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
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

  .date-picker {
    position: relative;
  }

  .date-trigger {
    width: 100%;
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

  .context-dropdown {
    position: relative;
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

  .date-picker-menu {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    width: min(100%, 320px);
    min-width: min(250px, calc(100vw - 48px));
    padding: 8px;
    border-radius: 10px;
    border: 1px solid rgba(192, 75, 255, 0.5);
    background: rgba(19, 16, 28, 0.98);
    box-shadow: 0 14px 24px rgba(0, 0, 0, 0.35), 0 0 18px rgba(192, 75, 255, 0.16);
    z-index: 90;
    display: grid;
    gap: 8px;
  }

  .date-picker-menu.open-up {
    top: auto;
    bottom: calc(100% + 6px);
    transform-origin: bottom left;
  }

  .calendar-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .calendar-head strong {
    font-size: 0.85rem;
    color: #e8dbff;
    text-transform: capitalize;
  }

  .nav-btn {
    width: 30px;
    height: 28px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.05);
    color: var(--k-text);
    font-size: 1rem;
    line-height: 1;
    padding: 0;
  }

  .calendar-weekdays,
  .calendar-grid {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: 4px;
  }

  .calendar-weekdays span {
    text-align: center;
    font-size: 0.72rem;
    color: var(--k-muted);
  }

  .day-btn,
  .day-empty {
    height: 30px;
    border-radius: 7px;
  }

  .day-btn {
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.03);
    color: var(--k-text);
    font-size: 0.8rem;
    padding: 0;
  }

  .day-btn:hover {
    border-color: rgba(192, 75, 255, 0.55);
    background: rgba(192, 75, 255, 0.14);
  }

  .day-btn.today:not(.selected) {
    border-color: rgba(107, 214, 255, 0.72);
    background: linear-gradient(135deg, rgba(20, 74, 96, 0.42), rgba(40, 116, 150, 0.25));
    color: #d9f5ff;
    box-shadow: inset 0 0 0 1px rgba(107, 214, 255, 0.28);
  }

  .day-btn.today:not(.selected):hover {
    border-color: rgba(107, 214, 255, 0.9);
    background: linear-gradient(135deg, rgba(24, 92, 118, 0.54), rgba(44, 136, 177, 0.32));
  }

  .day-btn.selected {
    border-color: rgba(192, 75, 255, 0.88);
    background: linear-gradient(135deg, rgba(166, 12, 219, 0.45), rgba(192, 75, 255, 0.38));
    color: #fff;
  }

  .day-empty {
    border: 1px solid transparent;
  }

  .date-manual {
    margin-top: 2px;
  }

  .date-field,
  .menu-inline input,
  .button-secondary,
  .button-primary,
  .danger,
  .button-danger {
    border-radius: 8px;
  }

  .date-field,
  .menu-inline input {
    width: 100%;
    padding: 6px 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.05);
    color: var(--k-text);
  }

  .date-field {
    min-height: 34px;
    border-color: rgba(192, 75, 255, 0.46);
    background: linear-gradient(180deg, rgba(28, 18, 40, 0.92), rgba(18, 14, 28, 0.92));
    box-shadow: inset 0 0 0 1px rgba(166, 12, 219, 0.12);
  }

  .date-field:focus,
  .menu-inline input:focus {
    outline: none;
    border-color: rgba(192, 75, 255, 0.88);
    box-shadow: 0 0 0 2px rgba(166, 12, 219, 0.24), inset 0 0 0 1px rgba(166, 12, 219, 0.15);
  }

  .danger {
    border: 1px solid rgba(255, 120, 145, 0.5);
    background: rgba(255, 120, 145, 0.12);
    color: #ffc4d3;
    justify-self: end;
    padding: 6px 8px;
    cursor: pointer;
  }

  .button-primary,
  .button-secondary,
  .button-danger {
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

  .compact-action {
    padding: 7px 10px;
    min-height: 34px;
    font-size: 0.78rem;
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

  .task-modal-head {
    align-items: flex-start;
  }

  .task-modal-title-block {
    min-width: 0;
    display: grid;
    gap: 4px;
  }

  .task-modal-title {
    overflow-wrap: anywhere;
  }

  .is-modal-root .menu-group {
    gap: 8px;
    padding: 10px;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.03);
  }

  .is-modal-root .menu-group p {
    color: #d9c2ef;
  }

  .task-modal-actions {
    align-items: stretch;
  }

  .task-modal-actions button {
    min-height: 38px;
  }

  .task-modal-input {
    width: 100%;
    min-height: 38px;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: linear-gradient(180deg, rgba(28, 18, 40, 0.92), rgba(18, 14, 28, 0.92));
    color: var(--k-text);
    padding: 9px 11px;
    box-shadow: inset 0 0 0 1px rgba(166, 12, 219, 0.1);
  }

  .task-modal-input:focus {
    outline: none;
    border-color: rgba(192, 75, 255, 0.82);
    box-shadow: 0 0 0 2px rgba(166, 12, 219, 0.22), inset 0 0 0 1px rgba(166, 12, 219, 0.12);
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  @media (max-width: 960px) {
    .menu-inline > * {
      flex: 1 1 120px;
    }

    .date-picker-menu {
      left: 50%;
      transform: translateX(-50%);
      width: min(92vw, 320px);
      min-width: 0;
    }

    .date-picker-menu.open-up {
      transform: translateX(-50%);
    }

    .confirm-actions {
      grid-template-columns: 1fr;
      flex-direction: column;
      align-items: stretch;
    }

    .task-modal-head {
      align-items: flex-start;
      flex-direction: row;
    }
  }

  @media (max-width: 460px) {
    .confirm-actions button {
      width: 100%;
    }
  }
</style>
