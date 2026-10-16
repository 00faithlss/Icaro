# Changelog

## [0.2.3] - 2026-10-16

- Acerca de muestra la versión compilada del workspace en lugar del texto fijo `0.1.0`.
- Acerca de añade **Código fuente en GitHub**, con icono de enlace externo, que abre el repositorio declarado en `Cargo.toml`.
- Amplía la guía de colaboradores con el estado del hotfix, los pendientes y las prioridades de la investigación de sincronización.
- La sincronización calcula los hashes de `mods` una vez por revisión en lugar de dos, y al aplicar solo vuelve a leerlos si cambiaron los metadatos de algún archivo. Pulsar Jugar con el conjunto al día pasa de tres lecturas completas a una.
- La lectura local de mods se ejecuta fuera del executor asíncrono y se detiene al cancelar. El límite de 90 segundos de la revisión ya no incluye el disco, por lo que una carpeta grande no se presenta como un fallo de Modrinth.
- Las compilaciones de desarrollo optimizan `sha2` y `sha1`; las mediciones de sincronización en debug dejan de estar dominadas por el hashing sin optimizar.
- Aplicar una sincronización ya no copia toda la carpeta `mods`: descarga y verifica solo lo nuevo, y mueve únicamente los archivos afectados mediante un diario que se deshace si falla o si el launcher se cierra a mitad. Los mods personales no se copian ni se recrean.
- La barra de progreso de la sincronización cuenta los cambios del plan en lugar de todos los mods del índice.
- Añade `logs/sincronizacion.log`, con rotación, para diagnosticar sincronizaciones: índice del anfitrión, revisión, plan, descargas, commit, rollback, recuperación y errores con su fase y duración. No incluye rutas absolutas ni datos de la cuenta.

Validación: pruebas del núcleo y de la interfaz y un banco sintético de 500 MiB; el usuario confirmó la mejora en Windows. La validación en HDD y Linux sigue pendiente. Consulta las [notas de 0.2.3](docs/releases/0.2.3.md).

## [0.2.2] - 2026-10-15

- Identifica versiones anteriores y copias duplicadas de un mod por su ID interno y sus hashes. La sincronización respalda y retira las copias activas del mismo mod del servidor, conserva otros mods personales y verifica que no queden dos versiones activas de un mod requerido.

- Reutiliza el paquete de actualización ya descargado tras reabrir el launcher o reintentar una instalación fallida; vuelve a verificar tamaño y SHA-256 y descarta archivos parciales o alterados.

- Mantiene bloqueada la actualización hasta cerrar el launcher, evita ayudantes simultáneos y normaliza las rutas para PowerShell/Inno Setup. Comprueba la versión instalada y muestra el motivo de un fallo.
- Distribuye el espacio del resumen de importación para que una lista larga de carpetas no aplaste las etiquetas.
- Muestra el plan de sincronización aunque un JAR propio choque con el del anfitrión; exige autorizar su reemplazo y conserva una copia verificable fuera de `mods`.
- Añade **Importar .jar** en la lista de mods: selección múltiple, publicación atómica y conservación de nombres ya ocupados.

Incluye una [investigación de rendimiento](docs/investigaciones/rendimiento-sincronizacion-2026-10-15.md) con mediciones y un banco de pruebas reproducible. Las optimizaciones propuestas siguen pendientes. Consulta las [notas de 0.2.2](docs/releases/0.2.2.md) para la validación y sus límites. Esta preparación local todavía requiere publicar los artefactos para distribuir la actualización.

## [0.2.1] - 2026-10-15

- Añade selección de MRPACK/ZIP por ruta y con Examinar, y adapta ImportarModpack al diseño entregado: grabado, iconos, fases de lectura, resumen, nombre/grupo y recuperación de errores.
- Conecta el componente ZonaSoltar del diseño a la importación local: clic, arrastre, revisión y protección frente a archivos adicionales durante una operación.

- Corrige la rueda del selector de versiones dentro del asistente de instancias.
- El ejecutable del launcher para Windows abre la interfaz sin una consola de fondo.
- Añade importación local de MRPACK y ZIP completos Vanilla/Fabric en instancias nuevas, con revisión, extracción por bloques y recuperación desde Descargas.
- Comparte caché y pausa de Modrinth entre tienda, cesta e instalación; agrupa comprobaciones de actualización por hash y fichas por proyecto para evitar repetir peticiones al añadir otro mod.
- Adapta la publicación a la rama de mantenimiento `v0.2.x`, conserva el tag `v0.2.1` para el código y evita volver a compilar/publicar el mismo contenido al integrarlo en otras ramas.

Validación: pruebas del núcleo y de la interfaz, importación ZIP64 sintética de más de 2 GiB, conservación de archivos y capturas en Tinta/Piedra. El usuario confirmó la importación de su instancia real. Los ZIP que solo contienen referencias de CurseForge y los instaladores de Quilt/Forge/NeoForge siguen pendientes; consulta las [notas completas](docs/releases/0.2.1.md).

## [0.2.0] - 2026-10-14

Publicación consolidada de las mejoras de servidores y las correcciones probadas durante su preparación. Su tag es `v0.2.0`; la rama de mantenimiento es `v0.2.x`.

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

[0.2.2]: docs/releases/0.2.2.md
[0.2.1]: https://github.com/00faithlss/Icaro-launcher/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/00faithlss/Icaro-launcher/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/00faithlss/Icaro-launcher/tree/v0.1.0
