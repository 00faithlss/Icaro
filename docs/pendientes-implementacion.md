# Implementación pendiente

Actualizado el **12-10-2026**. Este documento describe el trabajo que falta; no presenta estas funciones como disponibles. El [roadmap del System Design](roadmap-system-design.md) detalla los componentes y sus diferencias visuales.

## 1. Instalación y ejecución de loaders oficiales

**Estado actual:** el asistente y Cambiar versión consultan los catálogos oficiales de Fabric, Quilt, Forge y NeoForge y guardan la versión elegida. La instalación y el lanzamiento completos funcionan para Vanilla. Elegir un loader todavía no instala ni ejecuta ese loader.

- [ ] Conectar la instalación de cada loader usando sus perfiles, manifiestos, bibliotecas y/o instaladores oficiales, según corresponda. No sustituir el resultado oficial por versiones o perfiles inventados.
- [ ] Respetar la versión concreta guardada en la instancia y su compatibilidad con Minecraft; resolver Java y dependencias del perfil resultante.
- [ ] Preparar la instalación fuera de la carpeta activa, verificar las descargas y publicar el resultado solo al terminar. Conservar la instalación anterior ante error o cancelación.
- [ ] Integrar el avance con la cola, pausa/cancelación, consola y estados estables del botón; comprobar archivos existentes sin mostrar una instalación ficticia.
- [ ] Lanzar el perfil instalado con sus argumentos, bibliotecas y nativos reales y con los mods de esa instancia.
- [ ] Conectar el mismo recorrido a Crear instancia, Instalar/Jugar y Cambiar versión. Probar cada loader con una instancia nueva, una instalación reutilizada y una migración.

**Criterio de finalización:** instalar y abrir una instancia de cada loader con un mod compatible, reutilizar sus archivos al reiniciar y recuperarse de una descarga fallida sin perder mundos ni mods.

## 2. Importación e instalación de modpacks

**Estado actual:** la tienda descubre modpacks y existe la composición ImportarClonar. Falta convertir el pack elegido o el archivo importado en una instancia jugable. Exportar una lista de mods o un ZIP de instancia no implementa este recorrido.

- [ ] Leer MRPACK: metadatos, Minecraft, loader, referencias de archivos, hashes y archivos incluidos; presentar una revisión antes de importar.
- [ ] Descargar y verificar los archivos referenciados, aplicar los archivos incluidos y preparar el loader correspondiente mediante el instalador oficial.
- [ ] Importar los ZIP de Ícaro y los formatos compatibles de Prism/MultiMC; detectar el formato y explicar los formatos no admitidos.
- [ ] Conectar la selección de archivo y ZonaSoltar a la importación real.
- [ ] Validar las rutas del archivo comprimido y los límites de extracción; evitar que archivos o enlaces escriban fuera de la nueva instancia.
- [ ] Integrar ArchivosNoVerificados para los archivos sin origen comprobado; conservar las decisiones de confianza asociadas al contenido exacto.
- [ ] Completar el descubrimiento/importación desde otros launchers y la duplicación de una instancia sin alterar el original.
- [ ] Probar la importación manual de los ZIP exportados en Prism/MultiMC; actualmente se verifica su estructura con pruebas.
- [ ] Implementar la exportación MRPACK con versiones concretas y referencias verificadas, separada de la exportación ZIP y de la lista de mods.

**Criterio de finalización:** importar un pack en una instancia nueva, instalarlo y abrirlo; un fallo debe dejar el origen intacto y permitir cancelar o reintentar.

## 3. Editor, descargas y recuperación

- [ ] Completar las opciones pendientes de General y Java y su persistencia.
- [ ] Gestionar los mundos con acciones reales y conectar sus respaldos y recuperación a los componentes del diseño.
- [ ] Ordenar la prioridad de resourcepacks por arrastre y ampliar el soporte de configuración de shaders a otros motores.
- [ ] Completar la recuperación de archivos y capturas desde sus papeleras y la revisión de integridad.
- [ ] Guardar el historial de descargas entre sesiones y unificar los instaladores en la cola global.
- [ ] Acumular y guardar el tiempo jugado por instancia.

## 4. Cuenta, servidores y publicación

- [ ] Completar PantallaCuenta y EstudioSkin con la composición original y acciones reales.
- [ ] Implementar servidores y sincronización de mods. Por ahora Servidores muestra EnConstruccion en modo real.
- [ ] Resolver la integración opcional de CurseForge cuando exista acceso válido a su API y condiciones de distribución.
- [ ] Completar almacenamiento y bandeja, y revisar los comportamientos de cierre.
- [ ] Publicar el ejecutable y los metadatos de actualización y probar una actualización completa. La comprobación y descarga del actualizador ya están conectadas; ver el [contrato de publicación](actualizaciones-launcher.md).

## 5. Validación y diseño

- [ ] Repetir en Arch Linux el recorrido de Vanilla confirmado en Windows y validar los nuevos loaders en ambos sistemas.
- [ ] Revisar foco, teclado, ventanas pequeñas, escala, ambos temas, metales, rendimiento y virtualización.
- [ ] Completar las transiciones y efectos pendientes del System Design, respetando Reducir movimiento.
- [ ] Validar el empaquetado y los recorridos completos con archivos reales antes de la primera publicación.

## Correcciones recientes

La creación/preparación de instancias mantiene estable el estado del botón frente a eventos intercalados de descarga y verificación. La trama de Preparando es estática y no cambia cada fotograma. Ya se incorporaron la exportación de instancia, los seis formatos de lista de mods, el editor de iconos con vista previa inmediata, los avisos, apariencia, verificación de archivos y la confirmación de eliminación con bloques y grietas en toda la tarjeta; sus límites siguen registrados en el roadmap del diseño.
