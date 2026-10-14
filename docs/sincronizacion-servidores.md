# Sincronización con el servidor de un amigo

Se eligió un servicio en el PC del anfitrión, accesible por ZeroTier, que publica el índice de una instancia de cliente de Ícaro. Al pulsar Jugar, con **Sincronizar mods al iniciar** activado, el launcher consulta ese índice, muestra las diferencias en la pantalla original `SincronizarMods` y descarga archivos exactos desde Modrinth.

## Preparar al anfitrión

El anfitrión mantiene una instancia de cliente con el Minecraft, familia y versión exacta del loader y mods necesarios para entrar a su servidor dedicado. La carpeta del servidor puede incluir mods exclusivos de servidor y no sustituye esa instancia de cliente. Debe autorizar a sus amigos en ZeroTier.

El paquete portable y el instalador incluyen `icaro-sync-host.exe`. Obtener el `id` de la instancia en su `instance.json` y ejecutar:

```powershell
& .\icaro-sync-host.exe --datos "$env:APPDATA\Icaro" --instancia ID_DE_INSTANCE_JSON --escuchar 10.147.17.1:25566
```

Reemplazar la IP del ejemplo por la IP de ZeroTier del anfitrión. El servicio escucha en **el puerto de Minecraft más uno**: Minecraft 25565 → servicio 25566; Minecraft 25570 → servicio 25571. Permitir ese puerto en el cortafuegos de la red/interfaz correspondiente, sin abrirlo en el router. Se rechazan direcciones públicas y comodines; loopback se admite para pruebas. Ctrl+C detiene el servicio.

El servicio se inicia por esta herramienta. No se inventó una pantalla de publicación del anfitrión; su integración visual posterior necesita un componente entregado o aprobación del usuario.

## Quien juega

Agregar en Servidores la IP privada de ZeroTier y **el puerto de Minecraft**, vincular la instancia y activar **Sincronizar mods al iniciar**. Ícaro deriva el puerto del servicio. La primera implementación usa IP privada, no dominios públicos.

Consulta `GET /icaro/v1/mods`, comprueba esquema y límites, Minecraft y loader exactos, y pide a Modrinth las versiones indicadas. Verifica proyecto, compatibilidad, dependencias y SHA-512; no busca por nombres aproximados ni instala la última versión por suposición.

La pantalla original muestra agregar, actualizar/activar y quitar, tamaños y archivos conservados. **Sincronizar y jugar** aplica y conecta; **Solo sincronizar** aplica y vuelve; **Jugar sin sincronizar** exige confirmar el riesgo en el diálogo existente. Sin diferencias, verifica y registra el conjunto antes de conectar. Escape cancela la consulta o la preparación, esperando el acuse antes de desbloquear la instancia. Hay progreso general y por fila.

## Contenido y recuperación

El índice contiene solo Minecraft, loader, versión del loader, proyecto, versión y hash de cada mod. El servicio entrega únicamente el índice: no comparte JAR, archivos arbitrarios, mundos, capturas, cuentas, credenciales ni rutas del anfitrión, ni acepta órdenes de instalación.

Identifica los archivos activos por su contenido en Modrinth. Excluye mods exclusivos de cliente/servidor de la base común e incluye dependencias de cliente obligatorias de un mod común. Un archivo activo no identificado en Modrinth impide publicar un índice incompleto. CurseForge y mods privados requieren proveedores adicionales.

Comprueba cambios de la instancia cliente cada 10 segundos y vuelve a preparar el índice si cambian los mods o el perfil. Un fallo lo deja indisponible. No inspecciona ni controla el proceso del servidor dedicado; el anfitrión debe mantener consistente su instancia cliente compartida.

Conserva archivos locales ajenos y mods de cliente. Solo puede quitar proyectos que esta instancia haya sincronizado previamente con ese mismo origen, y conserva los que Modrinth identifica como exclusivos de cliente. Cambiar de anfitrión no autoriza borrar el conjunto gestionado por otro.

Prepara los archivos en una carpeta separada, comprueba tamaño y SHA-512, revalida los archivos y perfil y publica el conjunto con el respaldo/recuperación del instalador de mods existente. Un fallo, cancelación, nombre ocupado o modificación local conserva los mods anteriores. Vuelve a consultar la revisión del índice antes de aplicar el plan. Bloquea las operaciones del launcher mientras aplica; no modifica una instancia ocupada por Minecraft.

## Por qué no basta el mensaje de Minecraft

Algunas versiones de Forge anuncian IDs y marcadores de compatibilidad; pueden truncar la información y no entregan hashes ni archivos de Modrinth. No existe una petición universal que proporcione el inventario instalable completo para todos los loaders. La comprobación al pulsar Jugar usa el servicio explícito. Fuentes: [código oficial de Forge](https://github.com/MinecraftForge/MinecraftForge/blob/1.20.x/src/main/java/net/minecraftforge/network/ServerStatusPing.java), [identificación por hash de Modrinth](https://docs.modrinth.com/api/operations/versionfromhash/) y [CLI de ZeroTier](https://docs.zerotier.com/cli/).

ZeroTier aporta conectividad y autorización de red; no publica el índice automáticamente. El índice viaja por HTTP dentro de la red privada, sin TLS ni autenticación adicional del servicio. Escuchar exclusivamente en la IP de ZeroTier elegida por el anfitrión. La revisión del usuario y las fuentes verificables de Modrinth siguen siendo necesarias.

## Verificación

Las pruebas cubren HTTP local, rutas/métodos rechazados, índices y perfiles inválidos, actualización, retirada de proyectos gestionados, archivos locales, hash incorrecto, cancelación y planes obsoletos. La prueba de integración optativa instala Fabric API y Sodium reales en una instancia temporal, excluye Sodium de la base común y sincroniza Fabric API hacia otra instancia por HTTP local, comparando hashes finales.

```powershell
cargo test -j 1 -p icaro-core mods::sincronizacion --lib
cargo test -j 1 -p icaro-core modrinth_real_publica_indice --lib -- --ignored
cargo test -j 1 -p icaro-ui --example launcher sincronizacion_real -- --test-threads=1
```

Falta probar con el PC del anfitrión y otro equipo autorizado: cortafuegos, cambios reales del servidor/instancia cliente, sincronización, entrada al juego y reconexión. La validación en Windows y Arch Linux conserva su casilla pendiente.
