export type CalendarCell = {
  label: string;
  iso: string | null;
  selected: boolean;
  isToday: boolean;
};

export function dueShortcut(kind: 'today' | 'tomorrow'): string {
  const date = new Date();
  if (kind === 'tomorrow') {
    date.setDate(date.getDate() + 1);
  }
  return date.toISOString().slice(0, 10);
}

export function formatYmd(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

export function parseYmd(value: string): Date | null {
  const normalized = value.trim();
  if (!/^\d{4}-\d{2}-\d{2}$/.test(normalized)) return null;
  const [yearStr, monthStr, dayStr] = normalized.split('-');
  const year = Number(yearStr);
  const month = Number(monthStr);
  const day = Number(dayStr);
  const date = new Date(year, month - 1, day);
  if (date.getFullYear() !== year || date.getMonth() !== month - 1 || date.getDate() !== day) {
    return null;
  }
  return date;
}

export function buildCalendarCells(baseDate: Date, selectedDate: string): CalendarCell[] {
  const startOfMonth = new Date(baseDate.getFullYear(), baseDate.getMonth(), 1);
  const firstWeekday = startOfMonth.getDay();
  const daysInMonth = new Date(baseDate.getFullYear(), baseDate.getMonth() + 1, 0).getDate();
  const todayIso = formatYmd(new Date());
  const cells: CalendarCell[] = [];

  for (let i = 0; i < firstWeekday; i += 1) {
    cells.push({ label: '', iso: null, selected: false, isToday: false });
  }

  for (let day = 1; day <= daysInMonth; day += 1) {
    const date = new Date(baseDate.getFullYear(), baseDate.getMonth(), day);
    const iso = formatYmd(date);
    cells.push({ label: String(day), iso, selected: iso === selectedDate, isToday: iso === todayIso });
  }

  while (cells.length % 7 !== 0) {
    cells.push({ label: '', iso: null, selected: false, isToday: false });
  }

  return cells;
}
