# Flatpak packaging

This folder contains a full Flatpak scaffold for KitoDo.

## Files
- `io.github.KitotsuMolina.KitoDo.yml` (manifest)
- `io.github.KitotsuMolina.KitoDo.desktop`
- `io.github.KitotsuMolina.KitoDo.metainfo.xml`
- `icons/io.github.KitotsuMolina.KitoDo.svg`

## Build locally

Primero genera iconos raster desde `assets/kitodo-icon.png`:

```bash
./packaging/flatpak/prepare.sh
```

Esto crea:
- `packaging/flatpak/icons/hicolor/64x64/apps/io.github.KitotsuMolina.KitoDo.png`
- `packaging/flatpak/icons/hicolor/128x128/apps/io.github.KitotsuMolina.KitoDo.png`
- `packaging/flatpak/icons/hicolor/256x256/apps/io.github.KitotsuMolina.KitoDo.png`

Luego construye e instala:

```bash
flatpak-builder --user --install --force-clean build-flatpak packaging/flatpak/io.github.KitotsuMolina.KitoDo.yml
```

## Run

```bash
flatpak run io.github.KitotsuMolina.KitoDo
```

CLI inside Flatpak sandbox:

```bash
flatpak run --command=kitodo-cli io.github.KitotsuMolina.KitoDo today --json
```

## Optional `kitodo` alias on host shell

Flatpak does not install a global `/usr/bin/kitodo` directly on host.
Use an alias for a command-like workflow:

```bash
echo "alias kitodo='flatpak run io.github.KitotsuMolina.KitoDo'" >> ~/.bashrc
```

Then reload shell and run:

```bash
kitodo
```
