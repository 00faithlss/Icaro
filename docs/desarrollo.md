# Desarrollo

## Requisitos

Rust estable, instalado con [rustup](https://rustup.rs), en Windows o Arch Linux.

## Compilar y probar

```
cargo run -p icaro-cli
cargo test --workspace
```

## Estructura

| Carpeta | Contenido |
| --- | --- |
| `crates/core` | Núcleo: manifiesto de versiones de Minecraft y verificación de descargas |
| `crates/cli` | Herramienta de línea de comandos |
| `crates/ui` | Sistema de diseño en iced: tema, tipografía, iconos y componentes |

## Sistema de diseño

Los valores del tema (colores, espaciado, medidas, tipografía y duraciones) están en `crates/ui/src/tema.rs` y siguen los nombres del diseño. Las fuentes y los iconos están en `crates/ui/assets`.

- Fuentes: Archivo y Martian Mono, como archivos TTF estáticos (licencia OFL, texto junto a cada fuente).
- Iconos: Pixelarticons (licencia MIT), en tamaños múltiplos de 12 px.

## Criterios de interfaz

- Sin esquinas redondeadas ni sombras; la jerarquía se marca con el grosor de la regla y la inversión de tinta.
- Todo cambio visible se anima, con las duraciones definidas en el tema.
- Con "reducir movimiento" activo, las transiciones pasan a un fundido corto o al estado final.
- Textos en español latinoamericano neutro; números con punto de miles y coma decimal.

## Licencia

GPL-3.0-only. El texto completo está en [LICENSE](../LICENSE).
