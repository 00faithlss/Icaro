# Changelog

## [0.2.0] - 2026-10-14

Publicación consolidada de las mejoras de servidores y las correcciones probadas durante su preparación. La rama y el tag de esta entrega son `v0.2.0`.

### Añadido

- Servicio independiente para sincronizar los mods de un servidor dedicado, con compilación para Linux y uso sin instalar el launcher en el anfitrión.
- Distribución de JAR privados o ausentes en Modrinth desde el anfitrión, verificando tamaño y SHA-512.
- Preferencias de Minecraft compartidas entre instancias, con filtrado de teclas de mods ausentes.
- Caché persistente de metadatos de Modrinth y consultas de versiones en lotes de hasta 50 IDs.

### Corregido

- Rutas extendidas de Windows en los argumentos de Java y el classpath, que impedían encontrar `KnotClient` de Fabric.
- Recuperación de dependencias obligatorias omitidas por índices antiguos, incluidas bibliotecas como CodxLib y Lithostitched cuando sus JAR admiten cliente.
- Dependencias de cliente no declaradas de Litemoretica 1.2.5 para Minecraft 1.21.1: la sincronización añade Litematica y resuelve MaLiLib automáticamente.
- Conservación de mods personales del cliente, tanto del catálogo como de JAR manuales y desactivados, durante la sincronización.
- Desactivación persistente de la sincronización al iniciar después de completarla correctamente; los fallos conservan la preferencia para reintentar.
- Selección manual de Java más reciente que el mínimo del perfil, como Java 25 para Minecraft que recomienda Java 21.
- Navegación bloqueada durante la consulta de mods y lentitud al volver a las instancias por la revisión repetida de miles de archivos.
- Lectura de descriptores Fabric incompletos y publicación de mods universales con metadatos incompletos, conservando los JAR originales con avisos.
- Resolución de dependencias y pins del catálogo sin sustituir silenciosamente el conjunto instalado del servidor.
- Detección de redes conectadas desde ZeroTier oficial y actualización de instancias vinculadas a servidores.

### Rendimiento y recuperación

- Descarga solo los archivos ausentes o cambiados; repetir una sincronización al día evita volver a copiar el conjunto completo.
- Reutiliza las consultas vigentes entre aperturas y conserva respuestas parciales correctas ante fallos posteriores.
- Respeta los límites de Modrinth y sus tiempos de espera ante HTTP 429.
- Permite salir de la consulta o navegar mientras termina la aplicación del plan, sin abrir Minecraft desde otra pantalla.

### Validación

- Pruebas de sincronización, dependencias, conservación de archivos, caché, Java y navegación.
- Integración con Litemoretica, Litematica y MaLiLib reales en una instancia temporal vacía; una segunda sincronización no propone cambios.
- El usuario confirmó el funcionamiento del launcher y el inicio del juego con la corrección de dependencias.
- La publicación de instaladores y los artefactos Linux se comprueban por separado en GitHub Actions. Los loaders adicionales y el resto del roadmap siguen pendientes.

## [0.1.0]

Primera entrega pública para Windows: instancias Vanilla y Fabric, catálogo de contenido, cuentas, consola, capturas y actualización del launcher. Consulta las [notas completas](docs/releases/0.1.0.md).

[0.2.0]: https://github.com/00faithlss/Icaro-launcher/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/00faithlss/Icaro-launcher/tree/v0.1.0
