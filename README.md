# Ícaro

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

Ícaro está en desarrollo. La interfaz ya permite recorrer instancias, servidores, descargas, consola, modpacks, capturas, ajustes y cuenta, con temas claro y oscuro.

> [!NOTE]
> El ejemplo conecta Microsoft y Modrinth, guarda las instancias, instala Minecraft Vanilla con Java automático, ejecuta el juego con una cuenta activa, instala y actualiza mods compatibles y exporta instancias en ZIP. Sin cuenta activa muestra la bienvenida; las secciones sin datos aparecen vacías. `ICARO_DEMO=1` permite revisar los datos de muestra. La instalación de loaders y el lanzamiento de instancias con mods siguen pendientes.

El núcleo descarga los archivos de Vanilla desde los manifiestos oficiales, comprueba tamaño y SHA-1 y reutiliza la instalación guardada. La versión de Java se obtiene de esos metadatos; si no hay una compatible, descarga el runtime oficial de Mojang. Consulta [Vanilla y Java](docs/vanilla-y-java.md) para probar el recorrido.

Los mods se gestionan dentro de cada instancia. La sección **Modpacks** está destinada a descubrir e instalar packs como nuevas instancias.

## Compilar y probar

Instala Rust estable con [rustup](https://rustup.rs/) y las herramientas de compilación correspondientes a tu sistema. Para descargar el repositorio y abrir la interfaz:

```sh
git clone https://github.com/00faithlss/Icaro.git
cd Icaro
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

Actualizado el **11-10-2026**. Las casillas marcadas corresponden a funciones implementadas en el alcance indicado; una pantalla de demostración no cuenta como integración real. El orden de los pendientes representa las prioridades actuales.

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
- [x] Recuperación de instancias, grupos y ajustes de Java desde disco; renombrado sin mover la carpeta de juego.
- [x] Eliminación de instancias a la papelera local y recuperación mediante Deshacer.
- [x] Vanilla omite la pestaña y el contador de mods; la barra inferior de Jugar se oculta sin sesión o sin instancia seleccionada.
- [x] Explorador conectado a la carpeta real: árbol, tamaños, vista previa de texto, abrir, copiar ruta, renombrar y mover archivos a su papelera.
- [x] Exportación ZIP con metadatos, selección de contenido, tamaños reales y selector nativo de destino; sin sobrescribir exportaciones anteriores.
- [x] Instalar Minecraft Vanilla al crear la instancia: cliente, bibliotecas, nativos, assets y configuración de registros verificados y reutilizables.
- [x] Detectar Java compatible o instalar el runtime oficial requerido por los metadatos de Minecraft; reutilizarlo y permitir selección por instancia.
- [x] Ejecutar Vanilla con la cuenta activa, memoria, Metaspace y argumentos de la instancia; capturar registros y código de salida, y detener el proceso con confirmación.
- [x] Conectar preparación, pausa, cancelación y reintento a la cola; estados reales de Jugar, Umbral, consola e informe de fallo con las composiciones existentes.
- [x] Primer lanzamiento real de Vanilla 26.3 en Windows confirmado por el usuario; los ajustes posteriores del Umbral requieren volver a probarse jugando.
- [ ] Validar el recorrido completo en Arch Linux y completar la revisión manual de cierre, reutilización y transición del Umbral en Windows.
- [ ] Instalar loaders compatibles y lanzar instancias con sus mods.
- [ ] Importar y duplicar instancias reales; cambiar Minecraft o loader con copia de seguridad y revisión de compatibilidad.
- [ ] Exportar modpacks MRPACK con la versión concreta de Minecraft y loader.
- [ ] Conectar recursos y shaders, su orden y activación a los archivos del juego.
- [ ] Leer mundos reales, crear y restaurar copias de seguridad y permitir jugar directamente a un mundo.
- [ ] Leer y gestionar capturas reales por instancia.
- [ ] Completar las vistas previas y la recuperación de archivos eliminados en el explorador.

### Catálogo y gestión de mods

- [x] Catálogo público de Modrinth con iconos, búsqueda, filtros de Minecraft y loader, orden y paginación.
- [x] Descripciones y changelogs formateados, galería con originales, creadores, licencias, fechas y estadísticas.
- [x] Ficha rápida, página completa, versiones compatibles y dependencias con sus nombres.
- [x] Volver conserva el catálogo, búsqueda, filtros, página y ficha anterior; la tienda permite regresar a los mods instalados.
- [x] Tienda de mods por instancia con categorías, filtro de lado y compatibilidad fijada a Minecraft y loader.
- [x] Lista de instalación con mods, dependencias y tamaño antes de descargar; selección de dependencias opcionales.
- [x] Instalación real de mods y dependencias obligatorias transitivas, con comprobación de compatibilidad, conflictos, tamaño y SHA-512.
- [x] Publicación conjunta de archivos preparados; una descarga fallida conserva los mods anteriores y los archivos locales ajenos a Ícaro.
- [x] Inventario persistente de mods, activación y desactivación, detección de actualizaciones compatibles y confirmación antes de actualizar.
- [x] Selección múltiple para activar, desactivar, actualizar y quitar; papelera de mods con Deshacer durante diez segundos.
- [ ] Instalar modpacks como nuevas instancias, incluidos sus archivos de configuración.
- [ ] Conectar la instalación de paquetes de recursos y shaders del catálogo.

### Próximas entregas, en orden

1. **Validar Vanilla con una cuenta real:** comprobar la llegada al menú del juego, su cierre y la reutilización de la instalación; repetir en Arch Linux.
2. **Instancias con loader:** instalar Fabric, NeoForge, Forge y Quilt en versiones compatibles y comprobar el lanzamiento con los mods ya gestionados.
3. **Importación y modpacks:** importar MRPACK/ZIP y otros launchers, duplicar instancias y exportar MRPACK. Completar el cambio de versión con copias de seguridad.
4. **Editor completo:** conectar mundos, recursos, shaders, capturas, integridad y recuperación de archivos; completar historial de descargas y seguimiento de la consola, usando las vistas del System Design.
5. **Cuenta y servicios adicionales:** completar Cuenta y Estudio de skin, servidores y proveedores opcionales.
6. **Primera publicación:** validar Windows y Arch Linux, rendimiento, teclado, ventanas angostas y empaquetado.

### Integraciones y validaciones restantes

- [ ] Integración opcional de CurseForge, sujeta al acceso a su API y a las condiciones de distribución.
- [ ] Conexión a servidores y sincronización de mods con un servidor.
- [ ] Red privada, bandeja del sistema y actualización del propio launcher.
- [ ] Verificación y reparación de instancias y revisión de archivos de origen desconocido.
- [ ] Completar adaptación a ventanas angostas, escala, foco y navegación por teclado según el diseño original.
- [ ] Persistir las preferencias generales y conectar sus efectos reales: idioma, inicio con el sistema y comportamiento al jugar/cerrar.
- [ ] Validación en Windows y Arch Linux, empaquetado y publicación de la primera versión funcional.

El [plan de integración](docs/integracion-real.md) detalla los servicios y sus condiciones de aceptación. El [Roadmap del System Design](docs/roadmap-system-design.md) indica qué vistas están conectadas, cuáles siguen como maqueta y qué falta completar de cada componente.

## Componentes de terceros, licencias y fuentes

Ícaro utiliza los siguientes componentes directos. Las versiones indicadas corresponden al `Cargo.lock` actual; cada proyecto conserva su licencia y sus avisos de autoría.

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
- [Acerca de Ícaro](docs/acerca-de.md)
- [Desarrollo y estructura del proyecto](docs/desarrollo.md)
- [Roadmap del System Design](docs/roadmap-system-design.md)
- [Comparación con el System Design](docs/revision-system-design.md)
- [Cuenta y estudio de skin](docs/cuenta.md)
- [Integración real, autenticación y seguridad](docs/integracion-real.md)
- [Inicio de sesión y catálogo](docs/login-y-catalogo.md)
- [Créditos](docs/creditos.md)

## Licencia

[![Licencia: GPL-3.0-only](https://img.shields.io/badge/License-GPL--3.0--only-red.svg)](LICENSE)

El código de Ícaro está disponible bajo **GNU General Public License v3.0 only**, identificada como `GPL-3.0-only`. El texto completo se encuentra en [LICENSE](LICENSE).

Las bibliotecas, tipografías, iconos e imágenes de terceros conservan sus licencias, detalladas en [Componentes de terceros](#componentes-de-terceros-licencias-y-fuentes). La fotografía utilizada para el icono de Ícaro se atribuye a **© Marie-Lan Nguyen / Wikimedia Commons** bajo [CC BY 2.5](https://creativecommons.org/licenses/by/2.5/).

[mit]: https://opensource.org/license/mit
[apache]: https://www.apache.org/licenses/LICENSE-2.0
