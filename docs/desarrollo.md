# Desarrollo

## Requisitos

Rust estable, instalado con [rustup](https://rustup.rs), en Windows o Arch Linux.
En Arch Linux, el backend de credenciales requiere `libdbus` para compilar y un servicio de secretos en la sesión del usuario para recordar cuentas.

## Compilar y probar

```
cargo run -p icaro-cli
cargo test --workspace
```

Ejemplos de la interfaz:

```
cargo run -p icaro-ui --example galeria
cargo run -p icaro-ui --example shell
cargo run -p icaro-ui --example launcher
```

## Estructura

| Carpeta | Contenido |
| --- | --- |
| `crates/core` | Núcleo: manifiesto de versiones de Minecraft y verificación de descargas |
| `crates/cli` | Herramienta de línea de comandos |
| `crates/ui` | Sistema de diseño en iced: tema, tipografía, iconos y componentes |

## Estados del ejemplo launcher

El ejemplo `launcher` acepta variables de entorno para abrir una pantalla en un estado concreto:

| Variable | Efecto |
| --- | --- |
| `ICARO_DEMO=1` | Cuentas y tiendas simuladas, sin consultas a servicios reales |
| `ICARO_SECCION` | Sección inicial, de 1 a 7 |
| `ICARO_PIEDRA` | Tema claro |
| `ICARO_VACIO` | Todas las pantallas sin datos, para ver los estados vacíos |
| `ICARO_AJUSTE` | Página de Ajustes (7 es Acerca de, que incluye el Gabinete) |
| `ICARO_EDITOR` | Pestaña del editor de instancia (5 es Capturas) |
| `ICARO_BUSCAR` | Texto de búsqueda de la tienda de mods |

Las capturas del ejemplo son recortes de grabados; copiar y abrir carpeta están simulados.
El login y el catálogo se prueban según la guía de [inicio de sesión y catálogo](login-y-catalogo.md). Las variables de estados ficticios de cuenta requieren el modo de demostración.

## Licencia

GPL-3.0-only. El texto completo está en [LICENSE](../LICENSE).
