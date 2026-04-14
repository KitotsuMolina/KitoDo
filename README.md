# KitoDo v0.6.6

KitoDo es una app de tareas estilo launcher para Linux. El runtime actual usa:

- `SvelteKit + TypeScript` para la UI
- `Electron` como shell de escritorio
- `Rust + SQLite` como backend local (`kitodo-server`)

La app mantiene una arquitectura de bajo acoplamiento: Electron solo hospeda la ventana y actúa como bridge seguro; toda la lógica de datos y GitHub vive en Rust.

## Guía rápida de uso

### Qué es KitoDo

KitoDo está pensado para capturar tareas muy rápido y ordenarlas después sin salir del teclado.

La idea principal es:

- escribir una tarea en una sola línea
- añadir proyecto y etiquetas directamente en el texto
- usar filtros y panel lateral para refinar prioridad, fecha y repetición

### Cómo crear tareas

La forma más rápida es usar la entrada principal de la app.

Ejemplos:

```text
Preparar demo @Trabajo #frontend p2
Pagar dominio @Admin #finanzas due tomorrow
Plan semanal @Casa #rutina every week
```

### Sintaxis rápida

KitoDo entiende estos fragmentos dentro de la línea:

- `@Proyecto`: asigna el proyecto
- `#tag`: añade una etiqueta
- `p1` a `p4`: define prioridad
- `due today`, `due tomorrow` o `due YYYY-MM-DD`: define fecha
- `every day`, `every week`, `every month` o `every mon..sun`: define repetición

Importante:

- los tokens `due` y `every` siguen estando en inglés porque forman parte de la sintaxis real del parser
- el texto del título puede ir en español sin problema

### Filtros y paneles

El panel lateral de filtros permite cambiar vista, filtrar por proyecto, etiqueta y abrir la configuración de GitHub.

Puedes abrirlo o cerrarlo de tres formas:

- botón `Filtros`
- atajo `Shift + F`
- tecla `Escape`

El panel de tarea se abre con el botón `⋯` de cada tarea. Desde ahí puedes cambiar:

- prioridad
- fecha
- repetición
- proyecto
- eliminación

Cuando el panel está abierto, puedes cerrarlo con:

- botón `✕`
- tecla `Escape`

### Eliminar tareas

Eliminar ya no es una acción silenciosa.

Ahora KitoDo:

- pide confirmación antes de borrar
- muestra un aviso de deshacer durante unos segundos

### Atajos principales

- `Ctrl + K`: enfocar entrada rápida
- `Ctrl + F`: enfocar buscador
- `/`: enfocar buscador
- `Ctrl + 1`: ir a Bandeja
- `Ctrl + 2`: ir a Hoy
- `Ctrl + 3`: ir a Próximos
- `Shift + F`: abrir o cerrar filtros
- `j` / `k`: mover selección de tarea
- `Enter`: editar título de la tarea seleccionada
- `x`: completar o reabrir tarea seleccionada
- `Delete`: pedir eliminación de la tarea seleccionada
- `?`: abrir ayuda
- `Escape`: cerrar modales, paneles o filtros
- `F11`: maximizar o restaurar ventana

### GitHub, backups y notificaciones

KitoDo también incluye:

- integración con GitHub
- exportación e importación en JSON

### Cómo preparar GitHub correctamente

No necesitas instalar GitHub CLI ni herramientas extra para la sincronización.

Sí necesitas:

- conexión a internet
- una cuenta de GitHub
- un `personal access token (classic)` si quieres importar notificaciones de GitHub
- un llavero del sistema disponible en Linux para guardar el token de forma segura

Pasos recomendados:

1. Abre el panel de GitHub desde KitoDo.
2. Crea un `personal access token (classic)` en GitHub.
3. Si quieres notificaciones, usa token classic. Los fine-grained no sirven para el endpoint de notifications.
4. Si trabajas con repos privados, añade permisos suficientes para esos repos.
5. Conecta el token en KitoDo.
6. Añade uno o más repos con formato `owner/repo`.
7. Activa qué quieres importar:
   - revisiones de PR
   - issues asignadas
   - notificaciones
8. Ajusta el intervalo de sincronización.

Notas importantes:

- KitoDo importa las notificaciones que GitHub ya te entrega; no sustituye la configuración de notificaciones de tu cuenta.
- Si el token no se guarda correctamente, revisa que tu entorno Linux tenga disponible GNOME Keyring, KWallet o un servicio compatible con Secret Service.
- Para que GitHub te genere notificaciones útiles, también debes estar suscrito, mencionado, asignado o participar en los hilos correspondientes.

Estado actual de notificaciones:

- la importación de notificaciones de GitHub sí está disponible
- todavía no hay notificaciones locales del sistema para tareas con fecha
- la base del producto ya deja claro el hueco funcional, pero esa integración queda para una segunda fase para no meter una implementación a medias en Electron

## Stack

- Electron
- SvelteKit + TypeScript
- Rust
- SQLite (`rusqlite`)
- UUID v7

## Requisitos

- Node.js 20+
- `pnpm`
- Rust toolchain estable
- Dependencias Linux típicas para Electron/AppImage

## Instalación

```bash
pnpm install
```

## Desarrollo

```bash
pnpm run electron:dev
```

Ese comando hace tres cosas:

- levanta Vite en `http://127.0.0.1:5173`
- compila `kitodo-server` en modo debug
- abre Electron usando un preload aislado y `sandbox`

## Build local

```bash
pnpm run appimage:build
```

Eso genera:

```bash
dist-electron/*.AppImage
```

El script:

- compila el frontend estático
- compila `kitodo-server` en release
- empaqueta con `electron-builder`

Note: on some Linux setups, the packaged AppImage may require additional launch flags related to GPU rendering or Wayland/X11 selection. See Troubleshooting below.

## Troubleshooting

### Known Linux AppImage Rendering Issue

On some Linux environments, the packaged Electron AppImage may fail to render correctly on launch.

Typical symptoms include:

- the app does not open
- a blank window appears
- the app crashes on launch
- the terminal shows repeated GPU process restart messages

This does not necessarily mean there is a functional bug in KitoDo itself. In the cases seen so far, the behavior looks more like an environment-specific rendering issue related to Electron/Chromium, Wayland, GPU acceleration, graphics drivers, or compositor compatibility.

This is more likely on `NVIDIA + Wayland` setups, but it can also happen on other Linux environments depending on driver and compositor support.

Representative log output may look like this:

```text
ERROR:gbm_pixmap_wayland.cc(...) Cannot create bo with format=RGBA_8888 and usage=SCANOUT
ERROR:gbm_pixmap_wayland.cc(...) Cannot create bo with format=RGBA_8888 and usage=GPU_READ
ERROR:shared_image_factory.cc(...) CreateSharedImage: could not create backing.
ERROR:shared_image_manager.cc(...) ProduceSkia: Trying to produce a Skia representation from a non-existent mailbox.
ERROR:gpu_service_impl.cc(...) Exiting GPU process because some drivers can't recover from errors.
ERROR:gpu_process_host.cc(...) GPU process exited unexpectedly
```

What this usually means in practice:

- Electron started, but Chromium could not initialize a stable GPU rendering path
- the failure is typically in the graphics stack, not in KitoDo's business logic
- Wayland, GPU acceleration, AppImage packaging, and driver support can all influence the result

#### Things to try

Run the AppImage with GPU disabled:

```bash
./KitoDo_x86_64.AppImage --disable-gpu
```

This disables GPU acceleration and forces software rendering.

Force X11 instead of Wayland:

```bash
./KitoDo_x86_64.AppImage --ozone-platform=x11
```

This tells Chromium/Electron to use the X11 backend instead of Wayland.

Or:

```bash
ELECTRON_OZONE_PLATFORM_HINT=x11 ./KitoDo_x86_64.AppImage
```

This sets an environment hint so Electron prefers X11 for that launch.

Optionally test explicit Wayland:

```bash
./KitoDo_x86_64.AppImage --enable-features=UseOzonePlatform --ozone-platform=wayland
```

This forces an explicit Wayland path, which can help confirm whether the issue is backend-specific.

Optional alternative:

```bash
./KitoDo_x86_64.AppImage --disable-gpu-compositing
```

This keeps the app on the GPU path where possible, but disables GPU compositing.

If the AppImage works with `--disable-gpu` or with an X11 launch mode, the issue is almost certainly related to the graphics stack rather than KitoDo's internal business logic.

#### When reporting this issue, please include

- Linux distribution and version
- desktop environment or compositor
- whether the session is `X11` or `Wayland`
- GPU model
- driver version
- the full launch command used
- terminal output

## Releases con GitHub Actions

El workflow [release-appimage.yml](/home/kitotsu/Programacion/Personal/KitoDo/.github/workflows/release-appimage.yml) se ejecuta en tags `v*` y adjunta un `.AppImage` al release.

Flujo recomendado:

```bash
./scripts/bump-version.sh 0.5.0
git add package.json pnpm-lock.yaml src-tauri/Cargo.toml README.md
git commit -m "chore: release v0.5.0"
git tag v0.5.0
git push origin main --follow-tags
```

## Optimización de Electron

La app quedó configurada para contener consumo y procesos extra:

- `contextIsolation: true`
- `sandbox: true`
- `nodeIntegration: false`
- `spellcheck: false`
- una sola ventana
- aperturas externas redirigidas al navegador del sistema en vez de abrir renderers nuevos
- backend pesado movido a un sidecar Rust fuera del proceso renderer

Eso evita el patrón más costoso de Electron: mezclar UI, acceso nativo y lógica de negocio dentro del renderer.

## Base de datos

Ruta de la DB:

- Linux: `~/.local/share/kitodo/kitodo.db`

Migraciones embebidas:

- `src-tauri/migrations/001_init.sql`
- `src-tauri/migrations/002_indexes.sql`
- `src-tauri/migrations/003_qol_sort_recurrence.sql`
- `src-tauri/migrations/004_qol_sort_recurrence_indexes.sql`
- `src-tauri/migrations/005_rebuild_tasks_fk.sql`
- `src-tauri/migrations/006_indexes_hardening.sql`
- `src-tauri/migrations/007_github_local_first.sql`
- `src-tauri/migrations/008_github_local_first_indexes.sql`

## API interna

La UI llama al sidecar local usando el mismo contrato lógico que antes exponía Tauri:

- `init_db`
- `quick_add`
- `list_inbox`
- `list_today`
- `list_overdue`
- `list_upcoming`
- `list_project_tasks`
- `list_projects`
- `list_labels`
- `get_project_sort_mode`
- `set_project_sort_mode`
- `reorder_project_tasks`
- `reset_project_manual_order`
- `toggle_task`
- `toggle_task_with_recurrence`
- `update_task_title`
- `update_task_priority`
- `update_task_due_date`
- `update_task_recurrence`
- `move_task_to_project`
- `soft_delete_task`
- `restore_task`
- `export_backup_json`
- `import_backup_json`
- `github_connect`
- `github_disconnect`
- `github_list_accounts`
- `github_get_settings`
- `github_set_settings`
- `github_list_repos`
- `github_add_repo_subscription`
- `github_remove_repo_subscription`
- `github_toggle_repo_subscription`
- `github_list_repo_subscriptions`
- `github_sync_now`
- `github_get_status`
- `github_list_external_items`
