export type ShortcutItem = {
  keys: string[];
  description: string;
};

export type ShortcutSection = {
  title: string;
  items: ShortcutItem[];
};

export type QuickAddExample = {
  label: string;
  value: string;
};

export type GithubSetupItem = {
  title: string;
  description: string;
};

export type GithubDocLink = {
  label: string;
  url: string;
};

export const quickAddExamples: QuickAddExample[] = [
  { label: 'Proyecto y etiqueta', value: 'Preparar demo @Trabajo #frontend p2' },
  { label: 'Con fecha', value: 'Pagar dominio @Admin #finanzas due tomorrow' },
  { label: 'Recurrente', value: 'Plan semanal @Casa #rutina every week' }
];

export const githubSetupItems: GithubSetupItem[] = [
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

export const githubDocLinks: GithubDocLink[] = [
  { label: 'Crear token classic', url: 'https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens' },
  { label: 'API de notifications', url: 'https://docs.github.com/en/rest/activity/notifications' },
  { label: 'Configurar notificaciones', url: 'https://docs.github.com/github/managing-subscriptions-and-notifications-on-github/setting-up-notifications/configuring-notifications' }
];

export const shortcutSections: ShortcutSection[] = [
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
