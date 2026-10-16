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

## Estado y fuentes de referencia

Base de trabajo: `0.2.2`, línea de mantenimiento `v0.2.x`. Consultar `git status` y las referencias remotas antes de asumir rama, publicación o estado de CI. El hotfix `e8c6614` fue subido y el workflow `38837519199` terminó correctamente; se verificó la release `v0.2.2-build.38837519199` como más reciente el 15-10-2026. Revalidar este estado al publicar de nuevo. La versión visible y el actualizador deben usar `env!("CARGO_PKG_VERSION")`, heredado del workspace; nunca escribir `0.1.0` u otra versión fija en la interfaz.

- [Notas de 0.2.2](docs/releases/0.2.2.md): recuperación del actualizador, bloqueo de ayudantes simultáneos, respaldo de conflictos, identificación de duplicados por ID/hash, importación múltiple de JAR y corrección del resumen de modpacks.
- [Pendientes funcionales](docs/pendientes-implementacion.md), [roadmap visual](docs/roadmap-system-design.md) e [integración real](docs/integracion-real.md): fuentes de alcance; una pantalla conectada o una prueba local no acreditan todo el recorrido.
- Conservar ZonaSoltar, iconos, grabados, tipografías, temas Tinta/Piedra y estados del diseño. Comparar las referencias antes de sustituir componentes o assets.

## Investigación y optimización pendiente

Leer la [investigación del 15-10-2026](docs/investigaciones/rendimiento-sincronizacion-2026-10-15.md), sus [datos](docs/investigaciones/datos/rendimiento-sincronizacion-2026-10-15.json) y el banco aislado `scripts/rendimiento_sincronizacion/` antes de modificar sincronización, hashing o descargas.

La investigación detectó hashes sin optimizar en debug, barridos completos repetidos y copia de todo `mods` incluso para retirar pocos archivos. La diferencia de SHA-512 medida entre debug/release es de un componente; no demuestra una aceleración equivalente del launcher. Las propuestas siguientes **no están implementadas**:

1. **P0:** usar release para medir experiencia real. Aplicado en `feat/sync-incremental`: `sha2`/`sha1` optimizados en desarrollo, una sola lectura de hashes por revisión con detección de cambios por metadatos (si difieren, se comparan hashes completos), lectura local en `spawn_blocking` con cancelación y límite de 90 s solo para la red. Pendiente: retiradas por archivo con journal durable, rollback y progreso en lugar de copiar `mods` completo.
2. **P1:** inventario y descriptores incrementales, reutilización del plan validado y caché de contenido verificado. Descargar por streaming con concurrencia y memoria acotadas.
3. **P1 API:** conservar lotes/caché/pausa compartida de Modrinth; deduplicar por clave sin retener el mutex global durante la red. Añadir presupuesto preventivo según `Limit`, `Remaining`, `Reset` y `Retry-After`, backoff y jitter. El límite observado/documentado en la investigación fue 300 solicitudes/minuto por IP; verificar reglas y cabeceras actuales, sin tratarlo como constante universal ni aplicarlo al CDN/anfitrión.
4. **P2:** host incremental, manifiesto materializado con ETag/304, conexiones persistentes, streaming y cola justa. Reanudar parciales con Range/If-Range validando respuesta y hash final.

No confiar solo en tamaño/mtime para integridad ni usar enlaces duros como copias independientes de JAR mutables. Sincronizar reproduce los archivos exactos del anfitrión; no instala automáticamente la última versión del catálogo. Conservar otros mods personales, archivos desactivados, dependencias y respaldos verificables.

Medir mediana/p95, bytes leídos/escritos, llamadas API, caché, espera por cuota y RAM. Cubrir cero cambios, cinco retiradas, actualización, descargas múltiples, duplicados, cambios externos, caché corrupta, 429, cortes, disco lleno, cancelación y recuperación tras cierre. Los objetivos de 1–2 segundos de la investigación son metas pendientes, no garantías.

```sh
cargo run --locked --offline --release --manifest-path scripts/rendimiento_sincronizacion/Cargo.toml
```

El benchmark de hash no usa la instancia. Su modo `io` requiere espacio para una copia y solo debe ejecutarse con el corpus estable; consultar primero sus instrucciones.

## Funciones y validaciones todavía pendientes

- Instalar/ejecutar Quilt, Forge y NeoForge con perfiles oficiales, Java, migración y recuperación; sus catálogos visibles no significan soporte completo.
- Importar ZIP de CurseForge que solo referencian archivos por ID requiere una integración válida; completar exportación MRPACK, descubrimiento de otros launchers y revisión de archivos sin origen verificado. Importación local MRPACK/ZIP completo Vanilla/Fabric e importación de JAR propios ya existen; no duplicarlas.
- Completar opciones General/Java, mundos y respaldos, orden de resourcepacks, otros motores de shaders, recuperación/papeleras, historial persistente de descargas, cola unificada y tiempo jugado.
- Completar instalación/permisos, salida y miembros de ZeroTier, traslado de raíz de datos y bandeja; validar cuenta/skins con cuenta real y funciones de varias cuentas/capes según su guía.
- Validar recorridos nativos en Arch/Linux, sincronización entre equipos y servidor real, reconexión al juego y estados de foco/teclado/escala/ventana pequeña en ambos temas.
- Probar actualización Windows completa, instalación limpia, reinstalación y desinstalación en equipo de prueba. Las pruebas de PowerShell y del instalador simulado no sustituyen esa validación. Un cliente 0.2.0 ejecuta su actualizador antiguo: no garantizar que incorpore retroactivamente el arreglo de 0.2.2.

## Publicación y criterios de entrega

Seguir [el contrato del actualizador](docs/actualizaciones-launcher.md) y `.github/workflows/windows-package.yml`. El push de una rama `v*` puede compilar y publicar automáticamente: comprobar su alcance antes de hacerlo. El tag fuente `v<versión>` y la release descargable `v<versión>-build.<compilación>` cumplen funciones distintas; conservar la correspondencia entre manifiesto, build, hashes y assets. No renombrar una release rompiendo ese contrato.

Mantener `Cargo.toml`, los paquetes locales de `Cargo.lock` y `docs/releases/<versión>.md/.json` alineados. No modificar versiones de dependencias o del benchmark aislado como parte de un bump del launcher. Verificar remoto, CI, release más reciente y artefactos independientemente; actualizar `main` por sí solo no distribuye el programa. Commit, push, tag, release y merge requieren el alcance solicitado por el usuario; no inferir autorización para integrar `develop`/`main` de un push a mantenimiento.
