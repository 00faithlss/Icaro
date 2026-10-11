# Roadmap del System Design

Actualizado el **11-10-2026**. Referencia: `icaro-design-system.rar`, sus componentes, definiciones y previews originales. Este archivo sigue la implementación del diseño; el [roadmap general](../README.md#roadmap) sigue las funciones del launcher.

Las siguientes entregas deben conectar y completar esos diseños, conservando sus composiciones, iconos, tipografías, colores y controles. Las diferencias conocidas se registran para evaluación antes de sustituir una vista.

## Cómo leer el estado

| Estado | Significado |
| --- | --- |
| Implementado | Está disponible en el alcance descrito. No significa que todos los usos futuros del componente estén conectados. |
| Parcial | Ya hay implementación, pero faltan partes del diseño o acciones reales. |
| Maqueta | La composición existe, pero usa ejemplos o acciones simuladas. Falta conectarla al núcleo. |
| Pendiente | Falta implementar o integrar el componente indicado. |
| A evaluar | Hay una diferencia con la referencia o una decisión de alcance que debe revisarse con el usuario. |

Una vista puede estar implementada visualmente y tener su función pendiente. Por ejemplo, el asistente instala Vanilla, pero todavía no instala loaders; el diálogo de exportación genera ZIP, pero no MRPACK.

## Avances recientes

- [x] Vanilla y Java oficiales conectados al asistente y a Jugar, con archivos guardados y verificados.
- [x] Estados de BotonJugar, cola de preparación, registros reales en Umbral/Consola e informe de fallo sin causas ficticias.
- [x] Asistente de cuatro pasos con selector de versiones, tarjetas de loader, radios de rombo y botones con sus flechas.
- [x] Filas de mods con casilla, interruptor, icono, creador, descripción, estado, versión y acciones.
- [x] Barra invertida de selección múltiple y papelera de mods con Deshacer durante diez segundos.
- [x] Tienda con filtros laterales, contexto de instancia y lista inferior de instalación con dependencias y tamaño.
- [x] Ficha lateral con miniaturas, versiones compatibles y dependencias obligatorias y opcionales.
- [x] Página completa con navegación fija, tabla de versiones y columna lateral de información.
- [x] Diálogo de exportación con casillas, tamaños, destino y acciones del pie; ZIP conectado a los archivos reales.
- [x] Archivos con barra de acciones, árbol y tamaños, vista previa de solo lectura y renombrado dentro de la fila.
- [x] Galería del catálogo conectada al visor existente, usando originales de mayor resolución.
- [x] Capturas aisladas de las vistas principales en Tinta y Piedra, a 1280×800.

## Base visual y componentes compartidos

| Referencia original | Estado | Lo que tenemos y lo que falta |
| --- | --- | --- |
| Tokens de color, tipografía, bordes y espaciado | Parcial | Tinta/Piedra, Archivo, Martian Mono y medidas compartidas. Falta completar metales de acento, escala y revisar todos los estados de foco. |
| Iconografía Pixelarticons | Implementado | Iconos originales de secciones y acciones; los botones restaurados usan el mismo sistema. |
| `Boton`, `Campo`, `Selector`, `Interruptores` | Implementado | Controles compartidos, variantes, casillas cuadradas, interruptores y rayado de deshabilitados. |
| `Chips`, `Insignia`, `Pestanas`, `Contador`, `Senal` | Implementado | Filtros, estados acompañados de texto, pestañas y contadores usados en las vistas. |
| `Progreso`, `BloqueCodigo` | Parcial | Progreso real en autenticación, mods, Vanilla y Java. Falta la pantalla completa de integridad y otras tareas. |
| `Tabla` | Parcial | Tabla de versiones en la página del proyecto. Falta completar su uso y navegación en las demás tablas del diseño. |
| `Tarjeta`, `Avatar` | Parcial | Tarjetas y representaciones de respaldo disponibles; iconos de proyectos y creadores desde Modrinth. Falta la textura real de la cuenta y los datos de servidores. |
| `DeslizadorMemoria` | Parcial | Configuración por instancia guardada; memoria mínima/máxima y Metaspace aplicados al proceso; límites según RAM real en Windows/Linux. Java compatible detectado o descargado. Falta completar recomendaciones y advertencias de reserva para sistema/otras instancias. |
| `Dialogo`, `MenuContextual`, `Toast` | Implementado | Diálogos con velo, menús y avisos; acciones de instancia y confirmación de actualizaciones. Aviso de Deshacer de mods de diez segundos. |
| `Tooltip` | Parcial | Ayudas disponibles en algunos controles; falta cubrir todos los botones de solo icono y el modo angosto. |
| `Banda`, `Detalle`, `Lamina`, `Cover` | Implementado | Grabados, recortes, portadas de respaldo y composición de títulos sobre bloques propios. Las capturas reales como portada siguen pendientes. |
| `Lupa` | Parcial | Utilidades existentes y zoom del visor. La lupa sobre las láminas del launcher todavía no está aplicada. |
| `ListaVirtual` | Pendiente | Los catálogos tienen paginación; falta una lista que dibuje solo las filas visibles para versiones, archivos y registros extensos. |
| `ListaReordenable` | Parcial | Hay interacción de orden en la maqueta de servidores. Falta aplicarla a recursos reales y persistir sus prioridades en el juego. |
| `SelectorColor` | Pendiente | Falta conectar el selector definido a iconos/colores de instancia y revisar su composición original. |
| `ZonaSoltar` | Parcial | Hay estados de arrastre en las maquetas. Falta conectar la importación real de packs, recursos y skins con la zona original. |

## Ventana, bienvenida y sesión

| Referencia original | Estado | Lo que tenemos y lo que falta |
| --- | --- | --- |
| `AppShell` | Parcial | Barra de título, siete secciones, cuenta y controles de ventana; Ctrl+1 a Ctrl+7 y Ctrl+K. Jugar se oculta sin sesión/selección; muestra proceso y preparación reales de Vanilla. Falta completar teclado, bandeja y estados de los demás instaladores. |
| `DisenoAngosto` | Pendiente | El cambio de tamaño de ventana está implementado. Falta el conjunto original de quiebres: barra colapsada, filtros en popover e índice de Ajustes como selector. |
| `PrimeraEjecucion` | Parcial | Bienvenida sin cuenta activa, acceso a Microsoft y entrada como invitado. Java y preparación inicial aún no ejecutan tareas reales. |
| `SelectorCuenta`, `EstadosSesion` | Parcial | Menú con cuenta real, invitado, iniciar/cerrar sesión, recuperación y errores. Falta conectar todos los estados del diseño y varias cuentas persistentes. |
| `CodigoDispositivo` | A evaluar | El modo real conserva Hermes y usa autorización por navegador con retorno a Ícaro y progreso. Código/QR quedan solo en la maqueta; se retiraron del flujo real por el problema observado y el recorrido por navegador fue aprobado y probado por el usuario. |
| `EstadosPantalla`, `EstadoVacio`, `EnConstruccion` | Parcial | Componentes y estados vacíos disponibles; catálogos con carga, error y cancelación, e instancias locales recuperadas del disco. Falta aplicar el conjunto completo de carga, vacío, error y sin conexión a cada servicio. |
| `MenuBandeja` | Pendiente | Falta integración nativa, estado del juego, instancias recientes y comportamiento al cerrar la ventana. |

## Instancias

| Referencia original | Estado | Lo que tenemos y lo que falta |
| --- | --- | --- |
| `TarjetaInstancia`, `PanelLateral` | Parcial | Lista, selección, grupos y metadatos guardados. Vanilla pasa de instalación a lista y ejecución con datos reales. Los loaders quedan **Por instalar**. Falta tiempo jugado acumulado y lanzamiento de loaders. |
| `Asistente`, `SelectorVersion`, `SelectorLoader` | Parcial | Composición original de cuatro pasos, iconos y radios de rombo; versiones Mojang con caché, nombre, portada y carpeta propia. Crear Vanilla instala Minecraft y Java. Falta consultar e instalar versiones concretas de loaders. |
| `EditorInstancia` | Parcial | Encabezado y pestañas existentes; nombre, grupo y Java guardados; Vanilla omite Mods. Java, memoria y argumentos aplicados al juego. General requiere icono/color, notas y guardado completo de resolución/servidor. |
| `AccionesInstancia` | Parcial | Abrir carpeta, eliminar a papelera y Deshacer conectados. Exportación ZIP real con diálogo original. Duplicar, importar y otras acciones aún necesitan núcleo. |
| `CambiarVersion` | Maqueta | Diálogo de riesgo y alternativa de copia disponibles. Faltan copia real, compatibilidad e instalación de la versión o loader nuevo. |
| `ImportarClonar` | Maqueta | Selector de orígenes y composición disponibles. Falta leer packs, detectar otros launchers, mostrar datos reales e importar/clonar archivos. |
| `ExportarModpack` | Pendiente | Falta el flujo completo MRPACK: nombre, versión, autor, referencias de mods y archivos incluidos. El selector MRPACK del diálogo de instancia permanece deshabilitado. |
| `ArchivosInstancia`, `Explorador` | Parcial | Barra con iconos, árbol y tamaños, abrir/copiar ruta/renombrar, vista previa de texto y papelera. Falta Deshacer/eliminación definitiva, navegación completa por flechas y previews de imágenes, JAR y mundos. |
| `RecursosInstancia` | Parcial | Componentes de recursos disponibles; la integración actual debe recuperar la separación original entre recursos y shaders. Faltan inventario real, prioridad en `options.txt`, activación, compatibilidad e instalación. |
| `MundosInstancia` | Maqueta | Lista y panel de copias disponibles. Falta leer mundos, gestionar copias/restauración y conectar sus acciones al juego y disco. |
| `Capturas` | Parcial | Galería original conectada a las PNG que Minecraft guarda por instancia, filtros por identidad, refresco en segundo plano y visor con el original. Copiar imagen/archivos y abrir carpeta funcionan en Windows. Falta eliminación reversible y copia de imágenes en Linux. |
| `BotonJugar`, `Umbral` | Parcial | Estados originales conectados a instalación, preparación y ejecución Vanilla; Jugar sin MB, Preparando al verificar archivos guardados e Instalando solo al descargar archivos. En Windows el Umbral espera la ventana del proceso Minecraft y retira la capa completa con fundido, sin cortina ni remontar la vista, según la corrección solicitada por el usuario. Falta detección equivalente en Linux y cerrar launcher conservando juego mediante bandeja. |
| `Picar` | Parcial | Animación y eliminación de instancia con papelera conectadas. Faltan los recorridos de eliminación de mundos y archivos previstos en el diseño. |

## Mods y modpacks

| Referencia original | Estado | Lo que tenemos y lo que falta |
| --- | --- | --- |
| `FilaMod` | Implementado | Casilla, interruptor, icono, nombre/creador, descripción, estados, versión y acciones; desactivados atenuados y conflictos identificados. |
| `ModsInstancia` | Implementado | Inventario real, filtros, selección múltiple, activar/desactivar, actualizaciones con confirmación, quitar a papelera y Deshacer. |
| `TiendaMods` | Parcial | Composición original, filtros laterales, contexto de Minecraft/loader, búsqueda, filas y lista inferior de instalación. Modrinth conectado; CurseForge visible pero deshabilitado. |
| `FichaMod` | Implementado | Panel rápido con acciones, miniaturas, versiones compatibles, nombres/estado de dependencias y selección de opcionales. Ver página y Volver conservan la navegación anterior. |
| `PaginaMod` | Implementado | Página completa con cabecera/pestañas visibles, columna fija de información, descripción, galería, tabla de versiones, cambios y dependencias desde Modrinth. Instalar modpacks sigue pendiente. |
| `VisorCaptura` | Parcial | Compartido entre capturas locales reales y galería de proyectos; originales de mayor resolución, zoom, arrastre, flechas y Escape. Copiar y abrir carpeta conectados en Windows. Falta eliminación reversible de capturas locales. |
| `BibliotecaMods` | A evaluar | El original reúne un mod y todas las instancias que lo usan. La sección actual es **Modpacks**, según el alcance trabajado, y los mods se gestionan por instancia. Esa biblioteca global no está implementada; evaluar su ubicación antes de añadir una sección. |

## Servicios, ajustes y pantallas auxiliares

| Referencia original | Estado | Lo que tenemos y lo que falta |
| --- | --- | --- |
| `PantallaDescargas`, `ColaDescargas` | Parcial | Cola real de Vanilla/Java con pausa, reanudación, cancelación y reintento en las filas originales. Falta historial persistente y unificar mods y demás instaladores en la cola global. |
| `PantallaConsola`, `Consola` | Implementado | Barra interior del diseño con filtros independientes y conteos, búsqueda con resaltado, autoscroll con pausa al subir e Ir al final, copiar/limpiar e iconos. Render de filas visibles, selector por instancia, último registro desde latest.log, exportación TXT y vacío con San Jerónimo. Registros en vivo acotados y credenciales ocultas. |
| `PantallaServidores`, `TarjetaServidor` | Maqueta | Lista, tarjetas y reordenación disponibles. Falta añadir/guardar servidores reales, consultar su estado y jugar con la instancia elegida. |
| `RedPrivada`, `SincronizarMods` | Maqueta | Pantallas originales disponibles. Falta conectar red, comparación de manifiestos y sincronización real. |
| `PantallaCuenta` | A evaluar | La cuenta real usa una vista simplificada; la composición original con skin de frente/espalda, Apolo, capas e historial está en la maqueta. Evaluar su recuperación y conectar datos/acciones reales. |
| `EstudioSkin` | Maqueta | Composición y controles disponibles; faltan textura, modelo/capas reales, validación de PNG y aplicación a Microsoft/Minecraft. |
| `PantallaAjustes`, `FilaAjuste` | Parcial | Índice y filas existentes; los ajustes de Java por instancia se guardan. Falta persistir las preferencias generales y conectar los controles que todavía describen servicios no implementados. |
| `AjustesGenerales` | Parcial | Controles disponibles, abrir carpeta real y mantener/minimizar al iniciar Java. Faltan persistencia general, idioma completo, inicio con el sistema y bandeja para cerrar launcher conservando el juego. |
| `Apariencia` | Parcial | Tinta/Piedra y reducir movimiento disponibles. Falta completar seguir el tema del sistema, Mármol/Plata/Bronce y aplicar/validar escala de 80 a 150 %. |
| `AjustesJava` | Parcial | Configuración por instancia guardada y aplicada; selección automática según metadatos, detección de Java del sistema y descarga verificada del runtime oficial. Ajustes/editor muestran runtimes reales guardados; sliders limitados por la RAM real. Faltan agregar/probar rutas manuales desde Ajustes y completar las recomendaciones de memoria. |
| `AjustesAlmacenamiento`, `Almacenamiento` | Maqueta | Pantalla disponible con cifras de muestra. Faltan tamaños reales, barra por tramas, limpieza y traslado seguro de la carpeta de instancias. Los tamaños reales del diálogo ZIP no completan esta pantalla. |
| `ActualizacionLauncher` | Maqueta | Pantalla de versión y notas disponible. Falta buscar, descargar y aplicar actualizaciones del propio launcher. |
| `Integridad` | Maqueta | Cuadrícula y progreso de ejemplo disponibles. Falta revisar/reparar todos los archivos reales de una instancia; la verificación de mods descargados ya existe en otro flujo. |
| `ArchivosNoVerificados` | Maqueta | Composición disponible; se preservan los mods locales ajenos a Ícaro. Falta identificación por hash, lista real y acciones de buscar, confiar o quitar. |
| `PantallaError`, `InformeFallo` | Parcial | Fallos reales del juego con código de salida, causa desconocida cuando corresponde, registro y copia de informe en la composición original. Falta diagnóstico específico de conflictos y exportación del informe a archivo. |
| `Gabinete` | Parcial | Mosaico y visor integrados en Acerca de. Faltan sus usos originales como fondo de bienvenida en ventanas grandes y selector completo de portada, además del acercamiento y Lupa previstos. |
| `Movimiento`, `TransicionesPantalla` | A evaluar | Losa, avisos, Picar y Umbral implementados parcialmente. El launcher cambia contenido con fundido; Muro/Rastrillo/Prensa y Lupa no están aplicados como establece la referencia. Reducir movimiento existe; falta revisar el recorrido completo. |

## Orden para completar el diseño

1. **Instancia jugable:** validar Vanilla con una cuenta real y en Linux; extender el instalador a loaders manteniendo los componentes ya conectados. Completar historial de descargas, seguimiento de consola y Umbral.
2. **Editor y acciones:** completar General/Java, Archivos, Recursos, Mundos y Capturas; después cambiar versión, importar/clonar y exportar MRPACK con copias de seguridad.
3. **Cuenta:** evaluar la diferencia de PantallaCuenta y conectar su composición original junto con EstudioSkin y todos los estados de sesión.
4. **Servicios adicionales:** servidores, red/sincronización, integridad, archivos sin verificar, almacenamiento, bandeja y actualización del launcher.
5. **Revisión transversal:** ventanas angostas, escala, metales, foco y teclado, tooltips, virtualización y movimientos pendientes; comprobar ambas paletas.

## Diferencias para evaluar

- **Cuenta:** recuperar la composición original en lugar de mantener la pantalla real simplificada.
- **BibliotecaMods:** decidir dónde encaja la biblioteca entre instancias, conservando la sección Modpacks actual.
- **Movimiento:** revisar el fundido actual frente a las transiciones y efectos originales antes de cambiarlo.
- **Inicio de sesión:** conservar el recorrido por navegador ya aprobado; revisar código/QR únicamente si se retoma ese modo y se resuelve su redirección.

Los ajustes de adaptación ya conocidos —tercer filtro de versiones antiguas en el asistente y pestaña Mods oculta en Vanilla— se conservan. La [comparación con el System Design](revision-system-design.md) describe las vistas recuperadas y sus límites.

## Dónde continuar

| Área | Implementación de referencia |
| --- | --- |
| Asistente, exportación, ajustes y pantallas auxiliares | [pantallas.rs](../crates/ui/examples/launcher_aux/pantallas.rs) |
| Filas, selección, tienda y ficha rápida | [diseno_mods.rs](../crates/ui/examples/launcher_aux/diseno_mods.rs) |
| Página completa, versiones, galería y cambios | [ficha.rs](../crates/ui/examples/launcher_aux/ficha.rs) |
| Servicios y navegación | [servicios.rs](../crates/ui/examples/launcher_aux/servicios.rs), [mods_reales.rs](../crates/ui/examples/launcher_aux/mods_reales.rs) |
| Instancias, editor y archivos | [instancias_reales.rs](../crates/ui/examples/launcher_aux/instancias_reales.rs), [editor.rs](../crates/ui/src/editor.rs), [archivos.rs](../crates/ui/examples/launcher_aux/archivos.rs) |
| Componentes y tokens | [componentes.rs](../crates/ui/src/componentes.rs), [tema.rs](../crates/ui/src/tema.rs), [iconos.rs](../crates/ui/src/iconos.rs) |
| Cuenta y skin de referencia | [cuenta.rs](../crates/ui/src/cuenta.rs), [sesion.rs](../crates/ui/examples/launcher_aux/sesion.rs) |
| Núcleo conectado | [auth.rs](../crates/core/src/auth.rs), [instancias.rs](../crates/core/src/instancias.rs), [mods.rs](../crates/core/src/mods.rs), [exportar.rs](../crates/core/src/exportar.rs) |
| Vanilla, Java y ejecución | [minecraft.rs](../crates/core/src/minecraft.rs), [java.rs](../crates/core/src/java.rs), [lanzamiento.rs](../crates/core/src/lanzamiento.rs), [juego_real.rs](../crates/ui/examples/launcher_aux/juego_real.rs) |
| Revisión visual aislada | [pruebas_diseno.rs](../crates/ui/examples/launcher_aux/pruebas_diseno.rs), [guía de capturas](desarrollo.md#capturas-aisladas-del-diseño) |
