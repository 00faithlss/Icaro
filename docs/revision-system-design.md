# Comparación con el System Design

Referencia: `icaro-design-system.rar`, entregado el 11 de octubre de 2026. Las definiciones y previews originales son la referencia para la composición, los controles, iconos Pixelarticons, tipografías, colores y medidas. Se revisan las pantallas con capturas de los widgets reales en Tinta y Piedra; los datos de las capturas son fixtures, separados de las cuentas e instancias del usuario.

El inventario completo de avances y pendientes por componente está en el [Roadmap del System Design](roadmap-system-design.md). Las próximas entregas funcionales se siguen en el [roadmap general](../README.md#roadmap).

## Vistas recuperadas

| Componente original | Implementación |
| --- | --- |
| `Asistente` | Diálogo de 880 px, cuatro pasos, selector de versiones a la izquierda, tarjetas de loader con radios de rombo a la derecha y pie con Cancelar, Atrás y Siguiente con sus flechas. Se conservan las versiones antiguas en un tercer filtro compacto. Las versiones de Minecraft se obtienen de Mojang; no se muestran versiones recomendadas de loader ficticias en el modo real. |
| `FilaMod` y `ModsInstancia` | Filas con separadores, casilla de selección, interruptor, icono, nombre, creador, descripción, estado, versión y acciones. La selección muestra la barra invertida para activar, desactivar, actualizar y quitar. Quitar conserva los archivos en la papelera y ofrece Deshacer durante diez segundos. |
| `TiendaMods` | Filtros laterales, encabezado con instancia, Minecraft y loader, búsqueda a la derecha, filas compactas con categoría, creador y descargas. Agregar prepara una lista; la barra inferior invertida enumera los mods, sus dependencias y el tamaño antes de Instalar. Volver a mis mods recupera la lista instalada. |
| `FichaMod` | Panel lateral de 640 px con acciones, descripción, tres miniaturas, versiones compatibles, nombres de dependencias y casillas para las opcionales. Las imágenes abren el visor existente. Ver página abre la ficha completa; Volver recupera el panel anterior. |
| `PaginaMod` | Encabezado y pestañas visibles durante el desplazamiento, tabla de versiones y columna lateral fija de 300 px con acción principal, compatibilidad, estadísticas, creador y enlaces. Se mantienen las pestañas Descripción, Galería, Versiones, Cambios y Dependencias. |
| `AccionesInstancia`: exportar | Diálogo de 560 px con formatos ZIP/MRPACK, casillas de contenido, tamaños, destino con icono de carpeta y selector nativo, estimación y acciones del pie. ZIP exporta los archivos reales seleccionados y los metadatos de la instancia. |
| `ArchivosInstancia` | Barra de ruta y acciones con sus iconos; árbol con tamaños a la izquierda y vista previa de solo lectura a la derecha, dentro del mismo marco. Renombrar edita el nombre dentro del árbol; Enter confirma y Escape cancela. |

La confirmación de actualización conserva `Dialogo`, el velo con losa y la caída de Faetón. Vanilla omite la pestaña y el contador de mods, manteniendo el resto del editor.

## Diferencias y funciones pendientes

- **Cuenta (`PantallaCuenta`)** conserva una vista real simplificada. El diseño original tiene skin de frente y espalda, Apolo, capas, historial y estudio; falta conectar esa composición y sus acciones a la cuenta real. No se sustituye aquí sin evaluación del usuario.
- **Inicio de sesión (`CodigoDispositivo`)** usa el flujo por navegador y retorno directo a Ícaro solicitado y probado por el usuario. No muestra código ni QR del flujo que fallaba; conserva la lámina de Hermes y el progreso real.
- **CurseForge** aparece como fuente deshabilitada, en el lugar definido por el diseño. No hay una conexión simulada en el modo real.
- **MRPACK** conserva su selector, pero no permite exportar hasta disponer de instalación real de Minecraft y de la versión concreta del loader. No genera un JSON aislado presentado como modpack.
- Crear una instancia guarda su configuración y carpeta; **instalar Minecraft, instalar el loader y lanzar el juego siguen pendientes**. Importar, duplicar e instalar modpacks también siguen pendientes.
- El explorador conserva su papelera interna; la recuperación y eliminación definitiva desde esa pestaña todavía requieren conexión al flujo completo definido en el diseño.

## Implementación

- Asistente y exportación: [pantallas.rs](../crates/ui/examples/launcher_aux/pantallas.rs).
- Tienda, filas, selección, lista de instalación y ficha rápida: [diseno_mods.rs](../crates/ui/examples/launcher_aux/diseno_mods.rs).
- Página completa: [ficha.rs](../crates/ui/examples/launcher_aux/ficha.rs).
- Navegación y servicios: [servicios.rs](../crates/ui/examples/launcher_aux/servicios.rs), [mods_reales.rs](../crates/ui/examples/launcher_aux/mods_reales.rs).
- Archivos: [editor.rs](../crates/ui/src/editor.rs), [archivos.rs](../crates/ui/examples/launcher_aux/archivos.rs).
- Exportación ZIP y papelera de mods: [exportar.rs](../crates/core/src/exportar.rs), [mods.rs](../crates/core/src/mods.rs).
- Capturas aisladas para revisión: [pruebas_diseno.rs](../crates/ui/examples/launcher_aux/pruebas_diseno.rs).
