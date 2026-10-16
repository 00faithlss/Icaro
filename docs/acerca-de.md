# Acerca de Icaro Launcher

Icaro Launcher es un launcher de Minecraft escrito en Rust, ligero y optimizado, para Windows y Arch Linux.

La pantalla **Ajustes → Acerca de** muestra la versión compilada de `icaro-ui`, heredada de `workspace.package.version` en `Cargo.toml`. No mantener números de versión escritos directamente en la interfaz; el actualizador usa la misma fuente. Los ejecutables ya distribuidos requieren recompilarse y publicarse para incorporar esta corrección.

## Funciones previstas

- Inicio de sesión con cuenta de Microsoft.
- Instalación de distintas versiones de Minecraft y de loaders como Fabric, NeoForge y Quilt.
- Sincronización de mods con un servidor.
- Conexión directa a un servidor desde el launcher.
- Consola de registros y ajuste de memoria.

## Servidores y red privada

La lista real permite guardar servidores, vincular una instancia, reordenar por arrastre, consultar estado/SRV y preparar el lanzamiento directo. RedPrivada detecta ZeroTier y permite solicitar pertenencia con su CLI. SincronizarMods consulta el servicio del anfitrión, muestra diferencias y aplica archivos exactos de Modrinth conservando archivos locales. Faltan automatización de instalación/permisos de ZeroTier y validación con dos equipos. Consulta [Servidores y ZeroTier](servidores.md) y [sincronización](sincronizacion-servidores.md).
