# Desarrollo

## Requisitos

Rust estable, instalado con [rustup](https://rustup.rs), en Windows o Arch Linux.

## Compilar y probar

```
cargo run -p icaro-cli
cargo test --workspace
```

Ejemplos de la interfaz:

```
cargo run -p icaro-ui --example galeria
cargo run -p icaro-ui --example shell
```

## Estructura

| Carpeta | Contenido |
| --- | --- |
| `crates/core` | Núcleo: manifiesto de versiones de Minecraft y verificación de descargas |
| `crates/cli` | Herramienta de línea de comandos |
| `crates/ui` | Sistema de diseño en iced: tema, tipografía, iconos y componentes |

## Licencia

GPL-3.0-only. El texto completo está en [LICENSE](../LICENSE).
