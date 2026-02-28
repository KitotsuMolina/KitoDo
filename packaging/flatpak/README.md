# Flatpak packaging

This folder contains a full Flatpak scaffold for KitoDo.

## Files
- `io.github.KitotsuMolina.KitoDo.yml` (manifest)
- `io.github.KitotsuMolina.KitoDo.desktop`
- `io.github.KitotsuMolina.KitoDo.metainfo.xml`
- `icons/io.github.KitotsuMolina.KitoDo.svg`

## Build locally

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
