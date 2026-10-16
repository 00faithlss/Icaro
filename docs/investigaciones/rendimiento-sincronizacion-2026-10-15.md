# Rendimiento de sincronización, descargas y actualizaciones

Investigación del 15-10-2026. Revisión del árbol local basado en `b740ea9`, incluyendo las correcciones todavía sin publicar. Este informe contiene mediciones y propuestas; no afirma que las optimizaciones ya estén implementadas.

## Resultado principal

La espera de varios minutos con **cero bytes por descargar** se explica principalmente por dos factores combinados:

1. El ejecutable entregado para estas pruebas está en `target/debug/examples/launcher.exe`: sus hashes se calculan sin optimizar.
2. Comprobar y aplicar una sincronización normalmente calcula cuatro veces los hashes de toda la carpeta, además de copiarla completa y verificar otros archivos individuales.

La instancia observada contiene **259 archivos y 547,03 MiB** después de retirar las cinco copias anteriores. Esas cinco copias ocupaban aproximadamente 3,92 MiB. El trabajo debería depender principalmente de esos cinco archivos, no repetir cuatro lecturas criptográficas de unos 547 MiB.

**La mayor mejora inmediata es corregir la compilación de prueba; la mejora estructural es convertir la operación en incremental.** Aumentar las descargas simultáneas no resuelve una retirada sin descargas.

## Método y límites

- Windows, NTFS, Ryzen 5 5600X, 6 núcleos/12 procesadores lógicos; Rust/Cargo 1.99.0; `sha2 0.10.9`, igual que el proyecto.
- Hashes: tres muestras de 32 MiB en RAM, bloques de 64 KiB y comparación del digest entre variantes.
- Disco: llamadas Rust `fs::copy`, `fs::hard_link`, `fs::rename`, metadatos y SHA-512 sobre el corpus local. Una pasada con cachés calientes; no es una medición del ancho de banda físico del disco.
- Red: tres consultas pequeñas al manifiesto, ocho GET de JAR pequeños al anfitrión y una consulta a la API pública de Modrinth. Sin pruebas de carga ni búsqueda del límite mediante errores.
- Las copias, movimientos y enlaces del benchmark se hicieron únicamente dentro de `target`. La instancia y el servidor no se modificaron.
- No se midió el launcher completo con un perfil de CPU, ni el equipo del amigo, Linux, HDD o una conexión lenta. Los resultados de componentes no equivalen a una aceleración idéntica de toda la aplicación.

Los [datos completos](datos/rendimiento-sincronizacion-2026-10-15.json) incluyen muestras, cabeceras observadas y limitaciones. El [banco reproducible](../../scripts/rendimiento_sincronizacion/src/main.rs) está separado del workspace de producción.

## Mediciones

### Hashing: aquí está el cuello de botella actual

| Variante | Mediana SHA-512 | Tiempo por 32 MiB |
| --- | ---: | ---: |
| Desarrollo predeterminado | 10,67 MiB/s | 3,000 s |
| Desarrollo, optimizando solamente `sha2` | 636,11 MiB/s | 0,050 s |
| Release, `opt-level=3` | 689,57 MiB/s | 0,046 s |
| Release con opciones del proyecto: `z`, LTO, una unidad de código, abort | 707,15 MiB/s | 0,045 s |

La diferencia de este componente es **66,3 veces** entre desarrollo y el perfil release del proyecto. Optimizar únicamente la dependencia `sha2` en desarrollo dio **59,6 veces** en el microbenchmark. No hace falta cambiar de algoritmo ni desactivar la integridad para obtener esa mejora.

Para SHA-256, utilizado por el actualizador, las medianas fueron **45,77 MiB/s en desarrollo** y **433,06 MiB/s con el perfil release del proyecto**. Son algoritmos y resultados distintos: no debe aplicarse automáticamente el factor de SHA-512 al actualizador.

Extrapolación, **no tiempo completo medido del launcher**: 547,03 / 10,67 equivale a unos **51,3 s por barrido**. Cuatro barridos suponen unos **205 s de cálculo**, sin contar red, copias y verificaciones adicionales. Dos de esos barridos están dentro de la aplicación del plan, por lo que incluso después de mostrar la lista todavía puede esperar alrededor de 103 s por ese componente en desarrollo.

Cargo usa `opt-level=0` por defecto en desarrollo y permite optimizar dependencias individualmente. El perfil release actual ya está optimizado; no hay evidencia aquí para sustituir `z` por `3`. [Documentación de perfiles de Cargo](https://doc.rust-lang.org/cargo/reference/profiles.html).

### Disco y análisis

| Operación | Resultado observado |
| --- | ---: |
| Consultar metadatos de los 259 archivos | 17,7 ms por recorrido, promedio de 100 |
| Copiar los 547,03 MiB con `fs::copy` | 0,309 s |
| Calcular SHA-512 de toda la carpeta, optimizado | 0,888 s |
| Crear 259 enlaces duros dentro del sandbox | 0,149 s |
| Mover cinco archivos dentro del sandbox | 1,12 ms |
| Leer descriptores Fabric con Python, referencia auxiliar | 0,480 s; 255 reconocidos |

El último resultado no mide el parser Rust. El movimiento de cinco archivos tampoco incluye un journal durable, validación, interfaz o recuperación. Estas cifras no son una promesa de duración final. En este equipo y con datos calientes, copiar no fue el coste dominante; sigue siendo trabajo y espacio innecesarios cuando solo se quitan cinco JAR.

### Red y límites

| Prueba | Resultado observado |
| --- | ---: |
| Manifiesto del anfitrión, 32.587 bytes | 65–84 ms; mediana 75,8 ms |
| Cuatro JAR, 2.851.990 bytes en total, serie | 0,691 s |
| Los mismos cuatro JAR, dos transferencias simultáneas | 0,431 s |
| API Modrinth, una consulta de categorías | 0,853 s, HTTP 200 |

La segunda descarga se midió después de la primera y puede beneficiarse de cachés. La muestra es pequeña: indica que vale la pena probar concurrencia acotada, no demuestra el ancho de banda sostenido ni una mejora universal del 60 %.

Modrinth documenta **300 solicitudes/minuto por IP**, iguales con o sin token, y las cabeceras `X-Ratelimit-Limit`, `Remaining` y `Reset`. La consulta real devolvió límite 300, restantes 299 y reset 0. Deben interpretarse las cabeceras de cada respuesta, sin dividir por un reset cero ni asumir que el límite nunca cambiará. [Reglas oficiales de la API](https://docs.modrinth.com/api/).

## Qué hace actualmente el código

| Zona | Evidencia | Consecuencia |
| --- | --- | --- |
| Análisis local | `sincronizacion.rs`: `huella`/`fingerprint`; inicio y final de `preparar` | Dos lecturas completas SHA-512 para revisar |
| Aplicación | `aplicar`: `fingerprint` inicial/final y `copiar(&dir, &stage, ...)` | Otros dos barridos y copia completa incluso para quitar |
| JAR propios | Se comprueba nuevamente el hash de cada destino existente | Lecturas adicionales de archivos ya reconocidos |
| Retirada de Mods | Existe `.icaro-transactions` y movimiento por archivo en `retirar_con_hash` | Ya hay una base local para una transacción incremental |
| Interfaz | Progreso basado en todos los mods del índice, más retiradas | El denominador no representa solo el trabajo pendiente |
| Respaldos de duplicados | El bucle de respaldo no emite el progreso de cada retirada | Una eliminación real puede mantener la barra casi vacía |
| Trabajo asíncrono | Lecturas, copias, ZIP y hashes síncronos dentro de futuros | Puede retrasar progreso, cancelación y otras tareas del executor |
| Modrinth | Lotes, caché persistente, caché negativa y cooldown compartido | Hay mecanismos útiles que deben conservarse |
| Caché API | Mutex de metadatos retenido mientras se espera la red | Un fallo lento puede bloquear consultas independientes y aciertos de caché |
| Presupuesto API | Se procesa `429`, `Retry-After` y reset; no `Remaining` preventivo | Se frena después de agotar el presupuesto |
| Descargas de mods/MRPACK | Bucles principalmente secuenciales; cuerpos en `Vec<u8>` | Latencia acumulada y memoria proporcional al tamaño por transferencia |
| Minecraft | Ya tiene hasta ocho tareas y hash durante la descarga | No debe tratarse como si todo el launcher descargara en serie |
| Servicio del anfitrión | Ocho slots, archivo completo en RAM, hash antes de responder y `Connection: close` | Reconexiones, memoria elevada y posible trabajo criptográfico por petición |
| Sondeo del anfitrión | Cada diez segundos vuelve a consultar una huella con hashes | Relee todo el conjunto incluso cuando nada cambió |
| Actualizador | Ya verifica durante la descarga y recupera un paquete completo válido | Conservar esta mejora; falta reanudación de parciales y mejor diagnóstico por fase |

Puntos de implementación: [sincronización](../../crates/core/src/mods/sincronizacion.rs), [operaciones de mods](../../crates/core/src/mods.rs), [Modrinth](../../crates/core/src/modrinth.rs), [caché](../../crates/core/src/modrinth/cache.rs), [host](../../crates/core/examples/sync_host.rs), [modpacks](../../crates/core/src/modpacks.rs), [Minecraft](../../crates/core/src/minecraft.rs), [actualizador](../../crates/core/src/actualizacion.rs) e [interfaz de sincronización](../../crates/ui/examples/launcher_aux/sincronizacion_real.rs).

## Arquitectura propuesta

### 1. Inventario incremental de archivos y descriptores

Guardar por archivo identidad del volumen/archivo, ruta, tamaño, fechas relevantes, SHA-512, ID interno, versión y entorno Fabric. Obtener los metadatos antes y después de calcular un hash; si cambian, descartar el resultado y reintentar de forma acotada.

Un archivo sin cambios reutiliza hash y descriptor. Uno nuevo, modificado, reemplazado o de identidad incierta se vuelve a leer. Las copias desactivadas forman parte del inventario, pero no deben descargarse o activarse por accidente.

**Tamaño y `mtime` solos no garantizan integridad**: un cambio puede conservarlos. Complementar con identidad de archivo y señales de cambio de la plataforma; conservar una verificación completa explícita y una ruta conservadora cuando no pueda validarse la caché. La caché es una optimización, no una fuente más fiable que el manifiesto y el contenido.

En Windows hay información por handle y notificaciones de directorio; si se pierden eventos o se desborda el buffer, hay que volver a enumerar. USN puede ser una mejora posterior, no un requisito que obligue al usuario a ejecutar como administrador. [Información de archivo](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfileinformationbyhandle), [notificaciones y desbordamientos](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-readdirectorychangesw), [change journal](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals).

### 2. Plan por diferencias y transacción por archivos afectados

El manifiesto del servidor fija el conjunto requerido. Comparar ID interno, origen y hash con el inventario local y generar únicamente agregar/actualizar/retirar. Sincronizar significa reproducir sus archivos exactos, no actualizar todo a la última versión del catálogo.

Para cinco retiradas:

1. Confirmar revisión del servidor y bloqueo de la instancia.
2. Revalidar los cinco archivos afectados; detectar cambios externos del inventario.
3. Escribir un journal de recuperación durable.
4. Mover únicamente esos cinco archivos a un respaldo del mismo volumen.
5. Publicar los metadatos actualizados y cerrar la transacción.

No copiar los otros 547 MiB ni descargar nada. Reutilizar y ampliar la lógica de `.icaro-transactions`; la recuperación debe restaurar solo lo movido, sin sobrescribir archivos que alguien haya cambiado después. Un lote de movimientos **no es atómico como conjunto**: necesita estados de journal, rollback y recuperación tras cierre abrupto. `rename` requiere origen y destino en el mismo sistema de archivos. [Semántica de Rust](https://doc.rust-lang.org/std/fs/fn.rename.html).

Para actualizaciones, descargar y verificar primero a temporales y después sustituir solo los destinos del plan. Mantener SHA-512, preservación de mods personales, validación del loader, dependencias exactas y defensa contra rutas/enlaces no permitidos.

### 3. Caché por contenido y red acotada

Usar un almacén de contenido verificado por SHA-512, compartido entre instalación, sincronización y modpacks. Si otra instancia ya descargó ese mismo archivo, reutilizarlo sin pedirlo otra vez. Acotar espacio, registrar referencias y aplicar limpieza LRU sin eliminar archivos en uso.

Descargar a disco por bloques, calcular hash mientras llegan y publicar únicamente al verificar tamaño y digest. No mantener cuatro u ocho JAR de 128 MiB completos en memoria. Punto inicial para probar: dos transferencias al anfitrión, cuatro al CDN y uno o dos trabajos de hashing; ajustar según medidas, presión de memoria y respuestas del servidor. No son límites oficiales ni valores óptimos demostrados.

Separar presupuestos por proveedor. No asumir que el contador de la API es el límite del CDN o del servicio privado; manejar sus propios errores. Reutilizar el cliente HTTP: el pool de `reqwest` ya existe en `Catalog`; el problema local es crear clientes por JAR propio y que el host cierre cada conexión. Cambiar solo el cliente no evita ese cierre del servidor. [Reutilización de conexiones](https://docs.rs/reqwest/latest/reqwest/struct.Client.html).

Los enlaces duros no deben ser el atajo predeterminado: comparten contenido y una modificación de una copia cambia las otras. El benchmark se hizo entre archivos del sandbox, nunca enlazando el corpus original. Para caché compartida, preferir copias independientes o clonación real cuando se valide soporte; mantener una ruta portable. [Semántica de enlaces duros](https://learn.microsoft.com/en-us/windows/win32/fileio/hard-links-and-junctions).

### 4. Modrinth: menos peticiones antes que más paralelismo

Conservar los lotes actuales: hashes hasta 512 y proyectos/versiones en lotes locales de 50. Esos tamaños son decisiones del cliente, no límites oficiales demostrados aquí. La API dispone de operaciones múltiples para [identificación por hashes](https://docs.modrinth.com/api/operations/versionsfromhashes/), [actualizaciones por hashes](https://docs.modrinth.com/api/operations/getlatestversionsfromhashes/), [versiones](https://docs.modrinth.com/api/operations/getversions/) y [proyectos](https://docs.modrinth.com/api/operations/getprojects/).

Propuesta:

- Un plan local que solo retira archivos no vuelve a resolver el catálogo si conserva la misma revisión y el grafo verificado. Consulta la revisión del anfitrión y revalida archivos; no inventa pins nuevos.
- Agrupar también los nombres y entornos que todavía se piden proyecto por proyecto.
- Deduplicar solicitudes simultáneas de **la misma clave**, sin mantener el mutex global durante la red. Los aciertos de caché deben seguir disponibles.
- Separar metadatos ligados a versión/hash de campos mutables. Hoy predominan TTL de una hora, negativos de diez minutos y búsquedas de actualización de cinco minutos. Probar mayor retención de identidades exactas; conservar refresco para información mutable y no tratar una entrada como certificación de seguridad.
- Leer `Limit`, `Remaining`, `Reset` y `Retry-After`. Empezar con un presupuesto prudente, por ejemplo hasta cuatro peticiones/s y ráfaga pequeña, reduciéndolo según cabeceras; compartirlo entre tienda, análisis e instalación. El presupuesto por IP también puede consumirlo otro equipo tras el mismo router.
- En `429`, esperar al menos lo indicado por el proveedor; añadir backoff y jitter acotados, sin reintentos masivos. Una pausa no debe invalidar toda la caché ni provocar una consulta por cada mod.

### 5. Host: publicar una vez y servir muchas veces

Detectar cambios por metadatos/eventos, agrupar ráfagas y calcular hashes solo para JAR nuevos o modificados. Mantener un manifiesto materializado y un grafo de dependencias por revisión; no recalcular todo cada diez segundos.

Permitir comprobar una revisión con `ETag`/`If-None-Match`. Un índice sin cambios puede devolver `304` sin reenviar el JSON; el cuerpo actual es pequeño, por lo que la ganancia mayor está en evitar regenerarlo y resolver dependencias. Si está reconstruyéndose, informar ese estado con espera acotada, sin presentar un conjunto vacío como sincronización correcta. [Caché HTTP](https://www.rfc-editor.org/rfc/rfc9111.html).

Servir el contenido por streaming, sin leer/hashar un JAR entero en cada GET si existe un snapshot inmutable verificado. Si se sirve directamente desde una carpeta mutable, revalidar su identidad y cambios y no entregar bytes bajo un hash antiguo. Separar capacidad para metadatos y descargas; hoy superar ocho slots cierra la conexión, no forma una cola justa.

### 6. Cancelación, progreso y reanudación

Mover hashing, ZIP y operaciones de disco costosas fuera de los futuros que coordinan UI y red. Usar trabajo acotado y comprobar cancelación entre bloques. `spawn_blocking` no cancela automáticamente un trabajo ya iniciado; hace falta cooperación explícita. [Tokio](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html).

Mostrar fases reales: inventario, análisis, espera del proveedor, descarga, respaldo y publicación. Calcular progreso con bytes y operaciones pendientes, no con todos los mods del servidor; anunciar cada retirada. Limitar refrescos a unos 10–20/s y emitir siempre las transiciones finales. Los límites de tiempo de red no deben contarse como si fueran límites de todo el procesamiento local.

Para actualizador y archivos grandes: conservar parciales identificados por hash y reanudar con `Range` y un validador fuerte. Validar `206` y `Content-Range`; si llega `200`, reiniciar sin concatenar; tratar `416` y cambios de ETag explícitamente. Verificar el hash completo antes de instalar. El host actual no implementa estas condiciones; no basta con enviar una cabecera. [Reglas de Range e If-Range](https://www.rfc-editor.org/rfc/rfc9110.html#section-13.1.5).

## Prioridades de implementación

| Prioridad | Cambio | Resultado que debe demostrar |
| --- | --- | --- |
| P0 | Compilar para uso real en release; optimizar `sha2` en desarrollo | Eliminar el coste artificial de 51 s por barrido de esta carpeta |
| P0 | Retirada incremental con journal y progreso por archivo | Quitar cinco JAR sin copiar ni hashear todo el modpack otra vez |
| P0 | Separar trabajo de disco/CPU del coordinador asíncrono | Progreso y cancelación siguen respondiendo |
| P1 | Inventario/descriptor por archivo y reutilización del plan | Conjunto intacto sin relecturas completas repetidas |
| P1 | Presupuesto API preventivo, lotes restantes y deduplicación por clave | No volver a producir tormentas de peticiones al agregar otro mod |
| P1 | Streaming y descargas acotadas con caché por contenido | Más rendimiento sin multiplicar RAM por tamaño de JAR |
| P2 | Host incremental, conexiones persistentes y reanudación | Menos CPU/red con varios amigos y conexiones interrumpidas |

La optimización de desarrollo demostrada se puede aplicar con:

```toml
[profile.dev.package.sha2]
opt-level = 3
```

No está añadida al `Cargo.toml` principal en esta investigación. La prueba del override se hizo en el banco aislado.

Objetivos iniciales para validar, **no garantías ya obtenidas**: revisión caliente sin cambios en torno a uno o dos segundos en este equipo; retirar cinco archivos pequeños en menos de dos segundos con caché válida; cero llamadas a Modrinth para volver a aplicar una revisión ya verificada sin cambios de dependencias. Registrar mediana y p95, bytes leídos/escritos, aciertos de caché, peticiones reales, espera por cuota, velocidad de transferencia y memoria máxima.

## Validaciones que deben acompañar los cambios

Cubrir: 0 cambios; 5 retiradas; 1 actualización; 100 descargas; segunda apertura; versiones duplicadas como Apogee; JAR renombrados y modificados con igual tamaño; fechas restauradas; reemplazo del archivo con otro ID; pérdida de eventos; caché corrupta; retirada de unidad externa; `429`; corte de red; disco lleno; permisos; Minecraft abierto; cancelación y cierre abrupto en cada fase del journal.

Medir también que un mod cliente propio no se borra, que una dependencia compartida solo se descarga una vez y que un archivo cambiado después de analizar no se pisa. Mantener un modo de comprobación completa. Verificar Windows y Linux con el servicio real; no confundir el microbenchmark con esa validación.

## Reproducir las mediciones

Desde la raíz del repositorio, las pruebas de hash no usan red ni archivos de las instancias:

```powershell
cargo run --locked --offline --manifest-path scripts/rendimiento_sincronizacion/Cargo.toml
cargo run --locked --offline --manifest-path scripts/rendimiento_sincronizacion/Cargo.toml --config 'profile.dev.package.sha2.opt-level=3'
cargo run --locked --offline --release --manifest-path scripts/rendimiento_sincronizacion/Cargo.toml
cargo run --locked --offline --release --manifest-path scripts/rendimiento_sincronizacion/Cargo.toml --config 'profile.release.opt-level="z"' --config 'profile.release.lto=true' --config 'profile.release.codegen-units=1' --config 'profile.release.panic="abort"'
```

Añadir `-- sha256` mide el algoritmo del actualizador. La medición opcional de disco lee una carpeta plana de mods y necesita espacio temporal para una copia completa; debe ejecutarse cuando no se estén modificando esos archivos:

```powershell
cargo run --locked --offline --release --manifest-path scripts/rendimiento_sincronizacion/Cargo.toml -- io "C:\ruta\a\mods"
```

La prueba crea su sandbox bajo `target/investigacion-rendimiento`, crea enlaces solo entre sus propias copias, comprueba tamaño/fecha del corpus y elimina sus archivos temporales al terminar normalmente. `--offline` requiere que las dependencias de `sha2` estén disponibles en la caché local de Cargo.
