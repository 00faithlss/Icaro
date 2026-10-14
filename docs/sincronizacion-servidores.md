# Sincronización con el servidor de un amigo

El servicio lee directamente `mods` de la carpeta del servidor dedicado que ejecuta el PC del anfitrión. No requiere una instancia cliente de Ícaro ni instalar el launcher en ese equipo. Al pulsar Jugar, con **Sincronizar mods al iniciar** activado, el launcher consulta ese índice por ZeroTier, muestra las diferencias en la pantalla original `SincronizarMods` y descarga archivos exactos desde Modrinth o los JAR propios publicados por el anfitrión.

## Preparar al anfitrión

El anfitrión debe autorizar a sus amigos en ZeroTier y conocer la carpeta, versión de Minecraft y versión exacta del loader de su servidor. El servicio lee archivos y publica metadatos; no modifica el servidor, sus mundos ni sus configuraciones.

### Arch Linux del anfitrión

Con Rust estable actualizado y herramientas de compilación C instaladas, desde la raíz del código actualizado:

```bash
cargo build --locked --release -p icaro-core --example sync_host
./target/release/examples/sync_host --servidor "/ruta/absoluta/al/servidor" \
  --minecraft 1.21.1 --loader fabric --version-loader 0.19.5 \
  --escuchar 10.62.50.154:25566
```

Estos son los datos reales proporcionados por el anfitrión. Cambia únicamente la carpeta por la real. También puede arrancarse mediante `bash packaging/linux/iniciar-sincronizacion.sh /ruta/absoluta/al/servidor`; `ICARO_SYNC_BIN` permite indicar otra ubicación del ejecutable. En su script de arranque, iniciar este comando en segundo plano con `&` junto a Minecraft y conservar su PID para cerrarlo cuando termine el servidor.

El workflow de GitHub Actions compila y prueba el servicio en Linux x86_64 y guarda un `.tar.gz` como artefacto; sus resultados se consultan en Actions. El 14-10-2026 el anfitrión confirmó compilación en Arch con Rust 1.99, pruebas de sincronización y escucha en `10.62.50.154:25566`. Después confirmó que la corrección de descriptores solucionó Debugify y Weapon Master YDM y pasaron las 12 pruebas anteriores. La carpeta real reveló otro caso: Ksyxis 1.4.4 tiene metadatos de versiones extremas en Modrinth. La conversión de ese caso a archivo propio se comprobó con fixtures y el JAR real de Modrinth en carpetas temporales; debe repetirse la publicación completa en el PC del anfitrión. Para compilar los cambios en desarrollo, usa la rama `develop`; los paquetes públicos de 0.1.0 no incluyen estas funciones.

Cuando exista el paquete Linux del workflow, extraerlo y ejecutar `./icaro-sync-host` con los mismos argumentos anteriores; no necesita Rust para ejecutarse. No usar el `.exe` de Windows en Arch.

### Windows

El paquete portable y el instalador incluyen `icaro-sync-host.exe`. Puede copiarse por separado al PC del anfitrión Windows:

```powershell
& .\icaro-sync-host.exe --servidor "C:\Minecraft\Servidor" --minecraft "1.21.1" --loader "fabric" --version-loader "0.19.5" --escuchar "10.62.50.154:25566"
```

Reemplazar la IP del ejemplo por la IP de ZeroTier del anfitrión. El servicio escucha en **el puerto de Minecraft más uno**: Minecraft 25565 → servicio 25566; Minecraft 25570 → servicio 25571. Permitir ese puerto en el cortafuegos de la red/interfaz correspondiente, sin abrirlo en el router. Se rechazan direcciones públicas y comodines; loopback se admite para pruebas. Ctrl+C detiene el servicio. Mantenerlo abierto junto al servidor: comprueba cambios cada 10 segundos y rehace el índice cuando cambia el contenido. Durante la preparación o un error responde 503, sin entregar el índice anterior.

Las versiones se indican explícitamente para evitar adivinarlas por nombres de archivos o logs antiguos. Si actualizas Minecraft o el loader, reinicia el servicio con los argumentos nuevos. Para conservar el origen anterior, también admite `--datos "C:\...\Icaro" --instancia ID --escuchar IP:PUERTO`; ambos modos son excluyentes.

El servicio se inicia por esta herramienta. No se inventó una pantalla de publicación del anfitrión; su integración visual posterior necesita un componente entregado o aprobación del usuario.

## Quien juega

Agregar en Servidores la IP privada de ZeroTier y **el puerto de Minecraft**, vincular la instancia y activar **Sincronizar mods al iniciar**. Ícaro deriva el puerto del servicio. La primera implementación usa IP privada, no dominios públicos.

Consulta `GET /icaro/v1/mods`, comprueba esquema y límites, Minecraft y loader exactos, y pide a Modrinth las versiones indicadas. Verifica proyecto, compatibilidad, dependencias y SHA-512; no busca por nombres aproximados ni instala la última versión por suposición.

La pantalla original muestra agregar, actualizar/activar y quitar, tamaños y archivos conservados. **Sincronizar y jugar** aplica y conecta; **Solo sincronizar** aplica y vuelve; **Jugar sin sincronizar** exige confirmar el riesgo en el diálogo existente. Sin diferencias, verifica y registra el conjunto antes de conectar. Escape cancela la consulta o la preparación, esperando el acuse antes de desbloquear la instancia. Hay progreso general y por fila.

## Contenido y recuperación

El índice contiene Minecraft, loader, versión del loader y entradas de Modrinth. El esquema 2 añade archivos propios: identidad, nombre, versión, nombre del JAR, tamaño y SHA-512, más los avisos para la revisión. Requiere el cliente 0.2.0 actualizado. El servicio sirve únicamente esos JAR mediante `GET /icaro/v1/files/<SHA-512>`, desde su carpeta de mods y solo si el contenido sigue coincidiendo. No acepta rutas ni URLs externas; no comparte mundos, cuentas, credenciales ni archivos arbitrarios.

Identifica los JAR activos directamente en `mods` por su SHA-512 en Modrinth. No incluye archivos `.disabled`. Excluye proyectos sin soporte de cliente; un mod Fabric que declare `environment: server` también se excluye aunque sea privado. Un JAR válido no identificado genera un aviso y se publica como archivo propio: no bloquea el resto del índice. Los JAR anidados se rechazan para evitar omitirlos silenciosamente. La integración de CurseForge como tienda sigue pendiente; un JAR instalado en el servidor puede compartirse como archivo propio sin esa integración.

La compatibilidad de un archivo propio no se confirma por Modrinth: el anfitrión debe revisar cuáles requiere el cliente. `--excluir-archivo nombre.jar` permite excluir archivos concretos y puede repetirse. Se usan nombres exactos, sin rutas ni comodines. Para Fabric se conserva la identidad del mod del descriptor al cambiar su nombre de archivo; en otros JAR se deriva del nombre.

Si Modrinth reconoce el SHA-512 pero no enumera el Minecraft o loader del servidor, el JAR instalado se publica como archivo propio con un aviso. No se interpretan versiones extremas como un rango ni se cambia el archivo por otra versión. La regla es general para JAR universales y metadatos incompletos, no una excepción por nombre para Ksyxis. Primero se excluyen los proyectos sin soporte de cliente. Una respuesta que no confirme el hash sigue siendo un error.

La elección se propaga a los mods instalados que dependan obligatoriamente de ese proyecto, incluyendo cadenas de dependencias: también se comparten sus JAR exactos con avisos. Así no se vuelve a exigir una resolución estricta de Modrinth para el mismo archivo universal. Una dependencia ausente sin archivo instalado sigue requiriendo una versión válida del catálogo; no se inventa un archivo ni se ignora un conflicto de versiones fijadas.

La lectura de `fabric.mod.json` intenta primero JSON estricto. Si falla, sustituye caracteres de control sin escapar por espacios únicamente en una copia en memoria y reintenta. Los JAR y sus hashes no cambian. El aviso identifica el archivo normalizado. Si el descriptor sigue siendo ilegible o supera el límite de 128 KiB, se publica un aviso con el nombre del JAR y se mantiene la identificación por Modrinth o por nombre de archivo, sin bloquear toda la lista. No se supone que sea exclusivo de servidor cuando no puede confirmarse su entorno; el anfitrión puede excluirlo explícitamente.

La clasificación usa los metadatos de [Modrinth](https://docs.modrinth.com/api/operations/getproject/) y el campo `environment` del [descriptor oficial de Fabric](https://wiki.fabricmc.net/documentation:fabric_mod_json_spec).

Las dependencias obligatorias usan la versión instalada en el servidor cuando existe, respetando cualquier versión fijada por Modrinth. Si falta una dependencia de cliente, usa la versión exacta declarada; si solo declara proyecto, selecciona la release estable compatible más reciente y fija su versión y hash en el índice. Los conflictos impiden publicar. No instala nada en el anfitrión. Los cambios de archivos durante la consulta invalidan la preparación y se reintentan.

Si Minecraft responde pero la sincronización no, comprobar el servicio separado y su puerto: estar aprobado en ZeroTier no inicia `icaro-sync-host`. Desde otro equipo autorizado, `Invoke-RestMethod "http://IP_DEL_ANFITRION:25566/icaro/v1/mods"` debe devolver el índice. La comprobación integral entre los dos PCs sigue pendiente; las pruebas automáticas cubren carpeta dedicada, filtro de servidor, dependencias, cambios de archivos y publicación HTTP.

Comprueba cambios de la carpeta del servidor cada 10 segundos y vuelve a preparar el índice si cambian los archivos. Un conflicto, contenedor JAR inválido o cambio durante la consulta lo deja indisponible; los archivos no reconocidos y los descriptores ilegibles son avisos. No inspecciona ni controla el proceso de Minecraft.

Conserva archivos locales ajenos y mods de cliente. Solo puede quitar proyectos o JAR propios que esta instancia haya sincronizado previamente con ese mismo origen, y conserva los proyectos que Modrinth identifica como exclusivos de cliente. Los JAR propios modificados por el jugador no se sobrescriben ni eliminan. Cambiar de anfitrión no autoriza borrar el conjunto gestionado por otro.

Prepara los archivos en una carpeta separada, comprueba tamaño y SHA-512, revalida los archivos y perfil y publica el conjunto con el respaldo/recuperación del instalador de mods existente. Un fallo, cancelación, nombre ocupado o modificación local conserva los mods anteriores. Vuelve a consultar la revisión del índice antes de aplicar el plan. Bloquea las operaciones del launcher mientras aplica; no modifica una instancia ocupada por Minecraft.

## Por qué no basta el mensaje de Minecraft

Algunas versiones de Forge anuncian IDs y marcadores de compatibilidad; pueden truncar la información y no entregan hashes ni archivos de Modrinth. No existe una petición universal que proporcione el inventario instalable completo para todos los loaders. La comprobación al pulsar Jugar usa el servicio explícito. Fuentes: [código oficial de Forge](https://github.com/MinecraftForge/MinecraftForge/blob/1.20.x/src/main/java/net/minecraftforge/network/ServerStatusPing.java), [identificación por hash de Modrinth](https://docs.modrinth.com/api/operations/versionfromhash/) y [CLI de ZeroTier](https://docs.zerotier.com/cli/).

ZeroTier aporta conectividad y autorización de red; no publica el índice automáticamente. El índice viaja por HTTP dentro de la red privada, sin TLS ni autenticación adicional del servicio. Escuchar exclusivamente en la IP de ZeroTier elegida por el anfitrión. La revisión del usuario y las fuentes verificables de Modrinth siguen siendo necesarias.

## Verificación

Las pruebas cubren HTTP local, rutas/métodos rechazados, índices y perfiles inválidos, actualización, retirada de proyectos gestionados y JAR propios, archivos locales, hash incorrecto, cancelación y planes obsoletos. Una carpeta de prueba contiene 163 mods: 144 identificados y 19 propios; se publica el índice completo y sus avisos. También se prueban descripciones con LF/CRLF/tabulaciones sin escapar, descriptores truncados, escapes JSON válidos, límites y conservación de hashes. No son los archivos reales del anfitrión. La integración optativa con Fabric API real verifica la descarga final. La publicación del índice corregido con su carpeta real y la conexión de los clientes siguen pendientes.

```powershell
cargo test -j 1 -p icaro-core mods::sincronizacion --lib
cargo test -j 1 -p icaro-core modrinth_real_publica_indice --lib -- --ignored
cargo test --locked -j 1 -p icaro-core ksyxis_real_no_bloquea --lib -- --ignored
cargo test -j 1 -p icaro-ui --example launcher sincronizacion_real -- --test-threads=1
```

Falta probar con el PC del anfitrión y otro equipo autorizado: cortafuegos, cambios reales del servidor/instancia cliente, sincronización, entrada al juego y reconexión. La validación en Windows y Arch Linux conserva su casilla pendiente.
