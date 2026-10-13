# Icaro Launcher

El nombre visible del producto es **Icaro Launcher** y el repositorio es `00faithlss/Icaro-launcher`. Las carpetas de datos `Icaro`, el protocolo `icaro://` y los identificadores internos conservan sus nombres actuales para mantener la compatibilidad.

Launcher de Minecraft escrito en Rust, con Windows y Arch Linux como plataformas objetivo.

[![Licencia: GPL-3.0-only](https://img.shields.io/badge/License-GPL--3.0--only-red.svg)](LICENSE)

Su nombre y su imagen se inspiran en la mitología. Ícaro, que quiso volar, da nombre al proyecto. Hefesto, el herrero, está en los ajustes, Atlas en el mundo y Faetón en los errores. Hay más figuras repartidas por el launcher, y descubrirlas es parte de usarlo.

## Tabla de contenidos

- [Estado actual](#estado-actual)
- [Compilar y probar](#compilar-y-probar)
- [Roadmap](#roadmap)
- [Componentes de terceros, licencias y fuentes](#componentes-de-terceros-licencias-y-fuentes)
- [Documentación](#documentación)
- [Licencia](#licencia)

## Estado actual

Icaro Launcher está en desarrollo. La interfaz ya permite recorrer instancias, servidores, descargas, consola, modpacks, capturas, ajustes y cuenta, con temas claro y oscuro.

> [!NOTE]
> El ejemplo conecta Microsoft y Modrinth, guarda las instancias, instala Minecraft Vanilla con Java automático, ejecuta el juego con una cuenta activa, instala y actualiza mods compatibles y exporta instancias en ZIP. Sin cuenta activa muestra la bienvenida; las secciones sin datos aparecen vacías. `ICARO_DEMO=1` permite revisar los datos de muestra. La instalación de loaders y el lanzamiento de instancias con mods siguen pendientes.

El núcleo descarga los archivos de Vanilla desde los manifiestos oficiales, comprueba tamaño y SHA-1 y reutiliza la instalación guardada. La versión de Java se obtiene de esos metadatos; si no hay una compatible, descarga el runtime oficial de Mojang. Consulta [Vanilla y Java](docs/vanilla-y-java.md) para probar el recorrido.

Los mods, resourcepacks y shaders se gestionan dentro de cada instancia con filas y controles compartidos. Los resourcepacks se activan en `options.txt`; los shaders usan la configuración existente de Iris. La sección **Modpacks** está destinada a descubrir packs; instalarlos como nuevas instancias sigue pendiente.

Las instancias usan carpetas legibles por nombre, mantienen su identidad interna y guardan `instance.cfg`. Las rutas anteriores se migran al recuperar las instancias, conservando sus mundos. El editor permite cambiar el icono desde su imagen o el menú de acciones, recortarlo y volver al icono generado. Consulta [instancias locales](docs/instancias-locales.md).

Apariencia incluye el tema del sistema, Mármol/Plata/Bronce y escala de 80 a 150 %. Los avisos se apilan con acciones, y la eliminación utiliza Picar con revisión de respaldos. La exportación de instancia incorpora el árbol de archivos y formatos Ícaro y Prism/MultiMC. La lista de mods se exporta por separado como HTML, Markdown, texto, JSON, CSV o una plantilla personalizada.

## Compilar y probar

Instala Rust estable con [rustup](https://rustup.rs/) y las herramientas de compilación correspondientes a tu sistema. Para descargar el repositorio y abrir la interfaz:

```sh
git clone https://github.com/00faithlss/Icaro-launcher.git
cd Icaro-launcher
git switch develop
cargo run -p icaro-ui --example launcher
```

La rama `develop` contiene el trabajo en curso; `main` recibe los cambios integrados. El ejemplo abre la interfaz con autenticación y catálogo conectados. `ICARO_DEMO=1` conserva las cuentas y tiendas simuladas para revisar el diseño sin consultas de red. Consulta [Inicio de sesión y catálogo](docs/login-y-catalogo.md) para probar los servicios.

> [!TIP]
> Para revisar los componentes del sistema de diseño, ejecuta `cargo run -p icaro-ui --example galeria`.

Para probar el núcleo y ejecutar las pruebas:

```sh
cargo run -p icaro-cli
cargo test --workspace
cargo test -p icaro-ui --example launcher
```

La herramienta de línea de comandos consulta el manifiesto de Mojang y verifica el JSON de Minecraft 1.21.1. Consulta [Desarrollo](docs/desarrollo.md) para los demás ejemplos y las opciones de la maqueta.

## Roadmap

Actualizado el **12-10-2026**. Las casillas marcadas corresponden a funciones implementadas en el alcance indicado; una pantalla de demostración no cuenta como integración real. El orden de los pendientes representa las prioridades actuales.

El avance de las pantallas y componentes originales se sigue por separado en el [Roadmap del System Design](docs/roadmap-system-design.md). Las diferencias que requieren evaluación están en la [comparación con el diseño](docs/revision-system-design.md).

### Base e interfaz

- [x] Estructura en Rust: núcleo, herramienta de línea de comandos e interfaz con iced.
- [x] Consulta del manifiesto de Mojang y verificación del JSON de una versión.
- [x] Componentes compartidos, tipografías, iconos Pixelarticons, grabados y temas Tinta y Piedra.
- [x] Navegación por las siete secciones, Ctrl+1 a Ctrl+7 y búsqueda global con Ctrl+K.
- [x] Ventana con controles propios y cambio de tamaño mediante los bordes en Windows.
- [x] Caché de imágenes y visor con originales de mayor resolución, zoom, arrastre y navegación por teclado.
- [x] Estados vacíos sin datos ficticios en el modo real; datos de muestra separados mediante `ICARO_DEMO=1`.
- [x] Avisos temporales, diálogos compartidos y opción de reducir movimiento.
- [x] Recuperación de las composiciones originales del asistente, filas de mods, tienda, fichas, tabla de versiones, exportación y Archivos.
- [x] Capturas aisladas para revisar estas vistas en Tinta y Piedra sin modificar cuentas ni instancias del usuario.

### Cuenta y sesión

- [x] Autorización de Microsoft con PKCE y comprobación de `state`; intercambio con Xbox y validación de acceso y perfil de Minecraft Java.
- [x] Retorno local y retorno directo a Ícaro en Windows mediante `icaro://oauth/microsoft`; probados con una cuenta real.
- [x] Almacenamiento protegido de una cuenta mediante el almacén del sistema; recuperación al reabrir confirmada en Windows.
- [x] Renovación de sesión y eliminación de la credencial al cerrar sesión implementadas; validación manual de ambos recorridos todavía pendiente.
- [x] Bienvenida sin cuenta activa, acceso como invitado y progreso de las etapas del inicio de sesión.
- [x] API de código de dispositivo con vencimiento y cancelación; emisión real probada. El código y QR no están disponibles en la interfaz real por la redirección fallida observada.
- [ ] Validar manualmente renovación, cierre de sesión y funcionamiento del almacén de credenciales en Linux.
- [ ] Conectar la pantalla original de Cuenta y Estudio de skin: textura real, capas, historial y aplicación de skins.
- [ ] Gestión persistente de varias cuentas.

### Instancias y archivos

- [x] Consulta de versiones de Mojang con caché de 24 horas y uso de la copia válida si falla la actualización.
- [x] Asistente de cuatro pasos que guarda versión, familia de loader, nombre y portada; cada instancia tiene identificador estable y carpeta propia.
- [x] Selectores de versión y loader separados, con versiones oficiales seleccionables y elección persistida.
- [x] Cambio de Minecraft y elección de loader con el nuevo `CambioVersionInstancia`: compatibilidad por lotes de todos los mods instalados, actualizaciones opcionales, respaldo obligatorio al bajar y copia independiente. Preparación aislada y recuperación de archivos/metadatos si falla; instalar el loader sigue pendiente.
- [x] Recuperación de instancias, grupos y ajustes de Java desde disco; renombrado sin mover la carpeta de juego.
- [x] Eliminación de instancias a la papelera local y recuperación mediante Deshacer.
- [x] Vanilla omite la pestaña y el contador de mods; la barra inferior de Jugar se oculta sin sesión o sin instancia seleccionada.
- [x] Explorador conectado a la carpeta real: árbol, tamaños, vista previa de texto, abrir, copiar ruta, renombrar y mover archivos a su papelera.
- [x] Exportación ZIP con metadatos, selección de contenido, tamaños reales y selector nativo de destino; sin sobrescribir exportaciones anteriores.
- [x] Instalar Minecraft Vanilla al crear la instancia: cliente, bibliotecas, nativos, assets y configuración de registros verificados y reutilizables.
- [x] Detectar Java compatible o instalar el runtime oficial requerido por los metadatos de Minecraft; reutilizarlo y permitir selección por instancia.
- [x] Ejecutar Vanilla con la cuenta activa, memoria, Metaspace y argumentos de la instancia; capturar registros y código de salida, y detener el proceso con confirmación.
- [x] Conectar preparación, pausa, cancelación y reintento a la cola; estados reales de Jugar, Umbral, consola e informe de fallo con las composiciones existentes.
- [x] Lanzamiento real de Vanilla 26.3 en Windows confirmado por el usuario y correcciones del Umbral probadas durante el uso; retirada con fundido al detectar la ventana de Minecraft.
- [ ] Validar el recorrido completo en Arch Linux y completar la revisión de todas las opciones de cierre en Windows.
- [ ] Instalar loaders compatibles y lanzar instancias con sus mods.
- [ ] Importar y duplicar instancias desde sus acciones generales; cambiar la familia de loader.
- [ ] Exportar modpacks MRPACK con la versión concreta de Minecraft y loader.
- [x] Instalar resourcepacks ZIP, leer su inventario y activar en `options.txt`; instalación fuera de versión con confirmación.
- [x] Resourcepacks y shaders con las mismas filas, búsqueda, filtros y selección de mods; activar/desactivar, actualizar y quitar archivos reales.
- [x] Pestaña Shaders e instalación ZIP; activación mediante configuración existente de Iris.
- [ ] Ordenar resourcepacks por prioridad y admitir otros motores de shaders.
- [ ] Leer mundos reales, crear y restaurar copias de seguridad y permitir jugar directamente a un mundo.
- [x] Leer capturas F2 reales por instancia; visor con controles de ventana, copiar imagen y abrir carpeta.
- [ ] Completar eliminación reversible de capturas locales.
- [ ] Completar las vistas previas y la recuperación de archivos eliminados en el explorador.

### Catálogo y gestión de mods

- [x] Catálogo público de Modrinth con iconos, búsqueda, filtros de Minecraft y loader, orden y paginación.
- [x] Descripciones y changelogs formateados, galería con originales, creadores, licencias, fechas y estadísticas.
- [x] Ficha rápida, página completa, versiones compatibles y dependencias con sus nombres.
- [x] `PaginaMod` e `InfoProyecto` del ZIP nuevo: galería, tabla con filtros, columna de 320 px, iconos originales, avatares reales y más proyectos del creador. Los datos que la API no entrega se omiten.
- [x] Volver conserva el catálogo, búsqueda, filtros, página y ficha anterior; la tienda permite regresar a los mods instalados.
- [x] Tienda de mods por instancia con categorías, filtro de lado y compatibilidad fijada a Minecraft y loader.
- [x] Lista de instalación con mods, dependencias y tamaño antes de descargar; selección de dependencias opcionales.
- [x] Instalación real de mods y dependencias obligatorias transitivas, con comprobación de compatibilidad, conflictos, tamaño y SHA-512.
- [x] Publicación conjunta de archivos preparados; una descarga fallida conserva los mods anteriores y los archivos locales ajenos a Ícaro.
- [x] Inventario persistente de mods, activación y desactivación, detección de actualizaciones compatibles y confirmación antes de actualizar.
- [x] Selección múltiple para activar, desactivar, actualizar y quitar; papelera de mods con Deshacer durante diez segundos.
- [x] Seleccionar todos los mods y panel de ficha entrando desde el borde derecho.
- [x] Clic fuera cierra el panel; volver desde la página completa restaura la instancia o el catálogo de origen.
- [ ] Perfil del creador dentro de la tienda: falta el componente de diseño; por ahora se abre su perfil público en Modrinth.
- [ ] Instalar modpacks como nuevas instancias, incluidos sus archivos de configuración.

### Actualizaciones del launcher

- [x] Pantalla `ActualizacionLauncher`, comprobación de releases y descarga portable verificada con SHA-256; aplicar ahora o al cerrar en Windows x86-64.
- [ ] Publicar releases con ejecutable y `latest.json`, y probar una actualización completa. [Contrato de publicación](docs/actualizaciones-launcher.md).

### Próximas entregas, en orden

Consulta el [detalle de implementación pendiente](docs/pendientes-implementacion.md) para el alcance y las comprobaciones de loaders oficiales, importación de packs y funciones restantes.

1. **Instancias con loader:** instalar Fabric, NeoForge, Forge y Quilt desde sus fuentes oficiales, respetar la versión elegida y comprobar el lanzamiento con los mods ya gestionados. Los catálogos y la elección están implementados; la instalación todavía está pendiente.
2. **Validación multiplataforma:** repetir el recorrido de Vanilla en Arch Linux y completar las opciones de cierre en Windows; el lanzamiento real en Windows ya fue confirmado.
3. **Importación y modpacks:** importar MRPACK/ZIP y otros launchers, duplicar instancias y exportar MRPACK. El cambio de versión ya permite respaldo, copia y actualización de mods; falta instalar los loaders.
4. **Editor completo:** conectar mundos, prioridad de resourcepacks, otros motores de shaders, integridad y eliminación/recuperación de archivos y capturas; completar el historial persistente y unificar los instaladores en la cola global. Recursos, shaders, capturas y consola ya tienen sus recorridos reales descritos arriba.
5. **Cuenta y servicios adicionales:** completar Cuenta y Estudio de skin, servidores y proveedores opcionales.
6. **Primera publicación:** validar Windows y Arch Linux, rendimiento, teclado, ventanas angostas y empaquetado.

### Integraciones y validaciones restantes

- [ ] Integración opcional de CurseForge, sujeta al acceso a su API y a las condiciones de distribución.
- [ ] Conexión a servidores y sincronización de mods con un servidor.
- [ ] Red privada y bandeja del sistema; publicar los artefactos y validar la actualización del propio launcher de extremo a extremo.
- [ ] Verificación y reparación de instancias e integración de la revisión de archivos de origen desconocido con los importes; la revisión manual de archivos ya está conectada.
- [ ] Completar adaptación a ventanas angostas, escala, foco y navegación por teclado según el diseño original.
- [ ] Persistir las preferencias generales y conectar sus efectos reales: idioma, inicio con el sistema y comportamiento al jugar/cerrar.
- [ ] Validación en Windows y Arch Linux, empaquetado y publicación de la primera versión funcional.

El [plan de integración](docs/integracion-real.md) detalla los servicios y sus condiciones de aceptación. El [Roadmap del System Design](docs/roadmap-system-design.md) indica qué vistas están conectadas, cuáles siguen como maqueta y qué falta completar de cada componente.

## Componentes de terceros, licencias y fuentes

Icaro Launcher utiliza los siguientes componentes directos. Las versiones indicadas corresponden al `Cargo.lock` actual; cada proyecto conserva su licencia y sus avisos de autoría.

| Componente | Versión | Uso | Licencia declarada |
| --- | --- | --- | --- |
| [iced](https://github.com/iced-rs/iced) | 0.13.1 | Interfaz de escritorio | [MIT][mit] |
| [winreg](https://github.com/gentoo90/winreg-rs) | 0.56.0 | Registro del enlace de retorno en Windows | [MIT][mit] |
| [image](https://github.com/image-rs/image) | 0.24.9 | Preparación y decodificación de imágenes | [MIT][mit] OR [Apache-2.0][apache] |
| [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark) | 0.11.3 | Lectura de descripciones Markdown del catálogo | [MIT][mit] |
| [Tokio](https://github.com/tokio-rs/tokio) | 1.53.2 | Ejecución asíncrona | [MIT][mit] |
| [reqwest](https://github.com/seanmonstar/reqwest) | 0.12.28 | Peticiones HTTP | [MIT][mit] OR [Apache-2.0][apache] |
| [Serde](https://github.com/serde-rs/serde) | 1.0.229 | Serialización de datos | [MIT][mit] OR [Apache-2.0][apache] |
| [serde_json](https://github.com/serde-rs/json) | 1.0.151 | Lectura y escritura de JSON | [MIT][mit] OR [Apache-2.0][apache] |
| [sha1 · RustCrypto](https://github.com/RustCrypto/hashes) | 0.10.7 | Verificación de los manifiestos de Mojang | [MIT][mit] OR [Apache-2.0][apache] |
| [thiserror](https://github.com/dtolnay/thiserror) | 2.0.21 | Errores del núcleo | [MIT][mit] OR [Apache-2.0][apache] |
| [anyhow](https://github.com/dtolnay/anyhow) | 1.0.104 | Errores de la herramienta de línea de comandos | [MIT][mit] OR [Apache-2.0][apache] |
| [embed-resource](https://github.com/nabijaczleweli/rust-embed-resource) | 3.0.12 | Recursos del ejecutable de Windows | [MIT][mit] |
| [base64](https://github.com/marshallpierce/rust-base64) | 0.22.1 | Codificación del desafío PKCE | [MIT][mit] OR [Apache-2.0][apache] |
| [getrandom](https://github.com/rust-random/getrandom) | 0.3.4 | Aleatoriedad del sistema para PKCE y estado de autorización | [MIT][mit] OR [Apache-2.0][apache] |
| [sha2 · RustCrypto](https://github.com/RustCrypto/hashes) | 0.10.9 | SHA-256 del desafío PKCE y SHA-512 de los mods descargados | [MIT][mit] OR [Apache-2.0][apache] |
| [zeroize](https://github.com/RustCrypto/utils) | 1.9.1 | Limpieza de buffers de credenciales | [Apache-2.0][apache] OR [MIT][mit] |
| [tokio-util](https://github.com/tokio-rs/tokio) | 0.7.20 | Cancelación de autenticación y consultas | [MIT][mit] |
| [webbrowser](https://github.com/amodm/webbrowser-rs) | 1.2.4 | Apertura del navegador del sistema | [MIT][mit] OR [Apache-2.0][apache] |
| [keyring](https://github.com/hwchen/keyring-rs) | 3.6.3 | Credential Manager en Windows y Secret Service en Linux | [MIT][mit] OR [Apache-2.0][apache] |
| [windows-sys](https://github.com/microsoft/windows-rs) | 0.61.2 | Cambio de tamaño de la ventana en Windows | [MIT][mit] OR [Apache-2.0][apache] |

`MIT OR Apache-2.0` indica una elección entre esas dos licencias. El árbol completo, incluidas las dependencias transitivas, está registrado en [Cargo.lock](Cargo.lock); los paquetes publicados incluyen sus propios textos de licencia.

### Tipografías, iconos e imágenes

| Recurso | Autor o proyecto | Licencia y fuente |
| --- | --- | --- |
| Archivo | [Omnibus-Type](https://github.com/Omnibus-Type/Archivo) | [SIL Open Font License 1.1](https://github.com/Omnibus-Type/Archivo/blob/master/OFL.txt) |
| Martian Mono | [Evil Martians](https://github.com/evilmartians/mono) | [SIL Open Font License 1.1](https://github.com/evilmartians/mono/blob/main/OFL.txt) |
| Pixelarticons | [Gerrit Halfmann](https://github.com/halfmage/pixelarticons) | [MIT](crates/ui/assets/iconos/LICENSE-pixelarticons.txt) |
| Fotografía de *La caída de Ícaro*, utilizada en el icono | Marie-Lan Nguyen / Wikimedia Commons; obra de Merry-Joseph Blondel | [CC BY 2.5](https://creativecommons.org/licenses/by/2.5/), con recorte y adaptación a grises. [Fuente y cambios](crates/ui/assets/icono/FUENTES-Y-LICENCIAS.md) |

Las obras y reproducciones utilizadas en los grabados conservan las condiciones de sus respectivas fuentes. Consulta los [créditos](docs/creditos.md) para la atribución del icono.

## Documentación

- [Índice de documentación](docs/README.md)
- [Acerca de Icaro Launcher](docs/acerca-de.md)
- [Desarrollo y estructura del proyecto](docs/desarrollo.md)
- [Roadmap del System Design](docs/roadmap-system-design.md)
- [Implementación pendiente: loaders, importación y siguientes entregas](docs/pendientes-implementacion.md)
- [Comparación con el System Design](docs/revision-system-design.md)
- [Cuenta y estudio de skin](docs/cuenta.md)
- [Integración real, autenticación y seguridad](docs/integracion-real.md)
- [Inicio de sesión y catálogo](docs/login-y-catalogo.md)
- [Créditos](docs/creditos.md)

## Licencia

[![Licencia: GPL-3.0-only](https://img.shields.io/badge/License-GPL--3.0--only-red.svg)](LICENSE)

El código de Icaro Launcher está disponible bajo **GNU General Public License v3.0 only**, identificada como `GPL-3.0-only`. El texto completo se encuentra en [LICENSE](LICENSE).

Las bibliotecas, tipografías, iconos e imágenes de terceros conservan sus licencias, detalladas en [Componentes de terceros](#componentes-de-terceros-licencias-y-fuentes). La fotografía utilizada para el icono de Ícaro se atribuye a **© Marie-Lan Nguyen / Wikimedia Commons** bajo [CC BY 2.5](https://creativecommons.org/licenses/by/2.5/).

[mit]: https://opensource.org/license/mit
[apache]: https://www.apache.org/licenses/LICENSE-2.0
