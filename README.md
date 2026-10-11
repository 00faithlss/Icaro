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
> El ejemplo conecta el inicio de sesión con Microsoft y el catálogo público de Modrinth. Las instancias, capturas y descargas restantes usan datos de demostración; la instalación y el lanzamiento del juego están pendientes.

El núcleo consulta el manifiesto oficial de versiones de Minecraft y verifica el SHA-1 del JSON de una versión.

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

Las casillas marcadas corresponden a funciones implementadas en el alcance indicado. El orden representa las prioridades actuales y puede cambiar durante el desarrollo.

### Base e interfaz

- [x] Estructura en Rust: núcleo, herramienta de línea de comandos e interfaz con iced.
- [x] Consulta del manifiesto de Mojang y verificación del JSON de una versión.
- [x] Interfaz de demostración con navegación, temas claro y oscuro y componentes compartidos.
- [x] Grabados de mayor resolución, caché de imágenes, visor y mejoras de contraste y desplazamiento.
- [x] Búsqueda global sobre los datos de demostración y avisos temporales.
- [x] Flujo de login con PKCE, retorno local, intercambio con Xbox y Minecraft y almacenamiento de una sesión en el sistema; probado con una cuenta real en Windows.
- [x] Inicio por código de vinculación de Microsoft en la página del launcher, con copia, vencimiento y cancelación; solicitud de código real probada, vinculación completa pendiente de prueba manual.
- [x] Catálogo público de Modrinth: búsqueda, paginación, fichas y consulta de versiones compatibles.

### Primera versión funcional

- [x] Autenticación real con Microsoft y validación del perfil y acceso a Minecraft.
- [ ] Protección de credenciales, renovación de sesión y cierre de sesión completo.
- [ ] Creación de instancias reales, instalación verificada de Minecraft Vanilla y lanzamiento con Java.
- [ ] Instalación de loaders: Fabric, NeoForge, Forge y Quilt.
- [ ] Catálogo de Modrinth con búsqueda, compatibilidad, dependencias, instalación y actualizaciones dentro de cada instancia.
- [ ] Importación e instalación real de modpacks.
- [ ] Consola conectada al proceso del juego y cola de descargas real.
- [ ] Datos reales de cuenta, skins y capturas.

### Siguientes etapas

- [ ] Integración opcional de CurseForge, sujeta al acceso a su API y a las condiciones de distribución.
- [ ] Conexión a servidores y sincronización de mods con un servidor.
- [ ] Validación en Windows y Arch Linux, empaquetado y publicación de la primera versión funcional.

El [plan de integración](docs/integracion-real.md) desarrolla el flujo de autenticación, la protección de credenciales y las comprobaciones necesarias antes de dar cada etapa por terminada.

## Componentes de terceros, licencias y fuentes

Ícaro utiliza los siguientes componentes directos. Las versiones indicadas corresponden al `Cargo.lock` actual; cada proyecto conserva su licencia y sus avisos de autoría.

| Componente | Versión | Uso | Licencia declarada |
| --- | --- | --- | --- |
| [iced](https://github.com/iced-rs/iced) | 0.13.1 | Interfaz de escritorio | [MIT][mit] |
| [image](https://github.com/image-rs/image) | 0.24.9 | Preparación y decodificación de imágenes | [MIT][mit] OR [Apache-2.0][apache] |
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
| [sha2 · RustCrypto](https://github.com/RustCrypto/hashes) | 0.10.9 | SHA-256 del desafío PKCE | [MIT][mit] OR [Apache-2.0][apache] |
| [zeroize](https://github.com/RustCrypto/utils) | 1.9.1 | Limpieza de buffers de credenciales | [Apache-2.0][apache] OR [MIT][mit] |
| [tokio-util](https://github.com/tokio-rs/tokio) | 0.7.20 | Cancelación de autenticación y consultas | [MIT][mit] |
| [webbrowser](https://github.com/amodm/webbrowser-rs) | 1.2.4 | Apertura del navegador del sistema | [MIT][mit] OR [Apache-2.0][apache] |
| [keyring](https://github.com/hwchen/keyring-rs) | 3.6.3 | Credential Manager en Windows y Secret Service en Linux | [MIT][mit] OR [Apache-2.0][apache] |

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
