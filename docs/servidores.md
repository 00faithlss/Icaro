# Servidores y ZeroTier

La sección Servidores usa las tarjetas existentes con datos guardados y consultas reales. La demostración sigue separada mediante ICARO_DEMO=1.

## Guardar y conectar

1. Abre Servidores y pulsa **Agregar servidor**.
2. Escribe un nombre y una dirección: play.example.net, 127.0.0.1:25566 o [::1]:25565. Sin puerto se usa 25565.
3. Vincula la instancia con la versión de Minecraft y loader que necesite el servidor.
4. Guarda y pulsa **Jugar**. Se utiliza la sesión de Microsoft, Java, memoria y argumentos de esa instancia.

Editar y quitar actúan sobre la lista; quitar un servidor conserva la instancia y sus archivos. Subir/Bajar guardan el orden. Un servidor sin instancia vinculada abre el editor al intentar jugar. Si la instancia está ocupada, se conserva su actividad y se pide esperar.

Las versiones cuyo perfil oficial anuncia Quick Play usan --quickPlayMultiplayer con la dirección original; las anteriores usan --server y --port. La dirección se entrega como argumento literal, sin shell. El destino pertenece a ese intento de conexión: Jugar normalmente desde Instancias no hereda el servidor.

El lanzamiento hacia el servidor está implementado y probado mediante construcción de argumentos; falta confirmar la entrada efectiva con una cuenta y un servidor real. La consulta de estado no demuestra compatibilidad de mods o loader.

## Estado y persistencia

**Actualizar estado** consulta los servidores guardados: versión publicada, jugadores, mensaje y latencia. Se limitan las consultas a cuatro simultáneas, cinco segundos por servidor y 256 KiB por respuesta. Actualizar descarta respuestas de la consulta anterior. Un error conserva el servidor y permite intentar jugar, porque algunos servidores deshabilitan la consulta.

La lista se guarda en Icaro/servidores.json, esquema 1, con identificadores estables y hasta 64 entradas. Direcciones repetidas se rechazan. Una lista dañada o de esquema desconocido se conserva y bloquea cambios; **Releer lista** permite cargarla de nuevo después de corregir el archivo.

La consulta de estado admite DNS ordinario e IPv4/IPv6. Resolver registros DNS SRV para esta consulta sigue pendiente. El lanzamiento Quick Play conserva el nombre original para que Minecraft resuelva la conexión.

Los nombres de versión pueden ser etiquetas de proxies. La interfaz compara contra versiones oficiales solo cuando el servidor publica un nombre exacto reconocido. No inventa una familia de loader o una lista de mods a partir de ese texto.

## Detección de ZeroTier

La detección busca la CLI oficial en PATH y ubicaciones habituales de instalación; consulta info y listnetworks en JSON, con límites de tiempo y salida. En Windows los procesos auxiliares no abren una consola.

Se distinguen instalación ausente, servicio no consultable y respuesta válida. Cuando el servicio permite consultar, se muestra su versión, conexión, redes, autorización e IP asignada. Si el servicio está detenido o faltan permisos, se explica que no pudo consultarse; no se presentan datos de ejemplo como reales.

La detección no une ni abandona redes, no instala ZeroTier ni cambia sus permisos. Estos controles y la administración de redes requieren la siguiente etapa. Los servidores directos funcionan sin ZeroTier.

Consulta local de diagnóstico, sin modificar redes:

    cargo run -j 1 -p icaro-core --example detectar_zerotier

El funcionamiento de la CLI y la autorización de redes se describen en la [documentación oficial de ZeroTier](https://docs.zerotier.com/cli/). Los argumentos Quick Play están documentados en las [notas oficiales de Minecraft 1.20](https://feedback.minecraft.net/hc/en-us/articles/16499677456781-Minecraft-Java-Edition-1-20-Trails-Tales).

## Sincronización pendiente

Se necesita un manifiesto publicado por el servidor o anfitrión con versiones, hashes y fuentes de los mods. La consulta de estado de Minecraft no proporciona ese inventario. El flujo debe revisar diferencias, conservar mods exclusivos de cliente y archivos locales, preparar cambios con respaldo y recuperarse de cancelaciones o errores.

## Verificación

Las pruebas usan directorios temporales y un servidor TCP local: persistencia y colisiones, direcciones inválidas, consulta y pong, argumentos de conexión con memoria y perfiles modernos/anteriores, resultados tardíos y conservación de la lista ante fallo de guardado. Las capturas usan fixtures aislados, sin cuentas ni redes del usuario.

    cargo test -j 1 -p icaro-core servidores::tests --lib
    cargo test -j 1 -p icaro-core zerotier::tests --lib
    cargo test -j 1 -p icaro-core conectar_servidor_respeta_perfil --lib
    cargo test -j 1 -p icaro-ui --example launcher servidores_reales::tests -- --test-threads=1

La validación con equipos reales en Windows y Arch Linux sigue como tarea separada del roadmap.
