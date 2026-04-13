import type { TaskDTO } from '$lib/api/desktop';

export function isTextInputFocused(activeElement: HTMLElement | null): boolean {
  if (!activeElement) return false;
  return (
    activeElement.tagName === 'INPUT' ||
    activeElement.tagName === 'TEXTAREA' ||
    activeElement.getAttribute('contenteditable') === 'true' ||
    activeElement.closest('.order-dropdown') !== null ||
    activeElement.closest('.context-dropdown') !== null
  );
}

export function getNextKeyboardSelection(
  tasks: TaskDTO[],
  currentTaskId: string | null,
  direction: 1 | -1
): string | null {
  if (tasks.length === 0) return null;
  if (!currentTaskId) return tasks[0].id;

  const index = tasks.findIndex((task) => task.id === currentTaskId);
  if (index < 0) {
    return tasks[0].id;
  }

  const next = (index + direction + tasks.length) % tasks.length;
  return tasks[next].id;
}

export function shouldCloseFloatingUi(target: HTMLElement | null): boolean {
  return !target?.closest('.order-dropdown') && !target?.closest('.context-dropdown') && !target?.closest('.date-picker');
}
