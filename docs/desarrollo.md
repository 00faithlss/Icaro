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

El diseño de referencia es `icaro-design-system.rar`. Las funcionalidades nuevas se conectan a sus componentes y a las vistas existentes. Las diferencias de pantallas que ya tienen diseño se registran en la [comparación con el system design](revision-system-design.md) para evaluar cambios antes de sustituirlas.

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
El login y el catálogo se prueban según la guía de [inicio de sesión y catálogo](login-y-catalogo.md). La instalación y ejecución de Vanilla se prueban según [Vanilla y Java](vanilla-y-java.md). Las variables de estados ficticios de cuenta requieren el modo de demostración.

## Capturas aisladas del diseño

El ejemplo puede capturar los widgets reales con datos de prueba y cerrar esa ventana, sin consultar Microsoft o Modrinth ni guardar cambios en las cuentas e instancias del usuario. `ICARO_VISTA_DISENO` admite `asistente`, `exportar`, `mods`, `seleccion`, `tienda`, `ficha`, `pagina`, `archivos`, `archivos-renombrar`, `juego-descargas`, `juego-consola`, `juego-consola-vacia`, `juego-fallo`, `juego-editor`, `juego-instalando`, `juego-ejecutando` y las fases `juego-umbral-entrada`, `juego-umbral-arco`, `juego-umbral-registro`, `juego-umbral-retirada`, `juego-umbral-fallo`. `juego-capturas` y `juego-capturas-visor` leen una PNG de prueba indicada en `ICARO_QA_CAPTURA`, sin modificarla ni refrescarla desde el inventario real. El explorador usa una carpeta de muestra dentro del directorio temporal. Las escenas del juego no ejecutan Java ni usan una cuenta.

```powershell
cargo build -j 1 -p icaro-ui --example launcher
$env:ICARO_CAPTURA_DISENO = Join-Path $env:TEMP 'icaro-tienda.png'
$env:ICARO_VISTA_DISENO = 'tienda'
# $env:ICARO_PIEDRA = '1' para el tema claro.
.\target\debug\examples\launcher.exe
Remove-Item Env:ICARO_CAPTURA_DISENO, Env:ICARO_VISTA_DISENO
```

Si la compilación de pruebas agota la memoria, ejecuta los objetivos por separado con `-j 1`: núcleo, biblioteca de interfaz y ejemplo launcher.

## Licencia

GPL-3.0-only. El texto completo está en [LICENSE](../LICENSE).
