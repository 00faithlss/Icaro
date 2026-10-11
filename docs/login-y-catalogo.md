# Inicio de sesión y catálogo

El ejemplo `launcher` conecta la cuenta con Microsoft y consulta el catálogo público de Modrinth. La instalación de Minecraft, los loaders, los mods y los modpacks todavía está pendiente. Las instancias y los archivos que aparecen en las otras pantallas son datos de demostración.

## Iniciar sesión

```sh
cargo run -p icaro-ui --example launcher
```

1. Abre el menú de cuenta y pulsa **Iniciar sesión**.
2. Completa la autorización en el navegador del sistema.
3. Vuelve a Ícaro. La cuenta aparece activa después de comprobar Xbox, el acceso a Minecraft Java y el perfil del juego.

El Client ID público de Ícaro está incluido en el núcleo. El registro de Azure debe admitir cuentas personales de Microsoft y tener `http://localhost` registrado como URI de aplicaciones móviles y de escritorio. El receptor utiliza un puerto temporal; no requiere registrar un puerto fijo. No se usa un secreto de cliente.

**Cancelar** interrumpe el intento. Si el navegador permanece abierto, su respuesta posterior no activa la cuenta. Un intento vence a los diez minutos. Ante un error se muestra la etapa o el código HTTP, sin copiar la respuesta del proveedor ni sus credenciales.

## Sesión recordada

La primera integración admite una cuenta activa y recordada por equipo. La credencial de renovación se guarda mediante Credential Manager en Windows y Secret Service en Linux, separada de los ajustes del launcher. Al volver a abrir Ícaro se renueva la autorización y se comprueba que el perfil corresponda a la cuenta guardada.

Si el almacén del sistema no está disponible o no permite guardar, la cuenta funciona durante esa apertura y la interfaz indica que la sesión no quedó recordada. No se escribe una copia de la credencial en archivos JSON. En Linux se requiere un servicio de secretos disponible en la sesión y `libdbus` para compilar este backend.

**Cerrar sesión** cancela solicitudes, retira la cuenta activa y elimina la credencial guardada. Si la eliminación falla, la interfaz lo indica y permite reintentar. Este cierre es local: la sesión del navegador de Microsoft se administra desde el navegador.

La autorización se renueva antes de vencer, con una sola operación de autenticación en curso. Los intentos antiguos no pueden guardar credenciales ni sustituir la cuenta después de una cancelación o cierre de sesión.

## Modrinth

Desde una instancia, abre **Mods → Descargar mods** o el catálogo de recursos. La búsqueda aplica la versión de Minecraft y, para mods, el loader de esa instancia. La sección **Modpacks** consulta packs del catálogo público.

Las consultas se realizan en segundo plano, con una breve espera al escribir, paginación y descarte de respuestas anteriores. **Ver proyecto** muestra la descripción y las versiones compatibles; **Ver en Modrinth** abre la página oficial. Una respuesta HTTP 429 impone una espera antes de consultar de nuevo.

El catálogo utiliza un cliente HTTP separado del login y no recibe credenciales de Minecraft. La instalación desde la tienda, la resolución de dependencias y la verificación de archivos se implementarán en la siguiente etapa. CurseForge está pendiente de acceso y condiciones de distribución; no aparece como proveedor conectado.

## Modo de demostración

Para revisar las pantallas originales sin autenticación ni catálogo de red, inicia el ejemplo con `ICARO_DEMO=1`. En PowerShell:

```powershell
$env:ICARO_DEMO = '1'
cargo run -p icaro-ui --example launcher
Remove-Item Env:ICARO_DEMO
```

Las cuentas, los códigos de dispositivo, las tiendas y las instalaciones de este modo son simulados. Sus controles no autentican ni instalan contenido real.

## Validación

```sh
cargo test --workspace
cargo test -p icaro-ui --example launcher
cargo test -p icaro-core --test catalogo_real -- --ignored
cargo test -p icaro-core --test vault_native -- --ignored
```

Las pruebas locales cubren PKCE, validación del retorno, cancelación, intercambio de credenciales con respuestas controladas, titularidad, perfil, límites de respuestas, redirecciones y escrituras de sesión obsoletas. La prueba opcional de catálogo consulta la API pública de Modrinth. `vault_native`, disponible en Windows, escribe, lee y elimina una credencial de prueba en un espacio separado de las cuentas del launcher.

El 11 de octubre de 2026 se completó una prueba manual del login con una cuenta real en Windows: el navegador devolvió la autorización y el launcher mostró el perfil de Minecraft Java y la sesión recordada. También se confirmó la recuperación de la cuenta al cerrar y volver a abrir Ícaro. Quedan pendientes las pruebas manuales de renovación y eliminación al cerrar sesión, así como la comprobación del almacén de Linux.

Para comprobar la recuperación, cierra Ícaro y vuelve a abrirlo: debe recuperar el mismo perfil sin solicitar autorización en el navegador. Para comprobar el cierre de sesión, pulsa **Cerrar sesión** y reinicia: debe aparecer como invitado. La renovación automática requiere una prueba con la sesión próxima a vencer.

Las pruebas automatizadas no modifican las cuentas guardadas del launcher: usan un almacén en memoria o una entrada de prueba aislada en Windows.
