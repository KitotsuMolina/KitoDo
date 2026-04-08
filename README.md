# KitoDo v0.5.0

KitoDo es una app de tareas estilo launcher para Hyprland, construida con arquitectura Proper Tauri:
- Frontend SvelteKit + TypeScript solo para UI
- Backend Rust para lógica + SQLite
- Offline-first, sin sync

## Stack

- Tauri v2
- Rust
- SvelteKit + TypeScript
- SQLite (`rusqlite`)
- UUID v7
- Migrations embebidas

## Setup

Requisitos:
- Node.js 20+
- Rust toolchain estable
- Dependencias de sistema para Tauri (WebKitGTK en Linux)
- Tauri CLI v2

Instalar dependencias frontend:

```bash
pnpm install
```

## Instalación local desde este repositorio

### Opción A: AppImage local (recomendado)

Instalación rápida (recomendado):

```bash
./scripts/install-local.sh
```

Este script instala:
- `KitoDo.AppImage` en `~/.local/share/kitodo/`
- `kitodo` y `kitodo-cli` en `~/.local/bin`
- icono local desde `assets/kitodo-icon.png`
- iconos derivados `64x64`, `128x128` y `256x256` (si `magick` está disponible)
- launcher desktop en `~/.local/share/applications/kitodo.desktop`

Instalación manual:

1. Clona el repo y entra al proyecto:

```bash
git clone https://github.com/KitotsuMolina/KitoDo.git
cd KitoDo
```

2. Instala dependencias frontend:

```bash
pnpm install
```

3. Genera AppImage:

```bash
./scripts/build-appimage.sh
```

4. Instala localmente:

```bash
./scripts/install-local.sh
```

5. Ejecuta:

```bash
kitodo
```

## Backup e importación

KitoDo permite exportar e importar tus tareas desde la UI con un backup JSON versionado.

- `Backup` en el header: exporta un `.json` con proyectos y tareas activas/completadas
- importación por archivo o pegando JSON
- el import hace merge por `id`, sin duplicar tareas existentes

### Opción B: instalación local con Flatpak

1. Clona el repo y entra al proyecto:

```bash
git clone https://github.com/KitotsuMolina/KitoDo.git
cd KitoDo
```

2. Construye e instala el Flatpak local:

```bash
flatpak-builder --user --install --force-clean build-flatpak packaging/flatpak/io.github.KitotsuMolina.KitoDo.yml
```

3. Ejecuta:

```bash
flatpak run io.github.KitotsuMolina.KitoDo
```

## Desinstalación local (host)

Si instalaste con `./scripts/install-local.sh`, puedes remover binarios con:

```bash
./scripts/uninstall-local.sh
```

Instalar Tauri CLI (elige una):

```bash
# Opción global (cargo)
cargo install tauri-cli --version '^2.0.0' --locked
```

```bash
# Opción local (recomendada para el proyecto)
pnpm install
```

## Desarrollo

En una terminal:

```bash
pnpm dev
```

En otra terminal (raíz del proyecto):

```bash
pnpm run tauri:dev
```

> Si instalaste el CLI global, también puedes usar `cargo tauri dev`.
> La app usa `http://localhost:5173` en desarrollo. Si ese puerto está ocupado, libera el proceso previo antes de iniciar.
> En Hyprland/Wayland, si hay crash de protocolo GTK/WebKit, usa:
>
> ```bash
> pnpm run tauri:dev:x11
> ```

## Build

```bash
./scripts/build-appimage.sh
```

El AppImage queda en:

```bash
src-tauri/target/release/bundle/appimage/
```

## Releases automáticos con GitHub Actions

Se añadió un workflow en [release-appimage.yml](/home/kitotsu/Programacion/Personal/KitoDo/.github/workflows/release-appimage.yml) que:
- se ejecuta al hacer push de tags `v*`
- compila el `AppImage` en Linux
- crea un GitHub Release
- adjunta el `AppImage` al release

Flujo recomendado:

```bash
./scripts/bump-version.sh 0.5.0
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json README.md
git commit -m "chore: release v0.5.0"
git tag v0.5.0
git push origin main --follow-tags
```

También puedes dispararlo manualmente desde GitHub Actions, pero el flujo principal queda basado en tags.

## Flatpak

Flatpak queda como alternativa opcional. El flujo principal recomendado es AppImage.

Scaffold listo en:
- `packaging/flatpak/io.github.KitotsuMolina.KitoDo.yml`
- `packaging/flatpak/io.github.KitotsuMolina.KitoDo.desktop`
- `packaging/flatpak/io.github.KitotsuMolina.KitoDo.metainfo.xml`

Build + install local:

```bash
./packaging/flatpak/prepare.sh
flatpak-builder --user --install --force-clean build-flatpak packaging/flatpak/io.github.KitotsuMolina.KitoDo.yml
```

Ejecutar app:

```bash
flatpak run io.github.KitotsuMolina.KitoDo
```

Comando CLI en sandbox:

```bash
flatpak run --command=kitodo-cli io.github.KitotsuMolina.KitoDo today --json
```

Si quieres lanzar con `kitodo` desde tu shell host, usa alias:

```bash
echo "alias kitodo='flatpak run io.github.KitotsuMolina.KitoDo'" >> ~/.bashrc
```

## Base de datos

Ruta de DB con `app_data_dir`:
- Linux: `~/.local/share/kitodo/kitodo.db`

Migrations embebidas en:
- `src-tauri/migrations/001_init.sql`
- `src-tauri/migrations/002_indexes.sql`
- `src-tauri/migrations/003_qol_sort_recurrence.sql`
- `src-tauri/migrations/004_qol_sort_recurrence_indexes.sql`
- `src-tauri/migrations/005_rebuild_tasks_fk.sql`
- `src-tauri/migrations/006_indexes_hardening.sql`

Hardening aplicado:
- Backup automático antes de migrations pendientes: `kitodo.db.bak-YYYYMMDD-HHMMSS`
- Rotación automática de backups: se conservan los últimos 10
- FKs activas por conexión (`PRAGMA foreign_keys = ON`)

## API interna (Tauri commands)

- `init_db()`
- `quick_add(input)`
- `list_inbox(show_done)`
- `list_today(show_done)`
- `list_overdue(show_done)`
- `list_upcoming(days, show_done)`
- `list_project_tasks(project_id, show_done)`
- `list_projects()`
- `list_labels()`
- `get_project_sort_mode(project_id)`
- `set_project_sort_mode(project_id, mode)`
- `reorder_project_tasks(project_id, ordered_task_ids)`
- `reset_project_manual_order(project_id)`
- `toggle_task(id)`
- `toggle_task_with_recurrence(id)`
- `update_task_title(id, title)`
- `update_task_priority(id, priority)`
- `update_task_due_date(id, due_date_or_null)`
- `update_task_recurrence(id, recurrence_or_null)`
- `move_task_to_project(id, project_id_or_null, project_name_to_create_or_null)`
- `soft_delete_task(id)`
- `restore_task(id)`

## UX actual

- Metadatos visibles por tarea: `@proyecto`, `#labels`, `Due: YYYY-MM-DD`
- Tabs con contadores en vivo: `Inbox`, `Hoy`, `Próximos`
- Búsqueda local por título/proyecto/etiquetas (debounce ~180ms)
- Sección `Vencidas` (Overdue) en `Hoy` y `Próximos`
- Undo al eliminar (snackbar 5s con `Deshacer`)
- Menú contextual `⋯` por tarea:
  - cambiar prioridad (`P1..P4`)
  - cambiar fecha (`today`, `tomorrow`, manual o quitar)
  - cambiar recurrencia
  - mover a proyecto existente
  - crear proyecto y mover
  - eliminar (soft delete)
- Micro-animaciones sutiles en alta/baja/toggle done
- Modo expandido con sidebar animado (vistas, proyectos, etiquetas, filtros)
- Vista por proyecto con orden configurable:
  - `Auto`
  - `Manual` (reordenamiento drag & drop cuando no hay búsqueda)
  - `Volver a Auto` + `Actualizar/Recalcular`
- Mini métricas + barra de progreso diario

## Atajos

- `Esc`: cierra la ventana (o limpia búsqueda si estás en buscador)
- `Ctrl+K`: focus en Quick Add
- `Ctrl+F`: focus en búsqueda
- `Ctrl+1`: Inbox
- `Ctrl+2`: Hoy
- `Ctrl+3`: Próximos
- `J/K`: mover selección en lista
- `X`: toggle done de tarea seleccionada
- `Enter`: editar tarea seleccionada
- `Delete/Backspace`: eliminar tarea seleccionada (con Undo)
- `F`: toggle compacto/expandido
- `F11`: maximizar/restaurar ventana

## Quick Add

Tokens soportados:
- `@Proyecto` (máximo 1)
- `#tag` (múltiples)
- `p1`..`p4` (default `p4`)
- `due today`
- `due tomorrow`
- `due YYYY-MM-DD`
- `every day|week|month`
- `every mon|tue|wed|thu|fri|sat|sun`
- `every 2d|3w|1m` (intervalos)

Ejemplo:

```text
Comprar leche @Personal #compras p2 due tomorrow every week
```

## Notas actuales

Exclusiones intencionales:
- Sync
- RRULE completa / recurrencias avanzadas
- Subtareas
- Kanban
- Colaboración
- Notificaciones complejas

## GitHub Integration (Local-First)

KitoDo integra GitHub sin servidor ni webhooks: todo corre local con polling incremental.

### Qué importa

- PRs abiertas donde te pidieron review
- Issues abiertas asignadas a ti
- Notifications por repo (best-effort)

Cada item externo se deduplica por clave única y se mapea a una tarea local en proyecto destino (`GitHub Inbox` por defecto).

### Token y seguridad

- El token se guarda en **keyring del sistema** (no en SQLite).
- Recomendado PAT con scopes mínimos:
  - `repo` (repos privados, PR/issues)
  - `read:user`
  - `notifications` (si usarás import de notifications)

Nota: el endpoint de notifications puede requerir **PAT classic**. Si no es compatible, se muestra aviso y sync continúa para PR/issues.

### Configuración en UI

En modo expandido, abre sidebar y sección **GitHub**:
- Conectar token
- Elegir cuenta (si hay varias)
- Toggle Auto sync
- Intervalo (1m, 5m, 10m, 30m)
- Toggles de fuentes (PR review, issues assigned, notifications)
- Proyecto destino
- Suscripciones por repo (`owner/repo`)
- `Sync now`

### Comportamiento de mapeo

- Tarea creada/actualizada por item externo (1:1 por `external_item`)
- Labels automáticas: `#github`, `#pr|#issue|#notification`, `#review` (cuando aplica), `#repo-owner-repo`
- Prioridad heurística:
  - P1: título con `urgent|security|hotfix`
  - P2: notifications/review requested
  - P3 default
- Si PR/Issue cierra: tarea local pasa a `done`
- Si editas título manualmente, no se sobreescribe en sync siguientes

## CLI (Hyprland / Waybar)

El binario principal soporta modo CLI cuando se invoca con subcomandos.

Ejemplos:

```bash
kitodo today --json
kitodo overdue --json
kitodo inbox --json
kitodo add \"Comprar leche @Personal #compras due tomorrow p2 every week\" --json
```

Flags:
- `--all`: incluye tareas `done` (por defecto solo `todo`)
- `--json`: salida JSON estable

Formato JSON:
- Listados: `{ \"tasks\": [TaskDTO, ...] }`
- Add: `{ \"task\": TaskDTO }`

## AUR scaffold

Se añadió scaffold en:
- `packaging/aur/PKGBUILD`
- `packaging/aur/.SRCINFO`
- `packaging/aur/kitodo.desktop`
- `packaging/aur/README.md`

Antes de publicar:
1. Reemplazar `url` y datos de maintainer en `PKGBUILD`.
2. Regenerar `.SRCINFO` con `makepkg --printsrcinfo > .SRCINFO`.
