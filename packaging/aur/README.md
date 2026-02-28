# AUR scaffold (`kitodo-git`)

This is a local scaffold for publishing a `-git` package later.

## Files
- `PKGBUILD`
- `.SRCINFO`
- `kitodo.desktop`

## Notes
- Update `url` and maintainer fields before publishing.
- Regenerate `.SRCINFO` after modifying `PKGBUILD`:

```bash
cd packaging/aur
makepkg --printsrcinfo > .SRCINFO
```

## Build behavior
The PKGBUILD currently runs:
1. `pnpm install --frozen-lockfile`
2. `pnpm build`
3. `cargo tauri build --manifest-path src-tauri/Cargo.toml`
