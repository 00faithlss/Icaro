# Repository Guidelines

## Project Structure & Module Organization

This Rust 2021 workspace contains:

- `crates/core/src/`: authentication, instances, Minecraft/Java installation, mod synchronization, storage, and updates. Integration tests live in `crates/core/tests/`; utilities and the dedicated sync host live in `crates/core/examples/`.
- `crates/ui/src/`: reusable iced components, themes, and rendering. The desktop application is `crates/ui/examples/launcher.rs`; its service integrations are in `launcher_aux/`. Fonts, images, and icons live in `crates/ui/assets/`.
- `crates/cli/`: fetches and verifies Minecraft metadata.

Use `docs/` for guides, `docs/releases/` for release notes, `scripts/` for maintenance, and `packaging/` for platform launchers. Generated output belongs in ignored `target/`.

## Build, Test, and Development Commands

Run from the repository root with stable Rust:

```sh
cargo build --locked --workspace
cargo run --locked -p icaro-ui --example launcher
cargo run --locked -p icaro-cli
cargo build --locked --release -p icaro-core --example sync_host
```

The CLI accesses Mojang; the final command builds the standalone host. Set `ICARO_DEMO=1` for simulated accounts and catalogs. Linux dependencies are documented in `docs/desarrollo.md`.

Match Windows CI validation:

```sh
cargo test --locked -j1 -p icaro-core --lib --test actualizacion_local
cargo test --locked -j1 -p icaro-ui --lib --example launcher -- --test-threads=1
git diff --check
```

## Coding Style & Naming Conventions

Use four-space indentation and standard Rust naming: `snake_case` functions/modules, `PascalCase` types, and `SCREAMING_SNAKE_CASE` constants. Preserve existing Spanish domain names and comments. Format changed Rust code with rustfmt; use `cargo clippy --workspace --all-targets` for lint feedback. Avoid unrelated formatting changes. Keep filesystem/network work in core services and avoid blocking iced updates. Preserve the existing design system and asset attribution.

## Testing Guidelines

Use `#[test]` and `#[tokio::test]`, generally in colocated `mod tests`. Name tests after observable behavior. Add regressions for meaningful fixes, particularly integrity, cancellation, recovery, and file preservation. No coverage threshold is configured. External-service tests are typically ignored; run specific ones with `-- --ignored`. Distinguish fixture success from actual gameplay and platform validation.

## Commit & Pull Request Guidelines

Use Conventional Commits with Spanish descriptions, such as `fix(sync): conservar mods personales`. Keep changes focused. PRs should explain the problem, resulting behavior, validation, and remaining limitations; link relevant issues and include screenshots for UI changes. Update affected guides and release notes. Keep Cargo versions aligned with version branches/tags. Commit, push, and merge only when requested.

## Security & Data Preservation

Never commit credentials, tokens, or personal instance data. Preserve user mods, worlds, and settings; retain hash verification and atomic publication when changing installers or synchronization.
