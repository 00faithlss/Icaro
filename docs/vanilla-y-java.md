# Vanilla y Java

La creación de una instancia Vanilla prepara Minecraft y su Java automáticamente. Las instancias Vanilla anteriores se preparan al pulsar **Jugar**. La descarga no necesita una cuenta; para ejecutar el juego se requiere una sesión de Minecraft Java activa.

## Probar el recorrido

```powershell
cargo run -p icaro-ui --example launcher
```

1. Inicia sesión con Microsoft.
2. Crea una instancia seleccionando Minecraft y **Vanilla** en el asistente original.
3. Mira su preparación en **Descargas**. Puedes pausar, reanudar, cancelar y reintentar.
4. Abre la instancia y pulsa **Jugar**. El Umbral muestra registros reales y la consola recibe la salida de Java.
5. Al terminar, la instancia vuelve a estar disponible. Al jugar de nuevo, Ícaro comprueba los archivos guardados y descarga solamente los ausentes o dañados.
6. Mientras el juego está en ejecución, pulsa su botón: muestra **Cerrar juego** durante tres segundos. Un segundo clic lo detiene.

Las instancias Fabric, NeoForge, Forge y Quilt conservan la gestión de mods, pero su instalación y lanzamiento quedan para la siguiente entrega. El launcher lo indica al intentar jugar.

## Archivos guardados

En Windows los datos están en `%APPDATA%\Icaro`; en Linux, en `$XDG_DATA_HOME/Icaro` o `~/.local/share/Icaro`.

- `instances/<id>/instance.json`: datos de la instancia.
- `instances/<id>/.icaro-minecraft`: cliente, bibliotecas, nativos, assets, metadatos y marca de instalación.
- `instances/<id>/minecraft`: mundos, configuración, capturas y registros del usuario.
- `java`: runtimes oficiales gestionados y sus manifiestos de comprobación.

La comprobación inicial no vuelve a descargar una instalación válida. Los archivos verificados de Minecraft se conservan tras una cancelación; el runtime Java se reutiliza una vez publicado completo. Los binarios privados de Minecraft quedan fuera de las exportaciones ZIP.

El botón listo muestra **Jugar**, sin la memoria asignada. Al pulsarlo, la comprobación de los archivos y los componentes guardados muestra **Preparando**. **Instalando** se reserva para las descargas de archivos que faltan o no pasan la comprobación de integridad.

El indicador de descargas de la barra lateral aparece únicamente durante una descarga real. La comprobación y preparación de una instalación guardada no se cuentan como descargas; el indicador muestra el porcentaje disponible, sin inventar una velocidad de transferencia.

## Java y proceso

**Automático** lee la versión mayor y el componente de Java de los metadatos oficiales de Minecraft. Comprueba la copia gestionada y los candidatos del sistema; si faltan, descarga el runtime oficial. Una selección manual incompatible produce un error para corregirla desde la pestaña **Java**.

La pestaña mantiene su diseño original y muestra los runtimes gestionados guardados y el Java usado por las instancias preparadas durante la sesión. La memoria mínima, máxima, Metaspace y los argumentos adicionales se aplican al proceso sin pasar por un intérprete de comandos. La resolución y pantalla completa se aplican desde el editor abierto; su persistencia completa todavía está pendiente.

El launcher captura stdout y stderr, limita el registro en memoria y oculta el token de Minecraft. Una salida anormal abre el informe de fallo original con el código y los registros reales; si no se conoce la causa, no se atribuye a un mod ficticio. Cerrar el launcher o cerrar sesión detiene los procesos que controla. **Mantener abierto** y **Minimizar** están conectados; el modo de cerrar el launcher conservando el juego requiere la futura integración de bandeja.

Los eventos XML de Log4j se muestran como mensajes legibles, con su nivel, hilo y excepciones. En Windows el Umbral permanece hasta detectar una ventana visible de Minecraft del proceso iniciado, después de completar su entrada. La detección admite GLFW, SDL y LWJGL; iniciar Java o agotar el tiempo de observación no cuenta como tener el juego listo. Volver retira la capa sin detener Minecraft. La vista del launcher permanece montada debajo para evitar que vuelva a cargar al terminar la transición. La detección equivalente en Linux sigue pendiente.

La salida del Umbral usa un fundido de la capa completa de 200 ms (120 ms con reducir movimiento), sin recorte vertical. La consola reutiliza el panel original del diseño, con filtros independientes, búsqueda resaltada, autoscroll que se pausa al subir, copiar/limpiar, selector de instancia y exportación. Sin juego permite abrir el último `minecraft/logs/latest.log` guardado. El visor dibuja únicamente las filas visibles del registro.

Las capturas hechas con **F2** quedan en `instances/<id>/minecraft/screenshots`. La galería global y la pestaña Capturas las leen en segundo plano y se actualizan cada dos segundos mientras están abiertas. Galería y visor usan el archivo original, sin reducir su resolución. En Windows se puede copiar la imagen o la selección de archivos y abrir su carpeta. La eliminación real aún está pendiente; su acción informa que el archivo se conserva.

## Validación reproducible

```powershell
cargo test -j 1 -p icaro-core --lib
cargo test -j 1 -p icaro-ui --example launcher
cargo test -j 1 -p icaro-core minecraft::tests::metadatos_oficiales_actuales_y_legacy --lib -- --ignored --nocapture
cargo test -j 1 -p icaro-core minecraft::tests::instala_vanilla_real_y_reutiliza_archivos_verificados --lib -- --ignored --nocapture
cargo test -j 1 -p icaro-core java::tests::runtime_oficial_descarga_y_reutiliza_java --lib -- --ignored --nocapture
```

Las pruebas de red usan carpetas temporales aisladas, sin cuentas ni instancias del usuario. La instalación completa comprueba cliente, assets, nativos, reutilización y reparación de un JAR alterado. La prueba de Java descarga el runtime oficial, ejecuta `java -version` y comprueba su reutilización. Las pruebas del proceso usan un ejecutable local de prueba para verificar argumentos, registros y cancelación. La llegada al menú del juego con una cuenta real se valida manualmente.

Validación del 11 de octubre de 2026: 65 pruebas del núcleo, 49 de la biblioteca de interfaz y 22 del launcher aprobadas. Cubren registros XML, espera de la ventana, preparación de caché sin anunciar descarga, capturas originales y preservación del visor, búsqueda Unicode, virtualización y salida del Umbral sin cortina. En la validación anterior también pasaron las pruebas de red de metadatos actuales y antiguos, instalación completa de Vanilla 1.21.1 con reutilización y reparación, y descarga y reutilización de Java 21. Se generaron las vistas de consola y su estado vacío, capturas reales y visor, y salida del Umbral en Tinta y Piedra. El usuario confirmó previamente que Vanilla 26.3 abre y permanece funcionando en Windows; el recorrido final de arranque corregido todavía debe volver a probarse con el juego real.
