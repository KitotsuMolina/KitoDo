<script lang="ts">
  import { quintInOut, quintOut } from 'svelte/easing';
  import { fade, fly } from 'svelte/transition';
  import { browser } from '$app/environment';
  import { onMount, tick } from 'svelte';
  import {
    exportBackupJson,
    importBackupJson,
    initDb,
    moveTaskToProject,
    reorderProjectTasks,
    resetProjectManualOrder,
    setProjectSortMode,
    updateTaskDueDate,
    updateTaskPriority,
    updateTaskRecurrence,
    type GithubAccountDTO,
    type GithubSettingsDTO,
    type GithubStatusDTO,
    type ImportResultDTO,
    type LabelDTO,
    type ProjectDTO,
    type RepoSubDTO,
    type TaskDTO
  } from '$lib/api/desktop';
  import HeaderBar from '$lib/components/HeaderBar.svelte';
  import HelpModal from '$lib/components/HelpModal.svelte';
  import QuickGuideCard from '$lib/components/QuickGuideCard.svelte';
  import SidebarFilters from '$lib/components/SidebarFilters.svelte';
  import TaskActionsContent from '$lib/components/TaskActionsContent.svelte';
  import TaskCard from '$lib/components/TaskCard.svelte';
  import BackupModal from '$lib/components/BackupModal.svelte';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import GithubModal from '$lib/components/GithubModal.svelte';
  import {
    githubDocLinks,
    githubSetupItems,
    quickAddExamples,
    shortcutSections
  } from '$lib/constants/app-content';
  import {
    addGithubRepo,
    connectGithubAccount,
    disconnectGithubAccount,
    loadGithubState,
    removeGithubRepo,
    syncGithubAccount,
    toggleGithubRepo,
    updateGithubSettings,
    type GithubSettingsPatch,
    type GithubStateSnapshot
  } from '$lib/features/github-state';
  import {
    createTaskFromQuickInput,
    deleteTaskById,
    loadProjectTaskData,
    loadTaskData,
    restoreTaskById,
    saveTaskTitle,
    toggleTask,
    type TaskDataSnapshot
  } from '$lib/features/task-data';
  import { buildCalendarCells, dueShortcut, formatYmd, parseYmd } from '$lib/utils/date-helpers';
  import {
    filterTasks,
    formatRecurrenceLabel,
    highlightText,
    mergeUniqueTasks,
    priorityClass
  } from '$lib/utils/task-helpers';
  import { getNextKeyboardSelection, isTextInputFocused, shouldCloseFloatingUi } from '$lib/utils/keyboard-helpers';

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

  function currentKeyboardTasks(): TaskDTO[] {
    if (isProjectView) return filteredProjectTasks;
    if (currentTab === 'inbox') return filteredInbox;
    if (currentTab === 'today') return [...filteredOverdue, ...filteredToday];
    return [...filteredOverdue, ...filteredUpcoming];
  }

  function cycleKeyboardSelection(direction: 1 | -1) {
    keyboardSelectedTaskId = getNextKeyboardSelection(keyboardTasks, keyboardSelectedTaskId, direction);
  }

  function closeDeleteConfirm() {
    confirmDeleteOpen = false;
    confirmDeleteTaskId = null;
    confirmDeleteTaskTitle = '';
  }

  function closeFloatingMenus() {
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

  function hasOpenFloatingMenus(): boolean {
    return (
      orderDropdownOpen ||
      recurrenceDropdownOpen ||
      projectDropdownOpen ||
      datePickerOpen ||
      githubAccountDropdownOpen ||
      githubIntervalDropdownOpen ||
      githubProjectDropdownOpen
    );
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

  function applyGithubState(snapshot: GithubStateSnapshot) {
    githubAccounts = snapshot.accounts;
    githubSettings = snapshot.settings;
    githubRepoSubs = snapshot.repoSubs;
    githubStatus = snapshot.status;
    selectedGithubAccountId = snapshot.selectedAccountId;
  }

  async function refreshGithubState() {
    applyGithubState(await loadGithubState(selectedGithubAccountId));
  }

  async function refreshAll() {
    const snapshot: TaskDataSnapshot = await loadTaskData(showDone, selectedProjectId);
    tasksByTab = snapshot.tasksByTab;
    overdueTasks = snapshot.overdueTasks;
    projectTasks = snapshot.projectTasks;
    projects = snapshot.projects;
    labels = snapshot.labels;
    todayAll = snapshot.todayAll;
  }

  async function refreshProjectTasks() {
    projectTasks = await loadProjectTaskData(selectedProjectId, showDone);
  }

  async function onGithubConnect() {
    if (!githubTokenInput.trim()) return;
    githubBusy = true;
    githubMessage = '';
    try {
      const { account, message } = await connectGithubAccount(githubTokenInput);
      selectedGithubAccountId = account.accountId;
      githubTokenInput = '';
      await refreshGithubState();
      githubMessage = message;
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
      githubMessage = await disconnectGithubAccount(selectedGithubAccountId);
      await refreshGithubState();
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
      githubMessage = await syncGithubAccount(selectedGithubAccountId);
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
    githubBusy = true;
    try {
      await addGithubRepo(selectedGithubAccountId, githubRepoInput);
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
      await toggleGithubRepo(sub);
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
      await removeGithubRepo(sub);
      await refreshGithubState();
    } catch (e) {
      error = String(e);
    } finally {
      githubBusy = false;
    }
  }

  async function onGithubUpdateSettings(patch: GithubSettingsPatch) {
    if (!selectedGithubAccountId) return;
    githubBusy = true;
    try {
      githubSettings = await updateGithubSettings(selectedGithubAccountId, patch);
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
      await createTaskFromQuickInput(value);
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
      const result = await toggleTask(task.id);
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
      await saveTaskTitle(taskId, title);
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
      await deleteTaskById(taskId);
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
      await restoreTaskById(taskId);
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

      if (!event.ctrlKey && !event.metaKey && !isTextInputFocused(document.activeElement as HTMLElement | null)) {
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
          closeDeleteConfirm();
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

        if (hasOpenFloatingMenus()) {
          event.preventDefault();
          closeFloatingMenus();
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
      if (shouldCloseFloatingUi(target)) {
        closeFloatingMenus();
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
      <div
        in:fly={{ x: -42, duration: 700, easing: quintOut }}
        out:fly={{ x: -34, duration: 640, easing: quintInOut }}
        on:outroend={() => {
          sidebarClosing = false;
        }}
      >
        <SidebarFilters
          tabNames={tabNames}
          tabCounts={tabCounts}
          currentTab={currentTab}
          {projects}
          {labels}
          {selectedProjectId}
          {selectedLabel}
          githubAccountsCount={githubAccounts.length}
          {selectedGithubAccountLabel}
          {githubSettings}
          {githubStatus}
          onClose={closeSidebar}
          onSelectTab={switchTab}
          onSelectProject={onSelectProject}
          onSelectLabel={(label) => (selectedLabel = label)}
          onOpenGithub={() => (githubModalOpen = true)}
          onClearFilters={clearFilters}
        />
      </div>
    {/if}

    <section class="content">
      <HeaderBar
        {pendingCount}
        {todayPendingCount}
        {todayDoneCount}
        {todayTotalCount}
        {dayProgress}
        {showDone}
        {expandedMode}
        onToggleShowDone={toggleShowDone}
        onToggleSidebar={toggleSidebar}
        onOpenHelp={() => (helpModalOpen = true)}
        onOpenBackup={() => {
          backupModalOpen = true;
          backupNotice = '';
        }}
        onCloseApp={closeAppWindow}
      />

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
        <QuickGuideCard
          examples={quickAddExamples}
          onHide={hideQuickGuide}
          onOpenHelp={() => (helpModalOpen = true)}
          onSelectExample={(value) => {
            quickInput = value;
            quickInputRef?.focus();
            quickInputRef?.select();
          }}
        />
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
                <TaskCard
                  {task}
                  {searchQuery}
                  {editingId}
                  bind:editingTitle
                  {menuTaskId}
                  {keyboardSelectedTaskId}
                  extraClasses={priorityClass(task.priority)}
                  draggable={projectSortMode === 'manual' && searchQuery.length === 0}
                  {highlightText}
                  {formatRecurrenceLabel}
                  {onToggleTask}
                  onStartEditing={startEditing}
                  onSaveEdit={saveEdit}
                  onCancelEdit={cancelEdit}
                  onOpenMenu={openMenu}
                  onDragStart={onDragStart}
                  onDragOver={(e) => {
                    if (projectSortMode === 'manual' && searchQuery.length === 0) {
                      e.preventDefault();
                    }
                  }}
                  onDrop={onDropOnTask}
                />
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
                <TaskCard
                  {task}
                  {searchQuery}
                  {editingId}
                  bind:editingTitle
                  {menuTaskId}
                  {keyboardSelectedTaskId}
                  extraClasses={priorityClass(task.priority)}
                  {highlightText}
                  {formatRecurrenceLabel}
                  {onToggleTask}
                  onStartEditing={startEditing}
                  onSaveEdit={saveEdit}
                  onCancelEdit={cancelEdit}
                  onOpenMenu={openMenu}
                />
              {/each}
            {/if}
          {:else}
            {#if filteredOverdue.length > 0}
              <div class="section-header">Vencidas ({filteredOverdue.length})</div>
              {#each filteredOverdue as task (task.id)}
                <TaskCard
                  {task}
                  {searchQuery}
                  {editingId}
                  bind:editingTitle
                  {menuTaskId}
                  {keyboardSelectedTaskId}
                  extraClasses={`overdue ${priorityClass(task.priority)}`}
                  {highlightText}
                  {formatRecurrenceLabel}
                  {onToggleTask}
                  onStartEditing={startEditing}
                  onSaveEdit={saveEdit}
                  onCancelEdit={cancelEdit}
                  onOpenMenu={openMenu}
                />
              {/each}
            {/if}

            {#if currentTab === 'today' && filteredToday.length > 0}
              <div class="section-header">Hoy ({filteredToday.length})</div>
              {#each filteredToday as task (task.id)}
                <TaskCard
                  {task}
                  {searchQuery}
                  {editingId}
                  bind:editingTitle
                  {menuTaskId}
                  {keyboardSelectedTaskId}
                  extraClasses={priorityClass(task.priority)}
                  {highlightText}
                  {formatRecurrenceLabel}
                  {onToggleTask}
                  onStartEditing={startEditing}
                  onSaveEdit={saveEdit}
                  onCancelEdit={cancelEdit}
                  onOpenMenu={openMenu}
                />
              {/each}
            {/if}

            {#if currentTab === 'upcoming' && filteredUpcoming.length > 0}
              <div class="section-header">Próximos ({filteredUpcoming.length})</div>
              {#each filteredUpcoming as task (task.id)}
                <TaskCard
                  {task}
                  {searchQuery}
                  {editingId}
                  bind:editingTitle
                  {menuTaskId}
                  {keyboardSelectedTaskId}
                  extraClasses={priorityClass(task.priority)}
                  {highlightText}
                  {formatRecurrenceLabel}
                  {onToggleTask}
                  onStartEditing={startEditing}
                  onSaveEdit={saveEdit}
                  onCancelEdit={cancelEdit}
                  onOpenMenu={openMenu}
                />
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
            <TaskActionsContent
              selectedTask={selectedMenuTask}
              variant="sidebar"
              {selectedRecurrenceLabel}
              {selectedProjectLabel}
              {recurrenceOptions}
              {projects}
              bind:dueDateDraft
              bind:projectDraft
              {calendarMonthLabel}
              {calendarCells}
              {recurrenceDropdownOpen}
              {projectDropdownOpen}
              {datePickerOpen}
              {recurrenceMenuOpenUp}
              {projectMenuOpenUp}
              {datePickerOpenUp}
              bind:datePickerRef
              bind:datePickerMenuRef
              bind:recurrenceDropdownRef
              bind:recurrenceMenuRef
              bind:projectDropdownRef
              bind:projectMenuRef
              onClose={closeTaskPanel}
              {onUpdatePriority}
              {onPickDueShortcut}
              onToggleDatePicker={toggleDatePicker}
              onShiftCalendarMonth={shiftCalendarMonth}
              onApplyDateFromPicker={applyDateFromPicker}
              onApplyManualDate={applyManualDate}
              onToggleRecurrenceDropdown={toggleRecurrenceDropdown}
              {onUpdateRecurrence}
              onToggleProjectDropdown={toggleProjectDropdown}
              {onMoveProject}
              {onCreateAndMoveProject}
              onRequestDeleteTask={requestDeleteTask}
              onOpenExternalUrl={openExternalUrl}
            />
          </aside>
        {/if}
      </div>
    </section>
  </section>

  {#if taskPanelAsModal && selectedMenuTask}
    <div class="modal-backdrop" transition:fade={{ duration: 120 }}>
      <div class="confirm-modal task-modal" transition:fly={{ y: 10, duration: 160 }}>
        <TaskActionsContent
          selectedTask={selectedMenuTask}
          variant="modal"
          {selectedRecurrenceLabel}
          {selectedProjectLabel}
          {recurrenceOptions}
          {projects}
          bind:dueDateDraft
          bind:projectDraft
          {calendarMonthLabel}
          {calendarCells}
          {recurrenceDropdownOpen}
          {projectDropdownOpen}
          {datePickerOpen}
          {recurrenceMenuOpenUp}
          {projectMenuOpenUp}
          {datePickerOpenUp}
          bind:datePickerRef
          bind:datePickerMenuRef
          bind:recurrenceDropdownRef
          bind:recurrenceMenuRef
          bind:projectDropdownRef
          bind:projectMenuRef
          onClose={closeTaskPanel}
          {onUpdatePriority}
          {onPickDueShortcut}
          onToggleDatePicker={toggleDatePicker}
          onShiftCalendarMonth={shiftCalendarMonth}
          onApplyDateFromPicker={applyDateFromPicker}
          onApplyManualDate={applyManualDate}
          onToggleRecurrenceDropdown={toggleRecurrenceDropdown}
          {onUpdateRecurrence}
          onToggleProjectDropdown={toggleProjectDropdown}
          {onMoveProject}
          {onCreateAndMoveProject}
          onRequestDeleteTask={requestDeleteTask}
          onOpenExternalUrl={openExternalUrl}
        />
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

  <ConfirmModal
    open={confirmDeleteOpen}
    title="Confirmar eliminación"
    confirmLabel="Eliminar tarea"
    closeLabel="Cancelar eliminación"
    onClose={() => {
      closeDeleteConfirm();
    }}
    onConfirm={confirmDeleteTask}
  >
    <p>
      Vas a eliminar <strong>{confirmDeleteTaskTitle}</strong>. Después podrás deshacerlo durante unos segundos.
    </p>
  </ConfirmModal>

  <ConfirmModal
    open={confirmResetOpen}
    title="¿Reiniciar orden manual?"
    confirmLabel="Confirmar"
    onClose={() => (confirmResetOpen = false)}
    onConfirm={confirmResetProjectToAuto}
  >
    <p>Se limpiará el orden manual del proyecto y volverá a orden automático.</p>
  </ConfirmModal>

  <BackupModal
    open={backupModalOpen}
    busy={backupBusy}
    bind:draft={backupJsonDraft}
    notice={backupNotice}
    bind:fileInputRef={backupFileInputRef}
    onClose={() => (backupModalOpen = false)}
    onExport={onExportBackup}
    onTriggerImportFile={triggerImportBackupFile}
    onImportFile={onImportBackupFile}
    onImportDraft={onImportBackupDraft}
  />

  <GithubModal
    open={githubModalOpen}
    busy={githubBusy}
    message={githubMessage}
    {githubAccounts}
    {githubSettings}
    {githubRepoSubs}
    {githubStatus}
    {selectedGithubAccountId}
    {selectedGithubAccountLabel}
    {selectedGithubIntervalLabel}
    {selectedGithubProjectLabel}
    {githubTokenKindLabel}
    {githubNotificationsReady}
    bind:githubTokenInput
    bind:githubRepoInput
    {githubAccountDropdownOpen}
    {githubIntervalDropdownOpen}
    {githubProjectDropdownOpen}
    {githubAccountMenuOpenUp}
    {githubIntervalMenuOpenUp}
    {githubProjectMenuOpenUp}
    {githubSetupItems}
    {githubDocLinks}
    {projects}
    bind:githubAccountDropdownRef
    bind:githubIntervalDropdownRef
    bind:githubProjectDropdownRef
    bind:githubAccountMenuRef
    bind:githubIntervalMenuRef
    bind:githubProjectMenuRef
    onClose={() => (githubModalOpen = false)}
    onOpenExternalUrl={openExternalUrl}
    onConnect={onGithubConnect}
    onDisconnect={onGithubDisconnect}
    onSyncNow={onGithubSyncNow}
    onAddRepo={onGithubAddRepo}
    onToggleRepo={onGithubToggleRepo}
    onRemoveRepo={onGithubRemoveRepo}
    onUpdateSettings={(patch) => {
      if ('syncIntervalSec' in patch) {
        githubIntervalDropdownOpen = false;
        githubIntervalMenuOpenUp = false;
      }
      if ('defaultProjectId' in patch) {
        githubProjectDropdownOpen = false;
        githubProjectMenuOpenUp = false;
      }
      onGithubUpdateSettings(patch);
    }}
    onSelectAccount={async (account) => {
      selectedGithubAccountId = account.accountId;
      githubAccountDropdownOpen = false;
      githubAccountMenuOpenUp = false;
      await refreshGithubState();
    }}
    onToggleAccountDropdown={toggleGithubAccountDropdown}
    onToggleIntervalDropdown={toggleGithubIntervalDropdown}
    onToggleProjectDropdown={toggleGithubProjectDropdown}
  />

  <HelpModal
    open={helpModalOpen}
    sections={shortcutSections}
    onClose={() => (helpModalOpen = false)}
    onShowGuide={showQuickGuide}
  />
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
  .context-panel {
    border-radius: 10px;
  }

  .quick-add button,
  .tabs button {
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

  :global(mark.hit) {
    background: rgba(255, 222, 117, 0.26);
    color: #fff5cf;
    border-radius: 4px;
    padding: 0 2px;
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

  .task-modal {
    width: min(560px, 94vw);
    max-height: min(88vh, 860px);
    overflow: auto;
    overscroll-behavior: contain;
    scrollbar-color: rgba(192, 75, 255, 0.56) rgba(255, 255, 255, 0.05);
    gap: 14px;
  }

  @media (max-width: 1280px) {
    .panel {
      padding: 14px;
      overflow: auto;
    }

    .panel.expanded {
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

    .context-panel {
      padding: 12px;
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
    .project-actions button {
      width: 100%;
    }

    .snackbar {
      bottom: 14px;
      width: calc(100vw - 20px);
    }
  }
</style>
