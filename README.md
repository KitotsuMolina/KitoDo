# KitoDo v0.6.6

KitoDo es una app de tareas estilo launcher para Linux. El runtime actual usa:

- `SvelteKit + TypeScript` para la UI
- `Electron` como shell de escritorio
- `Rust + SQLite` como backend local (`kitodo-server`)

La app mantiene una arquitectura de bajo acoplamiento: Electron solo hospeda la ventana y actúa como bridge seguro; toda la lógica de datos y GitHub vive en Rust.

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
