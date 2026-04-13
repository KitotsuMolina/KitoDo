<script lang="ts">
  import { cubicOut, quintInOut, quintOut } from 'svelte/easing';
  import { fade, fly } from 'svelte/transition';
  import { browser } from '$app/environment';
  import { onMount, tick } from 'svelte';
  import {
    CircleHelp,
    Database,
    Download,
    ExternalLink,
    Eye,
    Filter,
    Plus,
    RefreshCw,
    RotateCcw,
    Trash2,
    Unplug,
    Upload,
    X
  } from 'lucide-svelte';
  import {
    exportBackupJson,
    githubAddRepoSubscription,
    githubConnect,
    githubDisconnect,
    githubGetSettings,
    githubGetStatus,
    githubListAccounts,
    githubListRepoSubscriptions,
    githubSetSettings,
    githubSyncNow,
    githubToggleRepoSubscription,
    githubRemoveRepoSubscription,
    importBackupJson,
    initDb,
    listInbox,
    listLabels,
    listOverdue,
    listProjectTasks,
    listProjects,
    listToday,
    listUpcoming,
    moveTaskToProject,
    quickAdd,
    reorderProjectTasks,
    resetProjectManualOrder,
    restoreTask,
    setProjectSortMode,
    softDeleteTask,
    toggleTaskWithRecurrence,
    updateTaskDueDate,
    updateTaskPriority,
    updateTaskRecurrence,
    updateTaskTitle,
    type GithubAccountDTO,
    type GithubSettingsDTO,
    type GithubStatusDTO,
    type ImportResultDTO,
    type LabelDTO,
    type ProjectDTO,
    type RepoSubDTO,
    type TaskDTO
  } from '$lib/api/desktop';

  type Tab = 'inbox' | 'today' | 'upcoming';

  const tabNames: Record<Tab, string> = {
    inbox: 'Bandeja',
    today: 'Hoy',
    upcoming: 'Próximos'
  };

  const recurrenceOptions: Array<{ value: string | null; label: string }> = [
    { value: null, label: 'Sin repetición' },
    { value: 'daily', label: 'Cada día' },
    { value: 'weekly', label: 'Cada semana' },
    { value: 'monthly', label: 'Cada mes' },
    { value: 'weekday:mon', label: 'Lunes' },
    { value: 'weekday:tue', label: 'Martes' },
    { value: 'weekday:wed', label: 'Miércoles' },
    { value: 'weekday:thu', label: 'Jueves' },
    { value: 'weekday:fri', label: 'Viernes' },
    { value: 'weekday:sat', label: 'Sábado' },
    { value: 'weekday:sun', label: 'Domingo' }
  ];

  type UndoState = {
    taskId: string;
    title: string;
    timerId: ReturnType<typeof setTimeout>;
  };

  type ShortcutItem = {
    keys: string[];
    description: string;
  };

  type QuickAddExample = {
    label: string;
    value: string;
  };

  type GithubSetupItem = {
    title: string;
    description: string;
  };

  type GithubDocLink = {
    label: string;
    url: string;
  };

  let currentTab: Tab = 'inbox';
  let quickInput = '';
  let searchInput = '';
  let searchQuery = '';
  let showDone = false;
  let expandedMode = false;
  let sidebarClosing = false;
  let orderDropdownOpen = false;
  let recurrenceDropdownOpen = false;
  let projectDropdownOpen = false;
  let datePickerOpen = false;
  let orderMenuOpenUp = false;
  let recurrenceMenuOpenUp = false;
  let projectMenuOpenUp = false;
  let datePickerOpenUp = false;

  let tasksByTab: Record<Tab, TaskDTO[]> = {
    inbox: [],
    today: [],
    upcoming: []
  };
  let overdueTasks: TaskDTO[] = [];
  let projectTasks: TaskDTO[] = [];

  let projects: ProjectDTO[] = [];
  let labels: LabelDTO[] = [];
  let todayAll: TaskDTO[] = [];

  let loading = false;
  let error = '';

  let editingId: string | null = null;
  let editingTitle = '';
  let originalEditingTitle = '';
  let saveEditInFlight = false;

  let quickInputRef: HTMLInputElement | null = null;
  let searchInputRef: HTMLInputElement | null = null;
  let orderDropdownRef: HTMLDivElement | null = null;
  let recurrenceDropdownRef: HTMLDivElement | null = null;
  let projectDropdownRef: HTMLDivElement | null = null;
  let orderMenuRef: HTMLDivElement | null = null;
  let recurrenceMenuRef: HTMLDivElement | null = null;
  let projectMenuRef: HTMLDivElement | null = null;
  let datePickerRef: HTMLDivElement | null = null;
  let datePickerMenuRef: HTMLDivElement | null = null;
  let githubAccountDropdownRef: HTMLDivElement | null = null;
  let githubIntervalDropdownRef: HTMLDivElement | null = null;
  let githubProjectDropdownRef: HTMLDivElement | null = null;
  let githubAccountMenuRef: HTMLDivElement | null = null;
  let githubIntervalMenuRef: HTMLDivElement | null = null;
  let githubProjectMenuRef: HTMLDivElement | null = null;

  let menuTaskId: string | null = null;
  let projectDraft = '';
  let dueDateDraft = '';
  let calendarBaseDate = new Date();

  let selectedProjectId: string | null = null;
  let selectedLabel: string | null = null;

  let undoState: UndoState | null = null;
  let recurrenceToast = '';
  let recurrenceToastTimer: ReturnType<typeof setTimeout> | null = null;
  let confirmResetOpen = false;
  let keyboardSelectedTaskId: string | null = null;

  let dragTaskId: string | null = null;
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let githubAccounts: GithubAccountDTO[] = [];
  let githubSettings: GithubSettingsDTO | null = null;
  let githubRepoSubs: RepoSubDTO[] = [];
  let githubStatus: GithubStatusDTO | null = null;
  let selectedGithubAccountId: string | null = null;
  let githubTokenInput = '';
  let githubRepoInput = '';
  let githubBusy = false;
  let githubMessage = '';
  let githubModalOpen = false;
  let githubAccountDropdownOpen = false;
  let githubIntervalDropdownOpen = false;
  let githubProjectDropdownOpen = false;
  let githubAccountMenuOpenUp = false;
  let githubIntervalMenuOpenUp = false;
  let githubProjectMenuOpenUp = false;
  let backupModalOpen = false;
  let backupBusy = false;
  let backupJsonDraft = '';
  let backupNotice = '';
  let backupFileInputRef: HTMLInputElement | null = null;
  let helpModalOpen = false;
  let quickGuideVisible = true;
  let confirmDeleteOpen = false;
  let confirmDeleteTaskId: string | null = null;
  let confirmDeleteTaskTitle = '';
  let viewportWidth = 1440;

  const quickAddExamples: QuickAddExample[] = [
    { label: 'Proyecto y etiqueta', value: 'Preparar demo @Trabajo #frontend p2' },
    { label: 'Con fecha', value: 'Pagar dominio @Admin #finanzas due tomorrow' },
    { label: 'Recurrente', value: 'Plan semanal @Casa #rutina every week' }
  ];

  const githubSetupItems: GithubSetupItem[] = [
    {
      title: 'No necesitas instalar GitHub CLI',
      description: 'La integración usa la API de GitHub directamente desde KitoDo. Solo necesitas conexión a internet.'
    },
    {
      title: 'Necesitas una cuenta de GitHub',
      description: 'KitoDo importa PRs, issues asignadas y notificaciones usando tu identidad de GitHub.'
    },
    {
      title: 'Usa un personal access token classic',
      description: 'Las notificaciones de GitHub no funcionan con fine-grained tokens. Para repos privados, añade también acceso `repo`.'
    },
    {
      title: 'Activa un keyring en Linux',
      description: 'KitoDo guarda el token en el llavero del sistema. Si falla el guardado, revisa GNOME Keyring, KWallet o Secret Service.'
    },
    {
      title: 'Debes estar suscrito o participando en GitHub',
      description: 'Las notificaciones aparecen cuando GitHub ya te las envía: repos observados, conversaciones asignadas o menciones.'
    }
  ];

  const githubDocLinks: GithubDocLink[] = [
    { label: 'Crear token classic', url: 'https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens' },
    { label: 'API de notifications', url: 'https://docs.github.com/en/rest/activity/notifications' },
    { label: 'Configurar notificaciones', url: 'https://docs.github.com/github/managing-subscriptions-and-notifications-on-github/setting-up-notifications/configuring-notifications' }
  ];

  const shortcutSections: Array<{ title: string; items: ShortcutItem[] }> = [
    {
      title: 'Creación rápida',
      items: [
        { keys: ['@proyecto'], description: 'Asigna el proyecto de la tarea' },
        { keys: ['#tag'], description: 'Añade una o varias etiquetas' },
        { keys: ['p1', 'p4'], description: 'Define prioridad de 1 a 4' },
        { keys: ['due', 'today'], description: 'Fecha con `today`, `tomorrow` o `YYYY-MM-DD`' },
        { keys: ['every', 'week'], description: 'Repite con `day`, `week`, `month` o `mon..sun`' }
      ]
    },
    {
      title: 'Navegacion',
      items: [
        { keys: ['Ctrl', 'K'], description: 'Enfocar entrada rápida' },
        { keys: ['Ctrl', 'F'], description: 'Enfocar buscador' },
        { keys: ['/'], description: 'Enfocar buscador sin usar raton' },
        { keys: ['Ctrl', '1'], description: 'Ir a Bandeja' },
        { keys: ['Ctrl', '2'], description: 'Ir a Hoy' },
        { keys: ['Ctrl', '3'], description: 'Ir a Proximos' },
        { keys: ['Shift', 'F'], description: 'Mostrar u ocultar filtros' },
        { keys: ['F11'], description: 'Maximizar o restaurar ventana' }
      ]
    },
    {
      title: 'Tareas',
      items: [
        { keys: ['j'], description: 'Mover seleccion hacia abajo' },
        { keys: ['k'], description: 'Mover seleccion hacia arriba' },
        { keys: ['x'], description: 'Completar o reabrir tarea seleccionada' },
        { keys: ['Enter'], description: 'Editar titulo de la tarea seleccionada' },
        { keys: ['Delete'], description: 'Eliminar tarea seleccionada' },
        { keys: ['Ctrl', 'Enter'], description: 'Agregar tarea y limpiar la entrada rápida' }
      ]
    },
    {
      title: 'General',
      items: [
        { keys: ['?'], description: 'Abrir o cerrar esta ayuda' },
        { keys: ['Escape'], description: 'Cerrar modales, paneles, filtros o la ventana' }
      ]
    }
  ];

  async function toggleWindowMaximize() {
    if (!browser) return;
    await window.kitodo.window.toggleMaximize();
  }

  async function closeAppWindow() {
    if (!browser) return;
    await window.kitodo.window.close();
  }

  async function openExternalUrl(url: string) {
    if (!browser) return;
    await window.kitodo.shell.openExternal(url);
  }

  function persistQuickGuideVisibility() {
    if (!browser) return;
    window.localStorage.setItem('kitodo.quickGuideVisible', quickGuideVisible ? '1' : '0');
  }

  function hideQuickGuide() {
    quickGuideVisible = false;
    persistQuickGuideVisibility();
  }

  function showQuickGuide() {
    quickGuideVisible = true;
    persistQuickGuideVisibility();
  }

  function closeSidebar() {
    if (!expandedMode) return;
    sidebarClosing = true;
    expandedMode = false;
  }

  function toggleSidebar() {
    if (expandedMode) {
      closeSidebar();
      return;
    }

    expandedMode = true;
  }

  $: selectedProject = selectedProjectId ? projects.find((project) => project.id === selectedProjectId) ?? null : null;
  $: isProjectView = selectedProjectId !== null;
  $: projectSortMode = selectedProject?.sortMode ?? 'auto';
  $: taskPanelAsModal = viewportWidth < 1280;

  $: pendingCount = mergeUniqueTasks([tasksByTab.inbox, tasksByTab.today, tasksByTab.upcoming, overdueTasks]).filter(
    (task) => task.status === 'todo'
  ).length;
  $: todayPendingCount = todayAll.filter((task) => task.status === 'todo').length;
  $: todayDoneCount = todayAll.filter((task) => task.status === 'done').length;
  $: todayTotalCount = todayAll.length;
  $: dayProgress = todayTotalCount === 0 ? 0 : Math.round((todayDoneCount / todayTotalCount) * 100);

  $: filteredOverdue = filterTasks(overdueTasks, selectedLabel, searchQuery);
  $: filteredInbox = filterTasks(tasksByTab.inbox, selectedLabel, searchQuery);
  $: filteredToday = filterTasks(tasksByTab.today, selectedLabel, searchQuery);
  $: filteredUpcoming = filterTasks(tasksByTab.upcoming, selectedLabel, searchQuery);
  $: filteredProjectTasks = filterTasks(projectTasks, selectedLabel, searchQuery);

  $: inboxCount = filteredInbox.length;
  $: todayCount = filteredToday.length + filteredOverdue.length;
  $: upcomingCount = filteredUpcoming.length + filteredOverdue.length;

  $: tabCounts = {
    inbox: inboxCount,
    today: todayCount,
    upcoming: upcomingCount
  };

  $: selectedMenuTask = menuTaskId
    ? mergeUniqueTasks([tasksByTab.inbox, tasksByTab.today, tasksByTab.upcoming, overdueTasks, projectTasks]).find(
        (task) => task.id === menuTaskId
      ) ?? null
    : null;
  $: selectedRecurrenceLabel =
    recurrenceOptions.find((o) => o.value === (selectedMenuTask?.recurrence ?? null))?.label ?? 'Sin repetición';
  $: selectedProjectLabel = selectedMenuTask?.projectName ? `@${selectedMenuTask.projectName}` : 'Bandeja';
  $: keyboardTasks = currentKeyboardTasks();
  $: calendarMonthLabel = calendarBaseDate.toLocaleDateString('es-ES', { month: 'long', year: 'numeric' });
  $: calendarCells = buildCalendarCells(calendarBaseDate, dueDateDraft);
  $: if (keyboardTasks.length > 0 && !keyboardSelectedTaskId) {
    keyboardSelectedTaskId = keyboardTasks[0].id;
  }
  $: if (keyboardSelectedTaskId && !keyboardTasks.some((task) => task.id === keyboardSelectedTaskId)) {
    keyboardSelectedTaskId = keyboardTasks[0]?.id ?? null;
  }
  $: selectedGithubAccountLabel =
    githubAccounts.find((a) => a.accountId === selectedGithubAccountId)?.username ?? 'Seleccionar cuenta';
  $: selectedGithubAccount =
    githubAccounts.find((a) => a.accountId === selectedGithubAccountId) ?? null;
  $: selectedGithubIntervalLabel = githubSettings
    ? ({ 60: '1m', 300: '5m', 600: '10m', 1800: '30m' }[githubSettings.syncIntervalSec] ?? `${githubSettings.syncIntervalSec}s`)
    : '5m';
  $: selectedGithubProjectLabel = githubSettings?.defaultProjectId
    ? `@${projects.find((p) => p.id === (githubSettings?.defaultProjectId ?? ''))?.name ?? 'Proyecto'}`
    : 'Bandeja GitHub (auto)';
  $: githubTokenKindLabel =
    selectedGithubAccount?.tokenKind === 'classic'
      ? 'Token classic'
      : selectedGithubAccount?.tokenKind === 'fine_grained'
        ? 'Token fine-grained'
        : selectedGithubAccount?.tokenKind
          ? `Token ${selectedGithubAccount.tokenKind}`
          : 'Sin token detectado';
  $: githubNotificationsReady = selectedGithubAccount?.tokenKind === 'classic';

  function mergeUniqueTasks(groups: TaskDTO[][]): TaskDTO[] {
    const map = new Map<string, TaskDTO>();
    for (const group of groups) {
      for (const task of group) {
        map.set(task.id, task);
      }
    }
    return Array.from(map.values());
  }

  function filterTasks(base: TaskDTO[], labelFilter: string | null, query: string): TaskDTO[] {
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

  function formatRecurrenceLabel(rule: string | null): string {
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

  function currentKeyboardTasks(): TaskDTO[] {
    if (isProjectView) return filteredProjectTasks;
    if (currentTab === 'inbox') return filteredInbox;
    if (currentTab === 'today') return [...filteredOverdue, ...filteredToday];
    return [...filteredOverdue, ...filteredUpcoming];
  }

  function isTextInputFocused(): boolean {
    const el = document.activeElement as HTMLElement | null;
    if (!el) return false;
    return (
      el.tagName === 'INPUT' ||
      el.tagName === 'TEXTAREA' ||
      el.getAttribute('contenteditable') === 'true' ||
      el.closest('.order-dropdown') !== null ||
      el.closest('.context-dropdown') !== null
    );
  }

  function cycleKeyboardSelection(direction: 1 | -1) {
    if (keyboardTasks.length === 0) return;
    if (!keyboardSelectedTaskId) {
      keyboardSelectedTaskId = keyboardTasks[0].id;
      return;
    }
    const index = keyboardTasks.findIndex((task) => task.id === keyboardSelectedTaskId);
    const next = (index + direction + keyboardTasks.length) % keyboardTasks.length;
    keyboardSelectedTaskId = keyboardTasks[next].id;
  }

  function escapeHtml(raw: string): string {
    return raw
      .replaceAll('&', '&amp;')
      .replaceAll('<', '&lt;')
      .replaceAll('>', '&gt;')
      .replaceAll('"', '&quot;')
      .replaceAll("'", '&#39;');
  }

  function highlightText(text: string, query: string): string {
    const clean = query.trim();
    if (!clean) return escapeHtml(text);
    const escapedQuery = clean.replace(/[.*+?^${}()|[\\]\\\\]/g, '\\\\$&');
    const re = new RegExp(`(${escapedQuery})`, 'ig');
    return escapeHtml(text).replace(re, '<mark class=\"hit\">$1</mark>');
  }

  async function bootstrap() {
    loading = true;
    error = '';

    try {
      await initDb();
      await refreshAll();
      await refreshGithubState();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function refreshGithubState() {
    githubAccounts = await githubListAccounts();
    if (githubAccounts.length === 0) {
      selectedGithubAccountId = null;
      githubSettings = null;
      githubRepoSubs = [];
      githubStatus = null;
      return;
    }

    if (!selectedGithubAccountId || !githubAccounts.some((a) => a.accountId === selectedGithubAccountId)) {
      selectedGithubAccountId = githubAccounts[0].accountId;
    }

    if (!selectedGithubAccountId) return;
    const [settings, subs, status] = await Promise.all([
      githubGetSettings(selectedGithubAccountId),
      githubListRepoSubscriptions(selectedGithubAccountId),
      githubGetStatus(selectedGithubAccountId)
    ]);
    githubSettings = settings;
    githubRepoSubs = subs;
    githubStatus = status;
  }

  async function refreshAll() {
    const [inbox, todayVisible, upcoming, overdue, allProjects, allLabels, allToday] = await Promise.all([
      listInbox(showDone),
      listToday(showDone),
      listUpcoming(7, showDone),
      listOverdue(showDone),
      listProjects(),
      listLabels(),
      listToday(true)
    ]);

    tasksByTab = {
      inbox,
      today: todayVisible,
      upcoming
    };
    overdueTasks = overdue;
    projects = allProjects;
    labels = allLabels;
    todayAll = allToday;

    if (selectedProjectId) {
      projectTasks = await listProjectTasks(selectedProjectId, showDone);
    } else {
      projectTasks = [];
    }
  }

  async function refreshProjectTasks() {
    if (!selectedProjectId) {
      projectTasks = [];
      return;
    }
    projectTasks = await listProjectTasks(selectedProjectId, showDone);
  }

  async function onGithubConnect() {
    if (!githubTokenInput.trim()) return;
    githubBusy = true;
    githubMessage = '';
    try {
      const account = await githubConnect(githubTokenInput.trim());
      selectedGithubAccountId = account.accountId;
      githubTokenInput = '';
      await refreshGithubState();
      githubMessage = `Conectado como ${account.username}`;
    } catch (e) {
      error = String(e);
    } finally {
      githubBusy = false;
    }
  }

  async function onGithubDisconnect() {
    if (!selectedGithubAccountId) return;
    githubBusy = true;
    githubMessage = '';
    try {
      await githubDisconnect(selectedGithubAccountId);
      await refreshGithubState();
      githubMessage = 'Cuenta desconectada';
    } catch (e) {
      error = String(e);
    } finally {
      githubBusy = false;
    }
  }

  async function onGithubSyncNow() {
    if (!selectedGithubAccountId) return;
    githubBusy = true;
    githubMessage = '';
    try {
      const result = await githubSyncNow(selectedGithubAccountId);
      githubMessage = `Sincronización: +${result.createdTasks} nuevas, ${result.updatedTasks} actualizadas, ${result.closedTasks} cerradas`;
      await refreshAll();
      await refreshGithubState();
    } catch (e) {
      error = String(e);
    } finally {
      githubBusy = false;
    }
  }

  async function onGithubAddRepo() {
    if (!selectedGithubAccountId) return;
    const value = githubRepoInput.trim();
    if (!value.includes('/')) {
      error = 'Formato repo inválido. Usa owner/repo';
      return;
    }
    const [owner, repo] = value.split('/', 2);
    if (!owner || !repo) {
      error = 'Formato repo inválido. Usa owner/repo';
      return;
    }

    githubBusy = true;
    try {
      await githubAddRepoSubscription(selectedGithubAccountId, owner, repo);
      githubRepoInput = '';
      await refreshGithubState();
    } catch (e) {
      error = String(e);
    } finally {
      githubBusy = false;
    }
  }

  async function onGithubToggleRepo(sub: RepoSubDTO) {
    githubBusy = true;
    try {
      await githubToggleRepoSubscription(sub.id, !sub.enabled);
      await refreshGithubState();
    } catch (e) {
      error = String(e);
    } finally {
      githubBusy = false;
    }
  }

  async function onGithubRemoveRepo(sub: RepoSubDTO) {
    githubBusy = true;
    try {
      await githubRemoveRepoSubscription(sub.id);
      await refreshGithubState();
    } catch (e) {
      error = String(e);
    } finally {
      githubBusy = false;
    }
  }

  async function onGithubUpdateSettings(patch: {
    enabled?: boolean;
    syncIntervalSec?: number;
    importPrReviews?: boolean;
    importAssignedIssues?: boolean;
    importNotifications?: boolean;
    defaultProjectId?: string | null;
  }) {
    if (!selectedGithubAccountId) return;
    githubBusy = true;
    try {
      githubSettings = await githubSetSettings(selectedGithubAccountId, patch);
      await refreshGithubState();
    } catch (e) {
      error = String(e);
    } finally {
      githubBusy = false;
    }
  }

  function onSearchInput() {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    debounceTimer = setTimeout(() => {
      searchQuery = searchInput.trim().toLowerCase();
    }, 180);
  }

  async function addTask(clearInputAfter = false) {
    const value = quickInput.trim();
    if (!value) return;

    try {
      await quickAdd(value);
      if (clearInputAfter) {
        quickInput = '';
      }
      await refreshAll();
    } catch (e) {
      error = String(e);
    }
  }

  async function onToggleTask(task: TaskDTO) {
    try {
      const result = await toggleTaskWithRecurrence(task.id);
      if (result.spawnedTask?.dueDate) {
        recurrenceToast = `Recurrente: creada próxima ocurrencia (${result.spawnedTask.dueDate})`;
        if (recurrenceToastTimer) {
          clearTimeout(recurrenceToastTimer);
        }
        recurrenceToastTimer = setTimeout(() => {
          recurrenceToast = '';
        }, 3500);
      }
      await refreshAll();
    } catch (e) {
      error = String(e);
    }
  }

  function startEditing(task: TaskDTO) {
    editingId = task.id;
    editingTitle = task.title;
    originalEditingTitle = task.title;
  }

  async function saveEdit(taskId: string) {
    if (saveEditInFlight) return;

    const title = editingTitle.trim();
    if (!title) {
      editingId = null;
      await refreshAll();
      return;
    }

    saveEditInFlight = true;
    try {
      await updateTaskTitle(taskId, title);
      editingId = null;
      await refreshAll();
    } catch (e) {
      error = String(e);
      await refreshAll();
    } finally {
      saveEditInFlight = false;
    }
  }

  function cancelEdit() {
    editingTitle = originalEditingTitle;
    editingId = null;
  }

  function clearUndoState() {
    if (!undoState) return;
    clearTimeout(undoState.timerId);
    undoState = null;
  }

  function closeTaskPanel() {
    menuTaskId = null;
    recurrenceDropdownOpen = false;
    projectDropdownOpen = false;
    datePickerOpen = false;
    recurrenceMenuOpenUp = false;
    projectMenuOpenUp = false;
    datePickerOpenUp = false;
  }

  function requestDeleteTask(taskId: string) {
    const task = mergeUniqueTasks([tasksByTab.inbox, tasksByTab.today, tasksByTab.upcoming, overdueTasks, projectTasks]).find(
      (candidate) => candidate.id === taskId
    );

    confirmDeleteTaskId = taskId;
    confirmDeleteTaskTitle = task?.title ?? 'esta tarea';
    confirmDeleteOpen = true;
  }

  async function confirmDeleteTask() {
    if (!confirmDeleteTaskId) return;
    const taskId = confirmDeleteTaskId;
    confirmDeleteOpen = false;
    confirmDeleteTaskId = null;
    confirmDeleteTaskTitle = '';
    await onDeleteTask(taskId);
  }

  async function onDeleteTask(taskId: string) {
    const task = mergeUniqueTasks([tasksByTab.inbox, tasksByTab.today, tasksByTab.upcoming, overdueTasks, projectTasks]).find(
      (t) => t.id === taskId
    );

    try {
      await softDeleteTask(taskId);
      await refreshAll();
      menuTaskId = null;

      if (task) {
        clearUndoState();
        const timerId = setTimeout(() => {
          undoState = null;
        }, 5000);

        undoState = {
          taskId,
          title: task.title,
          timerId
        };
      }
    } catch (e) {
      error = String(e);
      await refreshAll();
    }
  }

  async function undoDelete() {
    if (!undoState) return;

    const { taskId } = undoState;
    clearUndoState();

    try {
      await restoreTask(taskId);
      await refreshAll();
    } catch (e) {
      error = String(e);
    }
  }

  async function switchTab(tab: Tab) {
    if (currentTab === tab) return;

    const keepQuickFocus = document.activeElement === quickInputRef;
    currentTab = tab;
    menuTaskId = null;

    if (keepQuickFocus) {
      quickInputRef?.focus();
    }
  }

  async function toggleShowDone() {
    showDone = !showDone;
    await refreshAll();
  }

  function dueShortcut(kind: 'today' | 'tomorrow'): string {
    const date = new Date();
    if (kind === 'tomorrow') {
      date.setDate(date.getDate() + 1);
    }
    return date.toISOString().slice(0, 10);
  }

  function formatYmd(date: Date): string {
    const y = date.getFullYear();
    const m = String(date.getMonth() + 1).padStart(2, '0');
    const d = String(date.getDate()).padStart(2, '0');
    return `${y}-${m}-${d}`;
  }

  function parseYmd(value: string): Date | null {
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

  function buildCalendarCells(baseDate: Date, selectedDate: string): Array<{ label: string; iso: string | null; selected: boolean; isToday: boolean }> {
    const startOfMonth = new Date(baseDate.getFullYear(), baseDate.getMonth(), 1);
    const firstWeekday = startOfMonth.getDay();
    const daysInMonth = new Date(baseDate.getFullYear(), baseDate.getMonth() + 1, 0).getDate();
    const todayIso = formatYmd(new Date());
    const cells: Array<{ label: string; iso: string | null; selected: boolean; isToday: boolean }> = [];

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

  async function toggleDatePicker() {
    datePickerOpen = !datePickerOpen;
    if (datePickerOpen) {
      const parsed = parseYmd(dueDateDraft);
      const seed = parsed ?? new Date();
      calendarBaseDate = new Date(seed.getFullYear(), seed.getMonth(), 1);
      await tick();
      datePickerOpenUp = shouldOpenMenuUp(datePickerRef, datePickerMenuRef);
    } else {
      datePickerOpenUp = false;
    }
  }

  function shiftCalendarMonth(offset: number) {
    calendarBaseDate = new Date(calendarBaseDate.getFullYear(), calendarBaseDate.getMonth() + offset, 1);
  }

  async function applyDateFromPicker(taskId: string, isoDate: string) {
    dueDateDraft = isoDate;
    error = '';
    datePickerOpen = false;
    datePickerOpenUp = false;
    await onUpdateDueDate(taskId, isoDate);
  }

  async function applyManualDate(taskId: string) {
    const parsed = parseYmd(dueDateDraft);
    if (!parsed) {
      error = 'Fecha inválida. Usa formato YYYY-MM-DD';
      return;
    }
    error = '';
    await applyDateFromPicker(taskId, formatYmd(parsed));
  }

  function shouldOpenMenuUp(dropdownEl: HTMLElement | null, menuEl: HTMLElement | null): boolean {
    if (!dropdownEl || !menuEl) {
      return false;
    }

    const rect = dropdownEl.getBoundingClientRect();
    const menuHeight = Math.min(menuEl.scrollHeight, 220);
    const gap = 12;
    const spaceBelow = window.innerHeight - rect.bottom - gap;
    const spaceAbove = rect.top - gap;

    return spaceBelow < menuHeight && spaceAbove > spaceBelow;
  }

  async function toggleOrderDropdown() {
    orderDropdownOpen = !orderDropdownOpen;
    if (orderDropdownOpen) {
      await tick();
      orderMenuOpenUp = shouldOpenMenuUp(orderDropdownRef, orderMenuRef);
    } else {
      orderMenuOpenUp = false;
    }
  }

  async function toggleRecurrenceDropdown() {
    recurrenceDropdownOpen = !recurrenceDropdownOpen;
    projectDropdownOpen = false;
    projectMenuOpenUp = false;
    if (recurrenceDropdownOpen) {
      await tick();
      recurrenceMenuOpenUp = shouldOpenMenuUp(recurrenceDropdownRef, recurrenceMenuRef);
    } else {
      recurrenceMenuOpenUp = false;
    }
  }

  async function toggleProjectDropdown() {
    projectDropdownOpen = !projectDropdownOpen;
    recurrenceDropdownOpen = false;
    recurrenceMenuOpenUp = false;
    if (projectDropdownOpen) {
      await tick();
      projectMenuOpenUp = shouldOpenMenuUp(projectDropdownRef, projectMenuRef);
    } else {
      projectMenuOpenUp = false;
    }
  }

  async function toggleGithubAccountDropdown() {
    githubAccountDropdownOpen = !githubAccountDropdownOpen;
    githubIntervalDropdownOpen = false;
    githubProjectDropdownOpen = false;
    githubIntervalMenuOpenUp = false;
    githubProjectMenuOpenUp = false;
    if (githubAccountDropdownOpen) {
      await tick();
      githubAccountMenuOpenUp = shouldOpenMenuUp(githubAccountDropdownRef, githubAccountMenuRef);
    } else {
      githubAccountMenuOpenUp = false;
    }
  }

  async function toggleGithubIntervalDropdown() {
    githubIntervalDropdownOpen = !githubIntervalDropdownOpen;
    githubAccountDropdownOpen = false;
    githubProjectDropdownOpen = false;
    githubAccountMenuOpenUp = false;
    githubProjectMenuOpenUp = false;
    if (githubIntervalDropdownOpen) {
      await tick();
      githubIntervalMenuOpenUp = shouldOpenMenuUp(githubIntervalDropdownRef, githubIntervalMenuRef);
    } else {
      githubIntervalMenuOpenUp = false;
    }
  }

  async function toggleGithubProjectDropdown() {
    githubProjectDropdownOpen = !githubProjectDropdownOpen;
    githubAccountDropdownOpen = false;
    githubIntervalDropdownOpen = false;
    githubAccountMenuOpenUp = false;
    githubIntervalMenuOpenUp = false;
    if (githubProjectDropdownOpen) {
      await tick();
      githubProjectMenuOpenUp = shouldOpenMenuUp(githubProjectDropdownRef, githubProjectMenuRef);
    } else {
      githubProjectMenuOpenUp = false;
    }
  }

  function openMenu(task: TaskDTO) {
    if (menuTaskId === task.id) {
      closeTaskPanel();
      return;
    }

    menuTaskId = task.id;
    dueDateDraft = task.dueDate ?? '';
    projectDraft = '';
    recurrenceDropdownOpen = false;
    projectDropdownOpen = false;
    datePickerOpen = false;
    recurrenceMenuOpenUp = false;
    projectMenuOpenUp = false;
    datePickerOpenUp = false;
  }

  async function onUpdatePriority(taskId: string, priority: number) {
    try {
      await updateTaskPriority(taskId, priority);
      await refreshAll();
    } catch (e) {
      error = String(e);
    }
  }

  async function onUpdateDueDate(taskId: string, dueDate: string | null) {
    try {
      await updateTaskDueDate(taskId, dueDate);
      await refreshAll();
    } catch (e) {
      error = String(e);
    }
  }

  async function onPickDueShortcut(taskId: string, kind: 'today' | 'tomorrow' | null) {
    const dueDate = kind ? dueShortcut(kind) : null;
    datePickerOpen = false;
    datePickerOpenUp = false;
    await onUpdateDueDate(taskId, dueDate);
  }

  async function onUpdateRecurrence(taskId: string, recurrence: string | null) {
    try {
      await updateTaskRecurrence(taskId, recurrence);
      recurrenceDropdownOpen = false;
      recurrenceMenuOpenUp = false;
      await refreshAll();
    } catch (e) {
      error = String(e);
    }
  }

  async function onMoveProject(taskId: string, projectId: string | null) {
    try {
      await moveTaskToProject({ id: taskId, projectId });
      projectDropdownOpen = false;
      projectMenuOpenUp = false;
      await refreshAll();
    } catch (e) {
      error = String(e);
    }
  }

  async function onCreateAndMoveProject(taskId: string) {
    const name = projectDraft.trim();
    if (!name) return;

    try {
      await moveTaskToProject({ id: taskId, projectNameToCreate: name });
      projectDraft = '';
      await refreshAll();
    } catch (e) {
      error = String(e);
    }
  }

  async function onSelectProject(projectId: string | null) {
    selectedProjectId = projectId;
    closeTaskPanel();
    await refreshProjectTasks();
  }

  async function onSetProjectSortMode(mode: 'auto' | 'manual') {
    if (!selectedProjectId) return;
    try {
      const updated = await setProjectSortMode(selectedProjectId, mode);
      projects = projects.map((project) => (project.id === updated.id ? updated : project));
      orderDropdownOpen = false;
      orderMenuOpenUp = false;
      await refreshProjectTasks();
    } catch (e) {
      error = String(e);
    }
  }

  function onResetProjectToAuto() {
    confirmResetOpen = true;
  }

  async function confirmResetProjectToAuto() {
    if (!selectedProjectId) return;
    try {
      await resetProjectManualOrder(selectedProjectId);
      const updated = await setProjectSortMode(selectedProjectId, 'auto');
      projects = projects.map((project) => (project.id === updated.id ? updated : project));
      confirmResetOpen = false;
      await refreshProjectTasks();
    } catch (e) {
      error = String(e);
    }
  }

  async function onRecalculateProjectOrder() {
    if (!selectedProjectId) return;

    try {
      if (projectSortMode === 'manual') {
        await reorderProjectTasks(
          selectedProjectId,
          projectTasks
            .filter((task) => task.projectId === selectedProjectId)
            .map((task) => task.id)
        );
      }
      await refreshProjectTasks();
    } catch (e) {
      error = String(e);
    }
  }

  function onDragStart(taskId: string) {
    dragTaskId = taskId;
  }

  async function onDropOnTask(targetTaskId: string) {
    if (!dragTaskId || !selectedProjectId || dragTaskId === targetTaskId) {
      dragTaskId = null;
      return;
    }

    const from = projectTasks.findIndex((task) => task.id === dragTaskId);
    const to = projectTasks.findIndex((task) => task.id === targetTaskId);

    if (from < 0 || to < 0) {
      dragTaskId = null;
      return;
    }

    const next = [...projectTasks];
    const [moved] = next.splice(from, 1);
    next.splice(to, 0, moved);
    projectTasks = next;
    dragTaskId = null;

    try {
      await reorderProjectTasks(
        selectedProjectId,
        next.filter((task) => task.projectId === selectedProjectId).map((task) => task.id)
      );
      await refreshProjectTasks();
    } catch (e) {
      error = String(e);
      await refreshProjectTasks();
    }
  }

  function clearFilters() {
    selectedLabel = null;
    searchInput = '';
    searchQuery = '';
  }

  function focusQuickAdd() {
    quickInputRef?.focus();
  }

  function focusSearch() {
    searchInputRef?.focus();
    searchInputRef?.select();
  }

  function planTomorrowFromQuickAdd() {
    const suffix = ' due tomorrow';
    if (!quickInput.includes('due ')) {
      quickInput = `${quickInput.trim()}${quickInput.trim() ? '' : 'Nueva tarea'}${suffix}`;
    }
    quickInputRef?.focus();
  }

  async function onExportBackup() {
    backupBusy = true;
    backupNotice = '';
    try {
      const json = await exportBackupJson();
      backupJsonDraft = json;

      const stamp = new Date().toISOString().slice(0, 19).replaceAll(':', '-');
      const fileName = `kitodo-backup-${stamp}.json`;
      const blob = new Blob([json], { type: 'application/json;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = fileName;
      document.body.appendChild(link);
      link.click();
      link.remove();
      URL.revokeObjectURL(url);
      backupNotice = 'Respaldo exportado.';
    } catch (e) {
      error = String(e);
    } finally {
      backupBusy = false;
    }
  }

  function triggerImportBackupFile() {
    backupFileInputRef?.click();
  }

  async function runImportBackup(json: string) {
    const payload = json.trim();
    if (!payload) return;

    backupBusy = true;
    backupNotice = '';
    try {
      const result: ImportResultDTO = await importBackupJson(payload);
      backupNotice = `Importación lista: ${result.importedProjects} proyectos, ${result.createdTasks} tareas nuevas, ${result.updatedTasks} actualizadas.`;
      await refreshAll();
      await refreshGithubState();
    } catch (e) {
      error = String(e);
    } finally {
      backupBusy = false;
    }
  }

  async function onImportBackupFile(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    const text = await file.text();
    input.value = '';
    await runImportBackup(text);
  }

  async function onImportBackupDraft() {
    await runImportBackup(backupJsonDraft);
  }

  function priorityClass(priority: number): string {
    if (priority === 1) return 'p1';
    if (priority === 2) return 'p2';
    if (priority === 3) return 'p3';
    return 'p4';
  }

  onMount(() => {
    if (browser) {
      quickGuideVisible = window.localStorage.getItem('kitodo.quickGuideVisible') !== '0';
      viewportWidth = window.innerWidth;
    }

    bootstrap();
    quickInputRef?.focus();

    const onResize = () => {
      viewportWidth = window.innerWidth;
    };

    const onKeyDown = async (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        quickInputRef?.focus();
        quickInputRef?.select();
        return;
      }

      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'f') {
        event.preventDefault();
        focusSearch();
        return;
      }

      if ((event.ctrlKey || event.metaKey) && event.key === '1') {
        event.preventDefault();
        await switchTab('inbox');
        return;
      }

      if ((event.ctrlKey || event.metaKey) && event.key === '2') {
        event.preventDefault();
        await switchTab('today');
        return;
      }

      if ((event.ctrlKey || event.metaKey) && event.key === '3') {
        event.preventDefault();
        await switchTab('upcoming');
        return;
      }

      if (!event.ctrlKey && !event.metaKey && !isTextInputFocused()) {
        const key = event.key.toLowerCase();
        if (event.key === '/') {
          event.preventDefault();
          focusSearch();
          return;
        }
        if (event.key === '?') {
          event.preventDefault();
          helpModalOpen = !helpModalOpen;
          return;
        }
        if (key === 'j') {
          event.preventDefault();
          cycleKeyboardSelection(1);
          return;
        }
        if (key === 'k') {
          event.preventDefault();
          cycleKeyboardSelection(-1);
          return;
        }
        if (key === 'x' && keyboardSelectedTaskId) {
          event.preventDefault();
          const task = keyboardTasks.find((t) => t.id === keyboardSelectedTaskId);
          if (task) {
            await onToggleTask(task);
          }
          return;
        }
        if (event.key === 'Enter' && keyboardSelectedTaskId) {
          event.preventDefault();
          const task = keyboardTasks.find((t) => t.id === keyboardSelectedTaskId);
          if (task) {
            startEditing(task);
          }
          return;
        }
        if ((event.key === 'Delete' || event.key === 'Backspace') && keyboardSelectedTaskId) {
          event.preventDefault();
          requestDeleteTask(keyboardSelectedTaskId);
          return;
        }
      }

      if (event.shiftKey && event.key.toLowerCase() === 'f' && !event.ctrlKey && !event.metaKey) {
        event.preventDefault();
        toggleSidebar();
        return;
      }

      if (event.key === 'F11') {
        event.preventDefault();
        await toggleWindowMaximize();
        return;
      }

      if (event.key === 'Escape') {
        if (confirmDeleteOpen) {
          event.preventDefault();
          confirmDeleteOpen = false;
          confirmDeleteTaskId = null;
          confirmDeleteTaskTitle = '';
          return;
        }

        if (githubModalOpen) {
          event.preventDefault();
          githubModalOpen = false;
          return;
        }

        if (helpModalOpen) {
          event.preventDefault();
          helpModalOpen = false;
          return;
        }

        if (backupModalOpen) {
          event.preventDefault();
          backupModalOpen = false;
          return;
        }

        if (confirmResetOpen) {
          event.preventDefault();
          confirmResetOpen = false;
          return;
        }

        if (
          orderDropdownOpen ||
          recurrenceDropdownOpen ||
          projectDropdownOpen ||
          datePickerOpen ||
          githubAccountDropdownOpen ||
          githubIntervalDropdownOpen ||
          githubProjectDropdownOpen
        ) {
          event.preventDefault();
          orderDropdownOpen = false;
          recurrenceDropdownOpen = false;
          projectDropdownOpen = false;
          datePickerOpen = false;
          githubAccountDropdownOpen = false;
          githubIntervalDropdownOpen = false;
          githubProjectDropdownOpen = false;
          githubAccountMenuOpenUp = false;
          githubIntervalMenuOpenUp = false;
          githubProjectMenuOpenUp = false;
          orderMenuOpenUp = false;
          recurrenceMenuOpenUp = false;
          projectMenuOpenUp = false;
          datePickerOpenUp = false;
          return;
        }

        if (document.activeElement === searchInputRef || searchQuery.length > 0 || searchInput.length > 0) {
          event.preventDefault();
          searchInput = '';
          searchQuery = '';
          quickInputRef?.focus();
          return;
        }

        if (editingId) {
          event.preventDefault();
          cancelEdit();
          return;
        }

        if (menuTaskId) {
          event.preventDefault();
          closeTaskPanel();
          return;
        }

        if (expandedMode) {
          event.preventDefault();
          closeSidebar();
          return;
        }

        event.preventDefault();
        await closeAppWindow();
      }
    };

    const onPointerDown = (event: PointerEvent) => {
      const target = event.target as HTMLElement | null;
      if (!target?.closest('.order-dropdown') && !target?.closest('.context-dropdown') && !target?.closest('.date-picker')) {
        orderDropdownOpen = false;
        recurrenceDropdownOpen = false;
        projectDropdownOpen = false;
        datePickerOpen = false;
        githubAccountDropdownOpen = false;
        githubIntervalDropdownOpen = false;
        githubProjectDropdownOpen = false;
        githubAccountMenuOpenUp = false;
        githubIntervalMenuOpenUp = false;
        githubProjectMenuOpenUp = false;
        orderMenuOpenUp = false;
        recurrenceMenuOpenUp = false;
        projectMenuOpenUp = false;
        datePickerOpenUp = false;
      }
    };

    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('pointerdown', onPointerDown);
    window.addEventListener('resize', onResize);

    return () => {
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('pointerdown', onPointerDown);
      window.removeEventListener('resize', onResize);
      clearUndoState();
      if (recurrenceToastTimer) {
        clearTimeout(recurrenceToastTimer);
      }
    };
  });
</script>

<main class="shell">
  <section class="panel" class:expanded={expandedMode || sidebarClosing}>
    {#if expandedMode}
      <aside
        class="sidebar"
        in:fly={{ x: -42, duration: 700, easing: quintOut }}
        out:fly={{ x: -34, duration: 640, easing: quintInOut }}
        on:outroend={() => {
          sidebarClosing = false;
        }}
      >
        <div class="sidebar-head">
          <div>
            <h2>Filtros</h2>
            <p>Abre o cierra este panel con el botón `Filtros`, `Shift + F` o `Escape`.</p>
          </div>
          <button class="icon-button" aria-label="Cerrar filtros" on:click={closeSidebar}>
            <X size={18} strokeWidth={2.1} />
          </button>
        </div>

        <div class="side-block">
          <h3>Vistas</h3>
          {#each (Object.keys(tabNames) as Tab[]) as tab}
            <button class:active={currentTab === tab} on:click={() => switchTab(tab)}>
              {tabNames[tab]} ({tabCounts[tab]})
            </button>
          {/each}
        </div>

        <div class="side-block">
          <h3>Proyectos</h3>
          <button class:active={selectedProjectId === null} on:click={() => onSelectProject(null)}>Todos</button>
          {#each projects as project}
            <button class:active={selectedProjectId === project.id} on:click={() => onSelectProject(project.id)}>
              @{project.name}
            </button>
          {/each}
        </div>

        <div class="side-block">
          <h3>Etiquetas</h3>
          <button class:active={selectedLabel === null} on:click={() => (selectedLabel = null)}>Todas</button>
          {#each labels as label}
            <button class:active={selectedLabel === label.name} on:click={() => (selectedLabel = label.name)}>
              #{label.name}
            </button>
          {/each}
        </div>

        <div class="side-block github-block">
          <h3>GitHub</h3>
          {#if githubAccounts.length === 0}
            <div class="sidebar-badges">
              <span class="status-badge">Sin cuenta conectada</span>
              <span class="status-badge">Abre el panel para configurar GitHub</span>
            </div>
          {:else}
            <div class="sidebar-badges">
              <span class="status-badge ok">{selectedGithubAccountLabel}</span>
              <span class="status-badge" class:ok={githubSettings?.enabled}>
                {githubSettings?.enabled ? 'Sync activa' : 'Sync desactivada'}
              </span>
              <span class="status-badge">
                {githubStatus?.lastSyncAt ? `Última sync: ${githubStatus.lastSyncAt}` : 'Sin sincronización'}
              </span>
            </div>
          {/if}
        </div>

        <button class="clear-filters button-with-icon" on:click={() => (githubModalOpen = true)}>
          <ExternalLink size={16} strokeWidth={2.1} />
          <span>Mostrar panel de GitHub</span>
        </button>
        <button class="clear-filters button-with-icon" on:click={clearFilters}>
          <RotateCcw size={16} strokeWidth={2.1} />
          <span>Limpiar filtros</span>
        </button>
      </aside>
    {/if}

    <section class="content">
      <header class="header">
        <div class="header-left">
          <h1>KitoDo</h1>
          <p>Pendientes: {pendingCount} | Hoy: {todayPendingCount}</p>
          <label class="toggle top-left">
            <input type="checkbox" checked={showDone} on:change={toggleShowDone} />
            <span class="toggle-switch" aria-hidden="true">
              <span class="toggle-thumb"></span>
            </span>
            <span>Mostrar completadas</span>
          </label>
        </div>
        <div class="header-actions">
          <div class="header-tool-buttons">
            <button
              class="header-tool-button"
              aria-label={expandedMode ? 'Ocultar filtros' : 'Mostrar filtros'}
              title={expandedMode ? 'Ocultar filtros' : 'Mostrar filtros'}
              on:click={toggleSidebar}
            >
              <Filter size={18} strokeWidth={2.1} />
            </button>
            <button
              class="header-tool-button"
              aria-label="Abrir ayuda"
              title="Abrir ayuda"
              on:click={() => (helpModalOpen = true)}
            >
              <CircleHelp size={18} strokeWidth={2.1} />
            </button>
            <button
              class="header-tool-button header-tool-button--accent"
              aria-label="Abrir respaldo"
              title="Abrir respaldo"
              on:click={() => {
                backupModalOpen = true;
                backupNotice = '';
              }}
            >
              <Database size={18} strokeWidth={2.1} />
            </button>
            <button
              class="header-tool-button header-tool-button--danger"
              aria-label="Cerrar aplicación"
              title="Cerrar aplicación"
              on:click={closeAppWindow}
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

      <div class="quick-add">
        <input
          bind:this={quickInputRef}
          bind:value={quickInput}
          placeholder="Escribe una tarea... @Proyecto #tag p2 due tomorrow every week"
          on:keydown={(e) => {
            if (e.key === 'Enter' && e.ctrlKey) {
              e.preventDefault();
              addTask(true);
              return;
            }

            if (e.key === 'Enter') {
              e.preventDefault();
              addTask(false);
            }
          }}
        />
        <button on:click={() => addTask(true)}>Agregar</button>
      </div>

      {#if quickGuideVisible}
        <section class="guide-card">
          <div class="panel-head">
            <div class="guide-copy">
              <h2>Crear tareas en una línea</h2>
              <p>
                Usa la entrada rápida para combinar título, proyecto, etiquetas, prioridad, fecha y repetición.
                Los tokens `due` y `every` siguen estando en inglés porque son la sintaxis real del parser.
              </p>
            </div>
            <button class="icon-button" aria-label="Ocultar guía rápida" on:click={hideQuickGuide}>
              <X size={18} strokeWidth={2.1} />
            </button>
          </div>
          <div class="guide-examples">
            {#each quickAddExamples as example}
              <button
                class="example-pill"
                on:click={() => {
                  quickInput = example.value;
                  quickInputRef?.focus();
                  quickInputRef?.select();
                }}
              >
                <strong>{example.label}</strong>
                <span>{example.value}</span>
              </button>
            {/each}
          </div>
          <div class="guide-actions">
            <span>Filtros: botón `Filtros` o `Shift + F`. Cierre: mismo botón o `Escape`.</span>
            <button class="ghost-trigger compact button-with-icon" on:click={() => (helpModalOpen = true)}>
              <CircleHelp size={16} strokeWidth={2.1} />
              <span>Ver guía rápida</span>
            </button>
          </div>
        </section>
      {/if}

      <div class="toolbar">
        <div class="tabs">
          <button class:active={currentTab === 'inbox'} on:click={() => switchTab('inbox')}>
            Bandeja ({inboxCount})
          </button>
          <button class:active={currentTab === 'today'} on:click={() => switchTab('today')}>
            Hoy ({todayCount})
          </button>
          <button class:active={currentTab === 'upcoming'} on:click={() => switchTab('upcoming')}>
            Próximos ({upcomingCount})
          </button>
        </div>
      </div>

      <div class="search-row">
        <input
          class="search"
          bind:this={searchInputRef}
          bind:value={searchInput}
          placeholder="Buscar por título, proyecto o etiqueta..."
          on:input={onSearchInput}
        />
      </div>

      {#if isProjectView && selectedProject}
        <div class="project-controls">
          <div class="project-title">Proyecto: @{selectedProject.name}</div>
          <div class="project-actions">
            <div class="order-dropdown" bind:this={orderDropdownRef}>
              <span class="order-label">Orden:</span>
              <button
                class="order-trigger"
                class:open={orderDropdownOpen}
                on:click={toggleOrderDropdown}
              >
                {projectSortMode === 'manual' ? 'Manual' : 'Automático'}
                <span class="order-chevron">▾</span>
              </button>

              {#if orderDropdownOpen}
                <div class="order-menu" class:open-up={orderMenuOpenUp} bind:this={orderMenuRef} transition:fade={{ duration: 120 }}>
                  <button
                    class:active={projectSortMode === 'auto'}
                    on:click={() => onSetProjectSortMode('auto')}
                  >
                    Automático
                  </button>
                  <button
                    class:active={projectSortMode === 'manual'}
                    on:click={() => onSetProjectSortMode('manual')}
                  >
                    Manual
                  </button>
                </div>
              {/if}
            </div>
            <button on:click={onRecalculateProjectOrder}>Actualizar/Recalcular</button>
            <button on:click={onResetProjectToAuto}>Volver a automático</button>
          </div>
        </div>
      {/if}

      {#if error}
        <p class="error">{error}</p>
      {/if}

      <div class="workbench" class:project-view={isProjectView}>
        <section class="list">
          {#if loading}
            <p class="empty">Cargando...</p>
          {:else if isProjectView}
            {#if filteredProjectTasks.length === 0}
              <p class="empty">No hay tareas en el proyecto para los filtros actuales.</p>
            {:else}
              {#each filteredProjectTasks as task (task.id)}
                <article
                  class="task {task.status === 'done' ? 'done' : ''} {priorityClass(task.priority)} {(menuTaskId === task.id || keyboardSelectedTaskId === task.id) ? 'selected' : ''}"
                  transition:fly={{ y: 8, duration: 180, easing: cubicOut }}
                  draggable={projectSortMode === 'manual' && searchQuery.length === 0}
                  on:dragstart={() => onDragStart(task.id)}
                  on:dragover={(e) => {
                    if (projectSortMode === 'manual' && searchQuery.length === 0) {
                      e.preventDefault();
                    }
                  }}
                  on:drop={() => onDropOnTask(task.id)}
                >
                  <label class="check">
                    <input
                      type="checkbox"
                      checked={task.status === 'done'}
                      on:change={() => onToggleTask(task)}
                    />
                  </label>

                  <div class="task-body">
                    {#if editingId === task.id}
                      <input
                        class="title-edit"
                        bind:value={editingTitle}
                        on:blur={() => saveEdit(task.id)}
                        on:keydown={(e) => {
                          if (e.key === 'Enter') {
                            e.preventDefault();
                            saveEdit(task.id);
                          }
                          if (e.key === 'Escape') {
                            e.preventDefault();
                            cancelEdit();
                          }
                        }}
                      />
                    {:else}
                      <button class="title" on:dblclick={() => startEditing(task)}>{@html highlightText(task.title, searchQuery)}</button>
                    {/if}

                    <div class="meta-line">
                      {#if task.projectName}
                        <span class="chip project" class:hit-chip={searchQuery.length > 0 && (task.projectName ?? '').toLowerCase().includes(searchQuery)}>@{task.projectName}</span>
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
                    <button class="more" on:click={() => openMenu(task)}>⋯</button>
                  </div>
                </article>
              {/each}
            {/if}
          {:else if currentTab === 'inbox'}
            {#if filteredInbox.length === 0}
              <div class="empty-state">
                <p class="empty">Bandeja vacía. Prueba: <code>Comprar pan @Personal #compras p2</code></p>
                <button on:click={focusQuickAdd}>Ir a creación rápida</button>
              </div>
            {:else}
              {#each filteredInbox as task (task.id)}
                <article
                  class="task {task.status === 'done' ? 'done' : ''} {priorityClass(task.priority)} {(menuTaskId === task.id || keyboardSelectedTaskId === task.id) ? 'selected' : ''}"
                  transition:fly={{ y: 8, duration: 180, easing: cubicOut }}
                >
                  <label class="check"><input type="checkbox" checked={task.status === 'done'} on:change={() => onToggleTask(task)} /></label>
                  <div class="task-body">
                    {#if editingId === task.id}
                      <input
                        class="title-edit"
                        bind:value={editingTitle}
                        on:blur={() => saveEdit(task.id)}
                        on:keydown={(e) => {
                          if (e.key === 'Enter') {
                            e.preventDefault();
                            saveEdit(task.id);
                          }
                          if (e.key === 'Escape') {
                            e.preventDefault();
                            cancelEdit();
                          }
                        }}
                      />
                    {:else}
                      <button class="title" on:dblclick={() => startEditing(task)}>{@html highlightText(task.title, searchQuery)}</button>
                    {/if}
                    <div class="meta-line">
                      {#if task.projectName}<span class="chip project" class:hit-chip={searchQuery.length > 0 && (task.projectName ?? '').toLowerCase().includes(searchQuery)}>@{task.projectName}</span>{/if}
                      {#each task.labels as label}<span class="chip label" class:hit-chip={searchQuery.length > 0 && label.toLowerCase().includes(searchQuery)}>#{label}</span>{/each}
                      {#if task.dueDate}<span class="chip due">Vence: {task.dueDate}</span>{/if}
                      {#if task.recurrence}<span class="chip recurrence">{formatRecurrenceLabel(task.recurrence)}</span>{/if}
                    </div>
                  </div>
                  <span class="priority-badge">P{task.priority}</span>
                  <div class="menu-wrap"><button class="more" on:click={() => openMenu(task)}>⋯</button></div>
                </article>
              {/each}
            {/if}
          {:else}
            {#if filteredOverdue.length > 0}
              <div class="section-header">Vencidas ({filteredOverdue.length})</div>
              {#each filteredOverdue as task (task.id)}
                <article
                  class="task overdue {task.status === 'done' ? 'done' : ''} {priorityClass(task.priority)} {(menuTaskId === task.id || keyboardSelectedTaskId === task.id) ? 'selected' : ''}"
                  transition:fly={{ y: 8, duration: 180, easing: cubicOut }}
                >
                  <label class="check"><input type="checkbox" checked={task.status === 'done'} on:change={() => onToggleTask(task)} /></label>
                  <div class="task-body">
                    {#if editingId === task.id}
                      <input
                        class="title-edit"
                        bind:value={editingTitle}
                        on:blur={() => saveEdit(task.id)}
                        on:keydown={(e) => {
                          if (e.key === 'Enter') {
                            e.preventDefault();
                            saveEdit(task.id);
                          }
                          if (e.key === 'Escape') {
                            e.preventDefault();
                            cancelEdit();
                          }
                        }}
                      />
                    {:else}
                      <button class="title" on:dblclick={() => startEditing(task)}>{@html highlightText(task.title, searchQuery)}</button>
                    {/if}
                    <div class="meta-line">
                      {#if task.projectName}<span class="chip project" class:hit-chip={searchQuery.length > 0 && (task.projectName ?? '').toLowerCase().includes(searchQuery)}>@{task.projectName}</span>{/if}
                      {#each task.labels as label}<span class="chip label" class:hit-chip={searchQuery.length > 0 && label.toLowerCase().includes(searchQuery)}>#{label}</span>{/each}
                      {#if task.dueDate}<span class="chip due">Vence: {task.dueDate}</span>{/if}
                      {#if task.recurrence}<span class="chip recurrence">{formatRecurrenceLabel(task.recurrence)}</span>{/if}
                    </div>
                  </div>
                  <span class="priority-badge">P{task.priority}</span>
                  <div class="menu-wrap"><button class="more" on:click={() => openMenu(task)}>⋯</button></div>
                </article>
              {/each}
            {/if}

            {#if currentTab === 'today' && filteredToday.length > 0}
              <div class="section-header">Hoy ({filteredToday.length})</div>
              {#each filteredToday as task (task.id)}
                <article
                  class="task {task.status === 'done' ? 'done' : ''} {priorityClass(task.priority)} {(menuTaskId === task.id || keyboardSelectedTaskId === task.id) ? 'selected' : ''}"
                  transition:fly={{ y: 8, duration: 180, easing: cubicOut }}
                >
                  <label class="check"><input type="checkbox" checked={task.status === 'done'} on:change={() => onToggleTask(task)} /></label>
                  <div class="task-body">
                    {#if editingId === task.id}
                      <input
                        class="title-edit"
                        bind:value={editingTitle}
                        on:blur={() => saveEdit(task.id)}
                        on:keydown={(e) => {
                          if (e.key === 'Enter') {
                            e.preventDefault();
                            saveEdit(task.id);
                          }
                          if (e.key === 'Escape') {
                            e.preventDefault();
                            cancelEdit();
                          }
                        }}
                      />
                    {:else}
                      <button class="title" on:dblclick={() => startEditing(task)}>{@html highlightText(task.title, searchQuery)}</button>
                    {/if}
                    <div class="meta-line">
                      {#if task.projectName}<span class="chip project" class:hit-chip={searchQuery.length > 0 && (task.projectName ?? '').toLowerCase().includes(searchQuery)}>@{task.projectName}</span>{/if}
                      {#each task.labels as label}<span class="chip label" class:hit-chip={searchQuery.length > 0 && label.toLowerCase().includes(searchQuery)}>#{label}</span>{/each}
                      {#if task.dueDate}<span class="chip due">Vence: {task.dueDate}</span>{/if}
                      {#if task.recurrence}<span class="chip recurrence">{formatRecurrenceLabel(task.recurrence)}</span>{/if}
                    </div>
                  </div>
                  <span class="priority-badge">P{task.priority}</span>
                  <div class="menu-wrap"><button class="more" on:click={() => openMenu(task)}>⋯</button></div>
                </article>
              {/each}
            {/if}

            {#if currentTab === 'upcoming' && filteredUpcoming.length > 0}
              <div class="section-header">Próximos ({filteredUpcoming.length})</div>
              {#each filteredUpcoming as task (task.id)}
                <article
                  class="task {task.status === 'done' ? 'done' : ''} {priorityClass(task.priority)} {(menuTaskId === task.id || keyboardSelectedTaskId === task.id) ? 'selected' : ''}"
                  transition:fly={{ y: 8, duration: 180, easing: cubicOut }}
                >
                  <label class="check"><input type="checkbox" checked={task.status === 'done'} on:change={() => onToggleTask(task)} /></label>
                  <div class="task-body">
                    {#if editingId === task.id}
                      <input
                        class="title-edit"
                        bind:value={editingTitle}
                        on:blur={() => saveEdit(task.id)}
                        on:keydown={(e) => {
                          if (e.key === 'Enter') {
                            e.preventDefault();
                            saveEdit(task.id);
                          }
                          if (e.key === 'Escape') {
                            e.preventDefault();
                            cancelEdit();
                          }
                        }}
                      />
                    {:else}
                      <button class="title" on:dblclick={() => startEditing(task)}>{@html highlightText(task.title, searchQuery)}</button>
                    {/if}
                    <div class="meta-line">
                      {#if task.projectName}<span class="chip project" class:hit-chip={searchQuery.length > 0 && (task.projectName ?? '').toLowerCase().includes(searchQuery)}>@{task.projectName}</span>{/if}
                      {#each task.labels as label}<span class="chip label" class:hit-chip={searchQuery.length > 0 && label.toLowerCase().includes(searchQuery)}>#{label}</span>{/each}
                      {#if task.dueDate}<span class="chip due">Vence: {task.dueDate}</span>{/if}
                      {#if task.recurrence}<span class="chip recurrence">{formatRecurrenceLabel(task.recurrence)}</span>{/if}
                    </div>
                  </div>
                  <span class="priority-badge">P{task.priority}</span>
                  <div class="menu-wrap"><button class="more" on:click={() => openMenu(task)}>⋯</button></div>
                </article>
              {/each}
            {/if}

            {#if filteredOverdue.length === 0 && ((currentTab === 'today' && filteredToday.length === 0) || (currentTab === 'upcoming' && filteredUpcoming.length === 0))}
              {#if currentTab === 'today'}
                <div class="empty-state">
                  <p class="empty">Nada para hoy.</p>
                  <button on:click={planTomorrowFromQuickAdd}>Planear mañana</button>
                </div>
              {:else}
                <div class="empty-state">
                  <p class="empty">Sin tareas próximas.</p>
                  <button on:click={focusQuickAdd}>Crear tarea</button>
                </div>
              {/if}
            {/if}
          {/if}
        </section>

        {#if !taskPanelAsModal}
          <aside class="context-panel" transition:fade={{ duration: 120 }}>
            {#if selectedMenuTask}
              <div class="panel-head">
                <h3>Acciones</h3>
                <button class="icon-button" aria-label="Cerrar panel de tarea" on:click={closeTaskPanel}>
                  <X size={18} strokeWidth={2.1} />
                </button>
              </div>
              <p class="context-title">{selectedMenuTask.title}</p>

              <div class="menu-group">
                <p>Prioridad</p>
                <div class="menu-inline">
                  {#each [1, 2, 3, 4] as level}
                    <button on:click={() => onUpdatePriority(selectedMenuTask.id, level)}>P{level}</button>
                  {/each}
                </div>
              </div>

              <div class="menu-group">
                <p>Fecha</p>
                <div class="menu-inline">
                  <button on:click={() => onPickDueShortcut(selectedMenuTask.id, 'today')}>Hoy</button>
                  <button on:click={() => onPickDueShortcut(selectedMenuTask.id, 'tomorrow')}>Mañana</button>
                  <button on:click={() => onPickDueShortcut(selectedMenuTask.id, null)}>Quitar</button>
                </div>
                <div class="date-picker" bind:this={datePickerRef}>
                  <button class="context-trigger date-trigger" class:open={datePickerOpen} on:click={toggleDatePicker}>
                    {dueDateDraft || 'Seleccionar fecha'}
                    <span class="order-chevron">▾</span>
                  </button>
                  {#if datePickerOpen}
                    <div class="date-picker-menu" class:open-up={datePickerOpenUp} bind:this={datePickerMenuRef} transition:fade={{ duration: 120 }}>
                      <div class="calendar-head">
                        <button class="nav-btn" on:click={() => shiftCalendarMonth(-1)} aria-label="Mes anterior">‹</button>
                        <strong>{calendarMonthLabel}</strong>
                        <button class="nav-btn" on:click={() => shiftCalendarMonth(1)} aria-label="Mes siguiente">›</button>
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
                              on:click={() => applyDateFromPicker(selectedMenuTask.id, cell.iso!)}
                            >
                              {cell.label}
                            </button>
                          {:else}
                            <span class="day-empty"></span>
                          {/if}
                        {/each}
                      </div>
                      <div class="menu-inline date-manual">
                        <input class="date-field" placeholder="YYYY-MM-DD" bind:value={dueDateDraft} />
                        <button on:click={() => applyManualDate(selectedMenuTask.id)}>Aplicar</button>
                      </div>
                    </div>
                  {/if}
                </div>
              </div>

              <div class="menu-group">
                <p>Recurrencia</p>
                <div class="context-dropdown" bind:this={recurrenceDropdownRef}>
                  <button
                    class="context-trigger"
                    class:open={recurrenceDropdownOpen}
                    on:click={toggleRecurrenceDropdown}
                  >
                    {selectedRecurrenceLabel}
                    <span class="order-chevron">▾</span>
                  </button>
                  {#if recurrenceDropdownOpen}
                    <div class="context-menu" class:open-up={recurrenceMenuOpenUp} bind:this={recurrenceMenuRef} transition:fade={{ duration: 110 }}>
                      {#each recurrenceOptions as option}
                        <button
                          class:active={(selectedMenuTask.recurrence ?? null) === option.value}
                          on:click={() => onUpdateRecurrence(selectedMenuTask.id, option.value)}
                        >
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
                  <button
                    class="context-trigger"
                    class:open={projectDropdownOpen}
                    on:click={toggleProjectDropdown}
                  >
                    {selectedProjectLabel}
                    <span class="order-chevron">▾</span>
                  </button>
                  {#if projectDropdownOpen}
                    <div class="context-menu" class:open-up={projectMenuOpenUp} bind:this={projectMenuRef} transition:fade={{ duration: 110 }}>
                      <button
                        class:active={selectedMenuTask.projectId === null}
                        on:click={() => onMoveProject(selectedMenuTask.id, null)}
                      >
                        Bandeja
                      </button>
                      {#each projects as project}
                        <button
                          class:active={selectedMenuTask.projectId === project.id}
                          on:click={() => onMoveProject(selectedMenuTask.id, project.id)}
                        >
                          @{project.name}
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
                <div class="menu-inline">
                  <input placeholder="Crear proyecto..." bind:value={projectDraft} />
                  <button on:click={() => onCreateAndMoveProject(selectedMenuTask.id)}>Crear</button>
                </div>
              </div>

              <button class="danger" on:click={() => requestDeleteTask(selectedMenuTask.id)}>Eliminar</button>
              {#if selectedMenuTask.externalUrl}
                <button on:click={() => openExternalUrl(selectedMenuTask.externalUrl!)}>Abrir en GitHub</button>
              {/if}
            {:else}
              <h3>Panel de tarea</h3>
              <p class="context-hint">
                Selecciona una tarea con `⋯` para editar prioridad, fecha, recurrencia o proyecto. Cuando abras una tarea, podrás cerrar este panel con `✕` o `Escape`.
              </p>
            {/if}
          </aside>
        {/if}
      </div>
    </section>
  </section>

  {#if taskPanelAsModal && selectedMenuTask}
    <div class="modal-backdrop" transition:fade={{ duration: 120 }}>
      <div class="confirm-modal task-modal" transition:fly={{ y: 10, duration: 160 }}>
        <div class="panel-head task-modal-head">
          <div class="task-modal-title-block">
            <h4>Acciones</h4>
            <p class="context-title task-modal-title">{selectedMenuTask.title}</p>
          </div>
          <button class="icon-button" aria-label="Cerrar panel de tarea" on:click={closeTaskPanel}>
            <X size={18} strokeWidth={2.1} />
          </button>
        </div>

        <div class="menu-group">
          <p>Prioridad</p>
          <div class="menu-inline task-modal-actions">
            {#each [1, 2, 3, 4] as level}
              <button class="button-secondary compact-action" on:click={() => onUpdatePriority(selectedMenuTask.id, level)}>P{level}</button>
            {/each}
          </div>
        </div>

        <div class="menu-group">
          <p>Fecha</p>
          <div class="menu-inline task-modal-actions">
            <button class="button-secondary compact-action" on:click={() => onPickDueShortcut(selectedMenuTask.id, 'today')}>Hoy</button>
            <button class="button-secondary compact-action" on:click={() => onPickDueShortcut(selectedMenuTask.id, 'tomorrow')}>Mañana</button>
            <button class="button-secondary compact-action" on:click={() => onPickDueShortcut(selectedMenuTask.id, null)}>Quitar</button>
          </div>
          <div class="date-picker" bind:this={datePickerRef}>
            <button class="context-trigger date-trigger" class:open={datePickerOpen} on:click={toggleDatePicker}>
              {dueDateDraft || 'Seleccionar fecha'}
              <span class="order-chevron">▾</span>
            </button>
            {#if datePickerOpen}
              <div class="date-picker-menu" class:open-up={datePickerOpenUp} bind:this={datePickerMenuRef} transition:fade={{ duration: 120 }}>
                <div class="calendar-head">
                  <button class="nav-btn" on:click={() => shiftCalendarMonth(-1)} aria-label="Mes anterior">‹</button>
                  <strong>{calendarMonthLabel}</strong>
                  <button class="nav-btn" on:click={() => shiftCalendarMonth(1)} aria-label="Mes siguiente">›</button>
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
                        on:click={() => applyDateFromPicker(selectedMenuTask.id, cell.iso!)}
                      >
                        {cell.label}
                      </button>
                    {:else}
                      <span class="day-empty"></span>
                    {/if}
                  {/each}
                </div>
                <div class="menu-inline date-manual task-modal-actions">
                  <input class="date-field task-modal-input" placeholder="YYYY-MM-DD" bind:value={dueDateDraft} />
                  <button class="button-primary compact-action" on:click={() => applyManualDate(selectedMenuTask.id)}>Aplicar</button>
                </div>
              </div>
            {/if}
          </div>
        </div>

        <div class="menu-group">
          <p>Recurrencia</p>
          <div class="context-dropdown" bind:this={recurrenceDropdownRef}>
            <button
              class="context-trigger"
              class:open={recurrenceDropdownOpen}
              on:click={toggleRecurrenceDropdown}
            >
              {selectedRecurrenceLabel}
              <span class="order-chevron">▾</span>
            </button>
            {#if recurrenceDropdownOpen}
              <div class="context-menu" class:open-up={recurrenceMenuOpenUp} bind:this={recurrenceMenuRef} transition:fade={{ duration: 110 }}>
                {#each recurrenceOptions as option}
                  <button
                    class:active={(selectedMenuTask.recurrence ?? null) === option.value}
                    on:click={() => onUpdateRecurrence(selectedMenuTask.id, option.value)}
                  >
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
            <button
              class="context-trigger"
              class:open={projectDropdownOpen}
              on:click={toggleProjectDropdown}
            >
              {selectedProjectLabel}
              <span class="order-chevron">▾</span>
            </button>
            {#if projectDropdownOpen}
              <div class="context-menu" class:open-up={projectMenuOpenUp} bind:this={projectMenuRef} transition:fade={{ duration: 110 }}>
                <button
                  class:active={selectedMenuTask.projectId === null}
                  on:click={() => onMoveProject(selectedMenuTask.id, null)}
                >
                  Bandeja
                </button>
                {#each projects as project}
                  <button
                    class:active={selectedMenuTask.projectId === project.id}
                    on:click={() => onMoveProject(selectedMenuTask.id, project.id)}
                  >
                    @{project.name}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
          <div class="menu-inline task-modal-actions">
            <input class="task-modal-input" placeholder="Crear proyecto..." bind:value={projectDraft} />
            <button class="button-primary compact-action" on:click={() => onCreateAndMoveProject(selectedMenuTask.id)}>Crear</button>
          </div>
        </div>

        <div class="confirm-actions">
          <button class="button-danger button-with-icon" on:click={() => requestDeleteTask(selectedMenuTask.id)}>
            <Trash2 size={16} strokeWidth={2.1} />
            <span>Eliminar tarea</span>
          </button>
          {#if selectedMenuTask.externalUrl}
            <button class="button-secondary button-with-icon" on:click={() => openExternalUrl(selectedMenuTask.externalUrl!)}>
              <ExternalLink size={16} strokeWidth={2.1} />
              <span>Abrir en GitHub</span>
            </button>
          {/if}
          <button class="button-secondary button-with-icon" on:click={closeTaskPanel}>
            <X size={16} strokeWidth={2.1} />
            <span>Cerrar</span>
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if undoState}
    <div class="snackbar" transition:fade={{ duration: 140 }}>
      <span>Tarea eliminada: {undoState.title}</span>
      <button on:click={undoDelete}>Deshacer</button>
    </div>
  {/if}

  {#if recurrenceToast}
    <div class="snackbar recurrence" transition:fade={{ duration: 140 }}>
      <span>{recurrenceToast}</span>
    </div>
  {/if}

  {#if confirmDeleteOpen}
    <div class="modal-backdrop" transition:fade={{ duration: 120 }}>
      <div class="confirm-modal" transition:fly={{ y: 10, duration: 160 }}>
        <div class="modal-head">
          <h4>Confirmar eliminación</h4>
          <button
            class="icon-button"
            aria-label="Cancelar eliminación"
            on:click={() => {
              confirmDeleteOpen = false;
              confirmDeleteTaskId = null;
              confirmDeleteTaskTitle = '';
            }}
          >
            <X size={18} strokeWidth={2.1} />
          </button>
        </div>
        <p>
          Vas a eliminar <strong>{confirmDeleteTaskTitle}</strong>. Después podrás deshacerlo durante unos segundos.
        </p>
        <div class="confirm-actions">
          <button
            class="button-secondary"
            on:click={() => {
              confirmDeleteOpen = false;
              confirmDeleteTaskId = null;
              confirmDeleteTaskTitle = '';
            }}
          >
            Cancelar
          </button>
          <button class="button-danger" on:click={confirmDeleteTask}>Eliminar tarea</button>
        </div>
      </div>
    </div>
  {/if}

  {#if confirmResetOpen}
    <div class="modal-backdrop" transition:fade={{ duration: 120 }}>
      <div class="confirm-modal" transition:fly={{ y: 10, duration: 160 }}>
        <div class="modal-head">
          <h4>¿Reiniciar orden manual?</h4>
          <button class="icon-button" aria-label="Cerrar confirmación" on:click={() => (confirmResetOpen = false)}>
            <X size={18} strokeWidth={2.1} />
          </button>
        </div>
        <p>Se limpiará el orden manual del proyecto y volverá a orden automático.</p>
        <div class="confirm-actions">
          <button class="button-secondary" on:click={() => (confirmResetOpen = false)}>Cancelar</button>
          <button class="button-danger" on:click={confirmResetProjectToAuto}>Confirmar</button>
        </div>
      </div>
    </div>
  {/if}

  {#if backupModalOpen}
    <div class="modal-backdrop" transition:fade={{ duration: 120 }}>
      <div class="confirm-modal backup-modal" transition:fly={{ y: 10, duration: 160 }}>
        <div class="modal-head">
          <h4>Exportar / importar tareas</h4>
          <button class="icon-button" aria-label="Cerrar respaldo" on:click={() => (backupModalOpen = false)}>
            <X size={18} strokeWidth={2.1} />
          </button>
        </div>
        <p>Genera un JSON con tus tareas activas y completadas, o importa un backup existente haciendo merge por ID.</p>
        <div class="backup-actions">
          <button class="button-primary button-with-icon" disabled={backupBusy} on:click={onExportBackup}>
            <Download size={16} strokeWidth={2.1} />
            <span>Exportar JSON</span>
          </button>
          <button class="button-secondary button-with-icon" disabled={backupBusy} on:click={triggerImportBackupFile}>
            <Upload size={16} strokeWidth={2.1} />
            <span>Importar archivo</span>
          </button>
          <input
            bind:this={backupFileInputRef}
            class="hidden-file"
            type="file"
            accept=".json,application/json"
            on:change={onImportBackupFile}
          />
        </div>
        <textarea
          class="backup-textarea"
          bind:value={backupJsonDraft}
          placeholder="Pega aquí un backup JSON para importarlo"
        ></textarea>
        {#if backupNotice}
          <p class="backup-notice">{backupNotice}</p>
        {/if}
        <div class="confirm-actions">
          <button class="button-secondary button-with-icon" on:click={() => (backupModalOpen = false)}>
            <X size={16} strokeWidth={2.1} />
            <span>Cerrar</span>
          </button>
          <button class="button-primary button-with-icon" disabled={backupBusy || !backupJsonDraft.trim()} on:click={onImportBackupDraft}>
            <Upload size={16} strokeWidth={2.1} />
            <span>Importar JSON pegado</span>
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if githubModalOpen}
    <div class="modal-backdrop" transition:fade={{ duration: 120 }}>
      <div class="confirm-modal github-modal" transition:fly={{ y: 10, duration: 160 }}>
        <div class="modal-head">
          <h4>Configuración de GitHub</h4>
          <button class="icon-button" aria-label="Cerrar configuración de GitHub" on:click={() => (githubModalOpen = false)}>
            <X size={18} strokeWidth={2.1} />
          </button>
        </div>

        <p>
          Usa este panel para conectar tu cuenta, elegir qué importar y activar notificaciones de GitHub dentro de KitoDo.
        </p>

        <section class="github-setup-card">
          <h5>Antes de conectar</h5>
          <div class="github-setup-list">
            {#each githubSetupItems as item}
              <div class="github-setup-item">
                <strong>{item.title}</strong>
                <span>{item.description}</span>
              </div>
            {/each}
          </div>
          <div class="github-doc-links">
            {#each githubDocLinks as link}
              <button class="ghost-trigger compact button-with-icon" on:click={() => openExternalUrl(link.url)}>
                <ExternalLink size={16} strokeWidth={2.1} />
                <span>{link.label}</span>
              </button>
            {/each}
          </div>
        </section>

        {#if githubAccounts.length === 0}
          <section class="github-config-grid">
            <label class="github-field">
              <span>Personal access token</span>
              <input placeholder="ghp_..." bind:value={githubTokenInput} type="password" />
            </label>
            <div class="github-status-line">
              <small>
                Para importar notificaciones usa un token classic. Si solo conectas un token fine-grained, KitoDo podrá quedar limitado.
              </small>
            </div>
            <div class="confirm-actions">
              <button class="button-primary" disabled={githubBusy || !githubTokenInput.trim()} on:click={onGithubConnect}>Conectar GitHub</button>
            </div>
          </section>
        {:else}
          <section class="github-config-grid">
            <div class="github-status-card">
              <div class="github-status-head">
                <strong>{selectedGithubAccountLabel}</strong>
                <div class="status-badges">
                  <span class="status-badge" class:ok={selectedGithubAccount?.tokenKind === 'classic'}>
                    {githubTokenKindLabel}
                  </span>
                  <span class="status-badge" class:ok={githubSettings?.enabled}>
                    {githubSettings?.enabled ? 'Sync activa' : 'Sync desactivada'}
                  </span>
                  <span class="status-badge" class:ok={githubNotificationsReady}>
                    {githubNotificationsReady ? 'Notificaciones listas' : 'Notificaciones limitadas'}
                  </span>
                </div>
              </div>
              <div class="github-status-list">
                <span>
                  {githubNotificationsReady
                    ? 'Tu token permite usar la API de notifications.'
                    : 'Para importar notificaciones necesitas un personal access token classic.'}
                </span>
                <span>
                  {githubStatus?.lastSyncAt
                    ? `Última sincronización: ${githubStatus.lastSyncAt}`
                    : 'Todavía no se ha ejecutado ninguna sincronización.'}
                </span>
                {#if githubStatus?.lastError}
                  <span class="status-error">Último error: {githubStatus.lastError}</span>
                {/if}
              </div>
            </div>

            <div class="context-dropdown" bind:this={githubAccountDropdownRef}>
              <button
                class="context-trigger"
                class:open={githubAccountDropdownOpen}
                on:click={toggleGithubAccountDropdown}
              >
                {selectedGithubAccountLabel}
                <span class="order-chevron">▾</span>
              </button>
              {#if githubAccountDropdownOpen}
                <div class="context-menu" class:open-up={githubAccountMenuOpenUp} bind:this={githubAccountMenuRef}>
                  {#each githubAccounts as account}
                    <button
                      class:active={selectedGithubAccountId === account.accountId}
                      on:click={async () => {
                        selectedGithubAccountId = account.accountId;
                        githubAccountDropdownOpen = false;
                        githubAccountMenuOpenUp = false;
                        await refreshGithubState();
                      }}
                    >
                      {account.username} ({account.tokenKind})
                    </button>
                  {/each}
                </div>
              {/if}
            </div>

            {#if githubSettings}
              <label class="gh-inline">
                <input
                  type="checkbox"
                  checked={githubSettings.enabled}
                  on:change={(e) => onGithubUpdateSettings({ enabled: (e.currentTarget as HTMLInputElement).checked })}
                />
                Sincronización automática
              </label>
              <label class="gh-inline">
                <span>Intervalo</span>
                <div class="context-dropdown" bind:this={githubIntervalDropdownRef}>
                  <button
                    class="context-trigger"
                    class:open={githubIntervalDropdownOpen}
                    on:click={toggleGithubIntervalDropdown}
                  >
                    {selectedGithubIntervalLabel}
                    <span class="order-chevron">▾</span>
                  </button>
                  {#if githubIntervalDropdownOpen}
                    <div class="context-menu" class:open-up={githubIntervalMenuOpenUp} bind:this={githubIntervalMenuRef}>
                      {#each [
                        { value: 60, label: '1m' },
                        { value: 300, label: '5m' },
                        { value: 600, label: '10m' },
                        { value: 1800, label: '30m' }
                      ] as opt}
                        <button
                          class:active={githubSettings.syncIntervalSec === opt.value}
                          on:click={() => {
                            githubIntervalDropdownOpen = false;
                            githubIntervalMenuOpenUp = false;
                            onGithubUpdateSettings({ syncIntervalSec: opt.value });
                          }}
                        >
                          {opt.label}
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
              </label>
              <label class="gh-inline">
                <input
                  type="checkbox"
                  checked={githubSettings.importPrReviews}
                  on:change={(e) => onGithubUpdateSettings({ importPrReviews: (e.currentTarget as HTMLInputElement).checked })}
                />
                Revisiones de PR
              </label>
              <label class="gh-inline">
                <input
                  type="checkbox"
                  checked={githubSettings.importAssignedIssues}
                  on:change={(e) => onGithubUpdateSettings({ importAssignedIssues: (e.currentTarget as HTMLInputElement).checked })}
                />
                Issues asignadas
              </label>
              <label class="gh-inline">
                <input
                  type="checkbox"
                  checked={githubSettings.importNotifications}
                  on:change={(e) => onGithubUpdateSettings({ importNotifications: (e.currentTarget as HTMLInputElement).checked })}
                />
                Notificaciones de GitHub
              </label>

              <label class="gh-inline">
                <span>Proyecto destino</span>
                <div class="context-dropdown" bind:this={githubProjectDropdownRef}>
                  <button
                    class="context-trigger"
                    class:open={githubProjectDropdownOpen}
                    on:click={toggleGithubProjectDropdown}
                  >
                    {selectedGithubProjectLabel}
                    <span class="order-chevron">▾</span>
                  </button>
                  {#if githubProjectDropdownOpen}
                    <div class="context-menu" class:open-up={githubProjectMenuOpenUp} bind:this={githubProjectMenuRef}>
                      <button
                        class:active={!githubSettings.defaultProjectId}
                        on:click={() => {
                          githubProjectDropdownOpen = false;
                          githubProjectMenuOpenUp = false;
                          onGithubUpdateSettings({ defaultProjectId: null });
                        }}
                      >
                        Bandeja GitHub (auto)
                      </button>
                      {#each projects as project}
                        <button
                          class:active={githubSettings.defaultProjectId === project.id}
                          on:click={() => {
                            githubProjectDropdownOpen = false;
                            githubProjectMenuOpenUp = false;
                            onGithubUpdateSettings({ defaultProjectId: project.id });
                          }}
                        >
                          @{project.name}
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
              </label>
            {/if}

            <div class="gh-actions">
              <button class="button-primary button-with-icon" disabled={githubBusy} on:click={onGithubSyncNow}>
                <RefreshCw size={16} strokeWidth={2.1} />
                <span>Sincronizar ahora</span>
              </button>
              <button class="button-secondary button-with-icon" disabled={githubBusy} on:click={onGithubDisconnect}>
                <Unplug size={16} strokeWidth={2.1} />
                <span>Desconectar</span>
              </button>
            </div>

            <div class="gh-add-repo">
              <input class="github-repo-input" placeholder="owner/repo" bind:value={githubRepoInput} />
              <button class="button-primary button-with-icon" disabled={githubBusy} on:click={onGithubAddRepo}>
                <Plus size={16} strokeWidth={2.1} />
                <span>Añadir</span>
              </button>
            </div>
            <div class="gh-repos">
              {#each githubRepoSubs as sub}
                <div class="gh-repo-item">
                  <span>{sub.owner}/{sub.repo}</span>
                  <div class="gh-actions">
                    <button class="button-secondary compact-action" on:click={() => onGithubToggleRepo(sub)}>
                      {sub.enabled ? 'Desactivar' : 'Activar'}
                    </button>
                    <button class="button-danger compact-action button-with-icon" on:click={() => onGithubRemoveRepo(sub)}>
                      <Trash2 size={14} strokeWidth={2.1} />
                      <span>Quitar</span>
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          </section>
        {/if}

        {#if githubMessage}
          <p class="backup-notice">{githubMessage}</p>
        {/if}

        <div class="confirm-actions">
          <button class="button-secondary button-with-icon" on:click={() => (githubModalOpen = false)}>
            <X size={16} strokeWidth={2.1} />
            <span>Cerrar</span>
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if helpModalOpen}
    <div class="modal-backdrop" transition:fade={{ duration: 120 }}>
      <div class="confirm-modal help-modal" transition:fly={{ y: 10, duration: 160 }}>
        <div class="modal-head">
          <h4>Guía rápida de KitoDo</h4>
          <button class="icon-button" aria-label="Cerrar ayuda" on:click={() => (helpModalOpen = false)}>
            <X size={18} strokeWidth={2.1} />
          </button>
        </div>
        <p>
          KitoDo está pensado para capturar tareas rápido y luego refinarlas desde filtros y el panel lateral.
          Los atajos globales no se disparan mientras escribes en un campo de texto.
        </p>
        <div class="shortcut-sections">
          {#each shortcutSections as section}
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
          <button class="ghost-trigger compact button-with-icon" on:click={showQuickGuide}>
            <Eye size={16} strokeWidth={2.1} />
            <span>Mostrar guía</span>
          </button>
          <button class="button-secondary button-with-icon" on:click={() => (helpModalOpen = false)}>
            <X size={16} strokeWidth={2.1} />
            <span>Cerrar</span>
          </button>
        </div>
      </div>
    </div>
  {/if}
</main>

<style>
  .shell {
    width: 100vw;
    height: 100vh;
    display: grid;
    place-items: stretch;
    padding: 0;
    position: relative;
    overflow: hidden;
  }

  .panel {
    width: 100%;
    height: 100%;
    min-height: 0;
    border-radius: 0;
    border: 1px solid var(--k-border);
    background: var(--k-panel);
    backdrop-filter: blur(16px);
    box-shadow:
      0 0 0 1px rgba(166, 12, 219, 0.22),
      0 16px 60px rgba(0, 0, 0, 0.5),
      0 0 48px rgba(192, 75, 255, 0.2);
    padding: 18px;
    display: grid;
    gap: 14px;
    overflow: hidden;
    transition: grid-template-columns 720ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .panel.expanded {
    grid-template-columns: 250px 1fr;
    align-items: stretch;
  }

  .content {
    display: grid;
    grid-template-rows: auto auto auto auto auto minmax(0, 1fr);
    gap: 4px;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .sidebar {
    border-radius: 14px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(12, 12, 18, 0.7);
    padding: 10px;
    display: grid;
    gap: 10px;
    align-content: start;
    min-height: 0;
    overflow: auto;
    scrollbar-color: rgba(192, 75, 255, 0.62) rgba(255, 255, 255, 0.05);
  }

  .sidebar-head,
  .panel-head,
  .modal-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  .sidebar-head {
    padding-bottom: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .sidebar-head h2 {
    margin: 0;
    font-size: 0.95rem;
    letter-spacing: 0.04em;
  }

  .sidebar-head p {
    margin: 4px 0 0;
    color: var(--k-muted);
    font-size: 0.8rem;
    line-height: 1.4;
  }

  .side-block {
    display: grid;
    gap: 6px;
  }

  .side-block h3 {
    margin: 0;
    color: var(--k-muted);
    font-size: 0.77rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .side-block button,
  .clear-filters {
    text-align: left;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: var(--k-text);
    padding: 6px 8px;
    cursor: pointer;
  }

  .side-block button.active,
  .clear-filters:hover {
    border-color: rgba(192, 75, 255, 0.66);
    box-shadow: 0 0 10px rgba(192, 75, 255, 0.24);
  }

  .side-block input {
    width: 100%;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: var(--k-text);
    padding: 6px 8px;
  }

  .github-block {
    gap: 8px;
  }

  .sidebar-badges {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .gh-inline {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: 8px;
    font-size: 0.8rem;
    color: var(--k-muted);
  }

  .gh-actions {
    display: flex;
    gap: 6px;
  }

  .gh-add-repo {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 6px;
  }

  .gh-repos {
    display: grid;
    gap: 6px;
    max-height: 150px;
    overflow: auto;
    scrollbar-color: rgba(137, 198, 255, 0.46) rgba(255, 255, 255, 0.05);
  }

  .gh-repo-item {
    display: flex;
    justify-content: space-between;
    gap: 6px;
    align-items: center;
    font-size: 0.78rem;
    color: var(--k-text);
  }

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

  .backup-trigger {
    border-radius: 10px;
    border: 1px solid rgba(192, 75, 255, 0.45);
    background: rgba(192, 75, 255, 0.12);
    color: #efd8ff;
    padding: 8px 12px;
    cursor: pointer;
    white-space: nowrap;
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

  .button-primary,
  .button-secondary,
  .button-danger {
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

  .button-primary:disabled,
  .button-secondary:disabled,
  .button-danger:disabled {
    opacity: 0.55;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
  }

  .compact-action {
    padding: 7px 10px;
    min-height: 34px;
    font-size: 0.78rem;
  }

  .danger-soft {
    border-color: rgba(255, 120, 145, 0.4);
    color: #ffd4de;
  }

  .window-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    justify-content: flex-end;
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

  .quick-add {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 10px;
  }

  .quick-add input,
  .search {
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(10, 10, 14, 0.7);
    color: var(--k-text);
    padding: 11px 12px;
    outline: none;
  }

  .quick-add input:focus,
  .search:focus {
    border-color: rgba(192, 75, 255, 0.7);
    box-shadow: 0 0 0 3px rgba(166, 12, 219, 0.2);
  }

  .quick-add button,
  .tabs button,
  .more,
  .title,
  .context-panel button {
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.04);
    color: var(--k-text);
    cursor: pointer;
  }

  .quick-add button {
    min-width: 120px;
    padding: 0 14px;
    font-weight: 600;
    background: linear-gradient(135deg, rgba(166, 12, 219, 0.52), rgba(192, 75, 255, 0.44));
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

  .toolbar {
    display: flex;
    justify-content: flex-start;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .tabs {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    width: 100%;
  }

  .tabs button {
    padding: 8px 12px;
    min-height: 40px;
  }

  .tabs button.active {
    border-color: rgba(192, 75, 255, 0.8);
    box-shadow: 0 0 18px rgba(192, 75, 255, 0.25);
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

  .search-row {
    width: 100%;
    margin-bottom: -2px;
  }

  .search {
    width: 100%;
    display: block;
  }

  .project-controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.03);
    margin-bottom: 2px;
  }

  .project-title {
    color: #dcb4ff;
    font-size: 0.86rem;
  }

  .project-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .project-actions button {
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.04);
    color: var(--k-text);
    padding: 6px 8px;
  }

  .order-dropdown {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .order-label {
    font-size: 0.84rem;
    color: var(--k-muted);
  }

  .order-trigger {
    min-width: 108px;
    display: inline-flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    border-radius: 8px;
    border: 1px solid rgba(192, 75, 255, 0.55) !important;
    background: rgba(255, 255, 255, 0.05) !important;
    box-shadow: 0 0 0 1px rgba(166, 12, 219, 0.2), 0 0 16px rgba(192, 75, 255, 0.14);
    transition: border-color 180ms ease, box-shadow 180ms ease, transform 180ms ease;
  }

  .order-trigger.open {
    border-color: rgba(192, 75, 255, 0.9) !important;
    box-shadow: 0 0 0 3px rgba(166, 12, 219, 0.22), 0 0 20px rgba(192, 75, 255, 0.2);
  }

  .order-chevron {
    opacity: 0.9;
    font-size: 0.74rem;
  }

  .order-menu {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    min-width: 130px;
    padding: 6px;
    display: grid;
    gap: 5px;
    border-radius: 10px;
    border: 1px solid rgba(192, 75, 255, 0.45);
    background: rgba(19, 16, 28, 0.98);
    box-shadow: 0 14px 24px rgba(0, 0, 0, 0.35), 0 0 18px rgba(192, 75, 255, 0.16);
    z-index: 40;
  }

  .order-menu.open-up {
    top: auto;
    bottom: calc(100% + 6px);
    transform-origin: bottom left;
  }

  .order-menu button {
    text-align: left;
    border-radius: 7px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.03);
    color: var(--k-text);
    padding: 7px 8px;
  }

  .order-menu button:hover {
    border-color: rgba(192, 75, 255, 0.55);
    background: rgba(192, 75, 255, 0.14);
  }

  .order-menu button.active {
    border-color: rgba(192, 75, 255, 0.8);
    background: linear-gradient(135deg, rgba(166, 12, 219, 0.45), rgba(192, 75, 255, 0.38));
    color: #fff;
  }

  .context-dropdown {
    position: relative;
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

  .workbench {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(260px, 30%);
    gap: 12px;
    align-items: stretch;
    margin-top: 2px;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }

  .workbench.project-view {
    margin-top: 8px;
    padding-top: 10px;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  .list {
    display: grid;
    gap: 10px;
    align-content: start;
    min-height: 0;
    height: 100%;
    align-self: stretch;
    overflow: auto;
    padding-right: 2px;
    scrollbar-color: rgba(192, 75, 255, 0.62) rgba(255, 255, 255, 0.05);
  }

  .section-header {
    padding: 6px 8px;
    border-radius: 8px;
    color: var(--k-muted);
    font-size: 0.78rem;
    text-transform: uppercase;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.03);
  }

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

  :global(mark.hit) {
    background: rgba(255, 222, 117, 0.26);
    color: #fff5cf;
    border-radius: 4px;
    padding: 0 2px;
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
  }

  .context-panel {
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    background: rgba(14, 14, 20, 0.86);
    backdrop-filter: blur(12px);
    padding: 10px;
    display: grid;
    gap: 10px;
    box-shadow: 0 14px 38px rgba(0, 0, 0, 0.28);
    max-height: none;
    min-height: 0;
    min-width: 0;
    align-self: stretch;
    overflow: visible;
    position: relative;
    z-index: 30;
  }

  .context-panel h3 {
    margin: 0;
    font-size: 0.88rem;
    color: var(--k-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
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

  .date-picker {
    position: relative;
  }

  .date-trigger {
    width: 100%;
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

  .context-panel button,
  .context-panel input {
    padding: 6px 8px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.05);
    color: var(--k-text);
  }

  .context-panel input {
    width: 100%;
  }

  .context-panel .date-field {
    min-height: 34px;
    border-color: rgba(192, 75, 255, 0.46);
    background: linear-gradient(180deg, rgba(28, 18, 40, 0.92), rgba(18, 14, 28, 0.92));
    box-shadow: inset 0 0 0 1px rgba(166, 12, 219, 0.12);
  }

  .context-panel .date-field:focus {
    outline: none;
    border-color: rgba(192, 75, 255, 0.88);
    box-shadow: 0 0 0 2px rgba(166, 12, 219, 0.24), inset 0 0 0 1px rgba(166, 12, 219, 0.15);
  }

  .danger {
    border-color: rgba(255, 120, 145, 0.5);
    color: #ffc4d3;
    justify-self: end;
  }

  .snackbar {
    position: fixed;
    left: 50%;
    bottom: 24px;
    transform: translateX(-50%);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    background: rgba(14, 14, 20, 0.94);
    color: var(--k-text);
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 9px 12px;
    z-index: 200;
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.35);
  }

  .snackbar button {
    border-radius: 8px;
    border: 1px solid rgba(192, 75, 255, 0.5);
    color: #e8ccff;
    background: rgba(192, 75, 255, 0.15);
    padding: 6px 8px;
    cursor: pointer;
  }

  .snackbar.recurrence {
    bottom: 70px;
  }

  .empty {
    color: var(--k-muted);
    text-align: center;
    margin-top: 30px;
  }

  .empty-state {
    display: grid;
    justify-items: center;
    gap: 8px;
    padding-top: 16px;
  }

  .empty-state button {
    border-radius: 8px;
    border: 1px solid rgba(192, 75, 255, 0.5);
    background: rgba(192, 75, 255, 0.15);
    color: #f0ddff;
    padding: 6px 10px;
    cursor: pointer;
  }

  .error {
    margin: 0;
    color: #ff86ad;
    font-size: 0.86rem;
  }

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

  .help-modal {
    width: min(760px, 94vw);
  }

  .github-modal {
    width: min(760px, 94vw);
    max-height: min(88vh, 920px);
    overflow: auto;
    overscroll-behavior: contain;
    scrollbar-color: rgba(192, 75, 255, 0.56) rgba(255, 255, 255, 0.05);
  }

  .task-modal {
    width: min(560px, 94vw);
    max-height: min(88vh, 860px);
    overflow: auto;
    overscroll-behavior: contain;
    scrollbar-color: rgba(192, 75, 255, 0.56) rgba(255, 255, 255, 0.05);
    gap: 14px;
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

  .task-modal .menu-group {
    gap: 8px;
    padding: 10px;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.03);
  }

  .task-modal .menu-group p {
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

  .github-config-grid {
    display: grid;
    gap: 10px;
  }

  .github-field {
    display: grid;
    gap: 6px;
    color: var(--k-muted);
    font-size: 0.84rem;
  }

  .github-field input {
    width: 100%;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: var(--k-text);
    padding: 10px 12px;
  }

  .github-repo-input {
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(10, 10, 14, 0.7);
    color: var(--k-text);
    padding: 11px 12px;
    outline: none;
  }

  .github-repo-input:focus {
    border-color: rgba(192, 75, 255, 0.7);
    box-shadow: 0 0 0 3px rgba(166, 12, 219, 0.2);
  }

  .github-setup-card {
    display: grid;
    gap: 10px;
    padding: 12px;
    border-radius: 12px;
    border: 1px solid rgba(137, 198, 255, 0.22);
    background: rgba(255, 255, 255, 0.03);
  }

  .github-status-card {
    display: grid;
    gap: 10px;
    padding: 12px;
    border-radius: 12px;
    border: 1px solid rgba(192, 75, 255, 0.24);
    background:
      linear-gradient(135deg, rgba(192, 75, 255, 0.08), rgba(137, 198, 255, 0.05)),
      rgba(255, 255, 255, 0.03);
  }

  .github-status-head {
    display: grid;
    gap: 8px;
  }

  .github-status-head strong {
    font-size: 0.92rem;
  }

  .status-badges {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .status-badge {
    border-radius: 999px;
    padding: 5px 9px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.05);
    color: var(--k-muted);
    font-size: 0.78rem;
  }

  .status-badge.ok {
    border-color: rgba(155, 255, 184, 0.4);
    color: #c9ffd8;
    background: rgba(64, 120, 78, 0.2);
  }

  .github-status-list {
    display: grid;
    gap: 6px;
    color: var(--k-muted);
    font-size: 0.84rem;
    line-height: 1.45;
  }

  .status-error {
    color: #ffc4d3;
  }

  .github-setup-card h5 {
    margin: 0;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--k-muted);
  }

  .github-setup-list {
    display: grid;
    gap: 8px;
  }

  .github-setup-item {
    display: grid;
    gap: 4px;
  }

  .github-setup-item strong {
    font-size: 0.88rem;
  }

  .github-setup-item span,
  .github-status-line small {
    color: var(--k-muted);
    font-size: 0.84rem;
    line-height: 1.45;
  }

  .github-doc-links {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .shortcut-sections {
    display: grid;
    gap: 12px;
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

  .confirm-actions button {
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.06);
    color: var(--k-text);
    padding: 6px 9px;
    cursor: pointer;
  }

  @media (max-width: 1280px) {
    .panel {
      padding: 14px;
      overflow: auto;
    }

    .panel.expanded {
      grid-template-columns: 1fr;
    }

    .sidebar {
      order: 2;
      max-height: none;
    }

    .guide-examples {
      grid-template-columns: 1fr;
    }

    .content {
      order: 1;
      grid-template-rows: auto auto auto auto auto auto minmax(0, 1fr);
    }

    .workbench {
      grid-template-columns: 1fr;
    }

    .context-panel {
      max-height: none;
      overflow: visible;
    }

    .project-controls {
      flex-direction: column;
      align-items: stretch;
    }

    .project-actions {
      width: 100%;
    }

    .context-panel {
      order: -1;
    }

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

  @media (max-width: 960px) {
    .shell {
      padding: 0;
    }

    .panel {
      padding: 12px;
    }

    .content {
      gap: 8px;
    }

    .quick-add {
      grid-template-columns: 1fr;
    }

    .quick-add button {
      min-height: 42px;
    }

    .header-actions {
      justify-items: end;
      width: auto;
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

    .guide-actions {
      justify-content: stretch;
      flex-direction: column;
      align-items: stretch;
    }

    .progress-wrap {
      width: auto;
      min-width: 0;
      text-align: right;
    }

    .progress-bar {
      width: 100%;
    }

    .tabs {
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }

    .tabs button {
      width: 100%;
      padding: 9px 8px;
      font-size: 0.84rem;
    }

    .project-title {
      overflow-wrap: anywhere;
    }

    .project-actions {
      display: grid;
      grid-template-columns: 1fr;
    }

    .order-dropdown {
      display: grid;
      grid-template-columns: 1fr;
      align-items: stretch;
    }

    .order-label {
      margin-bottom: 2px;
    }

    .order-trigger {
      width: 100%;
      min-width: 0;
    }

    .gh-inline,
    .gh-add-repo,
    .gh-repo-item,
    .backup-actions,
    .confirm-actions {
      grid-template-columns: 1fr;
      flex-direction: column;
      align-items: stretch;
    }

    .gh-repo-item {
      display: grid;
    }

    .gh-actions {
      justify-content: flex-start;
      flex-wrap: wrap;
    }

    .context-panel {
      padding: 12px;
    }

    .menu-inline > * {
      flex: 1 1 120px;
    }

    .sidebar-head,
    .panel-head,
    .modal-head {
      align-items: stretch;
      flex-direction: column;
    }

    .task-modal .panel-head {
      align-items: flex-start;
      flex-direction: row;
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

    .backup-modal {
      width: min(94vw, 680px);
    }

    .help-modal {
      width: min(94vw, 760px);
    }

    .github-modal {
      width: min(94vw, 760px);
      max-height: min(90vh, 920px);
    }

    .backup-textarea {
      min-height: 180px;
    }

    .shortcut-row {
      grid-template-columns: 1fr;
      gap: 6px;
    }

    .github-doc-links {
      flex-direction: column;
      align-items: stretch;
    }

    .status-badges {
      flex-direction: column;
    }

    .snackbar {
      width: min(92vw, 460px);
      justify-content: space-between;
      align-items: flex-start;
      flex-wrap: wrap;
    }
  }

  @media (max-width: 460px) {
    .panel {
      padding: 10px;
      gap: 10px;
    }

    .tabs {
      grid-template-columns: 1fr;
    }

    .tabs button,
    .project-actions button,
    .context-panel button,
    .backup-actions button,
    .confirm-actions button,
    .example-pill {
      width: 100%;
    }

    .header-tool-button {
      width: 38px;
      height: 38px;
    }

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

    .date-picker-menu {
      width: min(94vw, 320px);
    }

    .snackbar {
      bottom: 14px;
      width: calc(100vw - 20px);
    }
  }
</style>
