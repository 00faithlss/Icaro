# Acerca de Icaro Launcher

Icaro Launcher es un launcher de Minecraft escrito en Rust, ligero y optimizado, para Windows y Arch Linux.

La pantalla **Ajustes → Acerca de** muestra la versión compilada de `icaro-ui`, heredada de `workspace.package.version` en `Cargo.toml`. No mantener números de versión escritos directamente en la interfaz; el actualizador usa la misma fuente. Los ejecutables ya distribuidos requieren recompilarse y publicarse para incorporar esta corrección.

El botón **Código fuente en GitHub**, con el icono de enlace externo, abre el repositorio en el navegador del sistema. La dirección se toma de `workspace.package.repository`, heredada por `icaro-ui`, y se valida antes de abrirla como el resto de enlaces del launcher.

## Funciones previstas

- Inicio de sesión con cuenta de Microsoft.
- Instalación de distintas versiones de Minecraft y de loaders como Fabric, NeoForge y Quilt.
- Sincronización de mods con un servidor.
- Conexión directa a un servidor desde el launcher.
- Consola de registros y ajuste de memoria.

## Servidores y red privada

La lista real permite guardar servidores, vincular una instancia, reordenar por arrastre, consultar estado/SRV y preparar el lanzamiento directo. RedPrivada detecta ZeroTier y permite solicitar pertenencia con su CLI. SincronizarMods consulta el servicio del anfitrión, muestra diferencias y aplica archivos exactos de Modrinth conservando archivos locales. Solo mueve los archivos afectados mediante un diario recuperable y deja cada paso en `logs/sincronizacion.log`. Faltan automatización de instalación/permisos de ZeroTier y validación con dos equipos. Consulta [Servidores y ZeroTier](servidores.md) y [sincronización](sincronizacion-servidores.md).
