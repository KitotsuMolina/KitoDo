import type { TaskDTO } from '$lib/api/desktop';

export function mergeUniqueTasks(groups: TaskDTO[][]): TaskDTO[] {
  const map = new Map<string, TaskDTO>();
  for (const group of groups) {
    for (const task of group) {
      map.set(task.id, task);
    }
  }
  return Array.from(map.values());
}

export function filterTasks(base: TaskDTO[], labelFilter: string | null, query: string): TaskDTO[] {
  return base.filter((task) => {
    if (labelFilter && !task.labels.includes(labelFilter)) {
      return false;
    }

    if (!query) {
      return true;
    }

    const haystack = `${task.title} ${task.projectName ?? ''} ${task.labels.join(' ')}`.toLowerCase();
    return haystack.includes(query);
  });
}

export function formatRecurrenceLabel(rule: string | null): string {
  if (!rule) return '';
  const byRule: Record<string, string> = {
    daily: 'Cada día',
    weekly: 'Cada semana',
    monthly: 'Cada mes',
    'weekday:mon': 'Cada lunes',
    'weekday:tue': 'Cada martes',
    'weekday:wed': 'Cada miércoles',
    'weekday:thu': 'Cada jueves',
    'weekday:fri': 'Cada viernes',
    'weekday:sat': 'Cada sábado',
    'weekday:sun': 'Cada domingo'
  };

  if (byRule[rule]) {
    return byRule[rule];
  }

  if (rule.startsWith('interval:')) {
    const [, unit, amountRaw] = rule.split(':');
    const amount = Number(amountRaw);
    if (!Number.isFinite(amount) || amount <= 0) {
      return rule;
    }
    const singularMap: Record<string, string> = { d: 'día', w: 'semana', m: 'mes' };
    const pluralMap: Record<string, string> = { d: 'días', w: 'semanas', m: 'meses' };
    const singular = singularMap[unit] ?? 'unidad';
    const plural = pluralMap[unit] ?? 'unidades';
    return `Cada ${amount} ${amount === 1 ? singular : plural}`;
  }

  return rule;
}

export function priorityClass(priority: number): string {
  if (priority === 1) return 'p1';
  if (priority === 2) return 'p2';
  if (priority === 3) return 'p3';
  return 'p4';
}

export function escapeHtml(raw: string): string {
  return raw
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

export function highlightText(text: string, query: string): string {
  const clean = query.trim();
  if (!clean) return escapeHtml(text);
  const escapedQuery = clean.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const re = new RegExp(`(${escapedQuery})`, 'ig');
  return escapeHtml(text).replace(re, '<mark class="hit">$1</mark>');
}
