# Inicio de sesión y catálogo

El ejemplo `launcher` conecta la cuenta con Microsoft y consulta el catálogo público de Modrinth. La instalación de Minecraft, los loaders, los mods y los modpacks todavía está pendiente. Las instancias y los archivos que aparecen en las otras pantallas son datos de demostración.

## Iniciar sesión

```sh
cargo run -p icaro-ui --example launcher
```

1. Abre el menú de cuenta y pulsa **Iniciar sesión**.
2. Ícaro solicita un código a Microsoft y lo muestra en las celdas de la página de inicio de sesión. Pulsa **Copiar código**, luego **Abrir Microsoft** e introdúcelo en la página de vinculación del navegador. El navegador se abre únicamente al pulsar ese botón.
3. Autoriza la vinculación con Ícaro desde Microsoft.
4. Vuelve a Ícaro. La cuenta aparece activa después de comprobar Xbox, el acceso a Minecraft Java y el perfil del juego; no requiere pulsar una confirmación en el launcher.

El código es emitido por Microsoft para cada intento. La pantalla muestra el tiempo restante según el vencimiento recibido del proveedor. Ícaro consulta la confirmación respetando el intervalo indicado, aumenta la espera si Microsoft lo solicita y permite pedir un código nuevo cuando termina o vence el intento. El código interno de autorización se mantiene en el núcleo; no se guarda en los ajustes ni se muestra en registros.

El Client ID público de Ícaro está incluido en el núcleo. El registro de Azure debe admitir cuentas personales de Microsoft y permitir flujos de cliente público para la vinculación por código. Este ajuste se encuentra en **Autenticación → Configuración → Permitir flujos de cliente público**. No se usa un secreto de cliente. Microsoft documenta este [flujo por código de dispositivo](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code) y su [configuración de cliente público](https://learn.microsoft.com/en-us/entra/identity-platform/msal-client-applications).

La pantalla de vinculación utiliza un único recorrido por código. El núcleo conserva la implementación de PKCE y retorno local, probada anteriormente, pero la interfaz ya no ofrece cambiar a ese método durante la vinculación.

**Cancelar** interrumpe el intento. Si el navegador permanece abierto, su autorización posterior no activa la cuenta en Ícaro. Ante un error se muestra la etapa o un mensaje específico, sin copiar la descripción del proveedor ni sus credenciales.

## Sesión recordada

La primera integración admite una cuenta activa y recordada por equipo. La credencial de renovación se guarda mediante Credential Manager en Windows y Secret Service en Linux, separada de los ajustes del launcher. Al volver a abrir Ícaro se renueva la autorización y se comprueba que el perfil corresponda a la cuenta guardada.

Si el almacén del sistema no está disponible o no permite guardar, la cuenta funciona durante esa apertura y la interfaz indica que la sesión no quedó recordada. No se escribe una copia de la credencial en archivos JSON. En Linux se requiere un servicio de secretos disponible en la sesión y `libdbus` para compilar este backend.

**Cerrar sesión** cancela solicitudes, retira la cuenta activa y elimina la credencial guardada. Si la eliminación falla, la interfaz lo indica y permite reintentar. Este cierre es local: la sesión del navegador de Microsoft se administra desde el navegador.

La barra inferior de **Jugar** aparece en la lista de instancias cuando hay una cuenta activa y una instancia seleccionada. Muestra el nombre y la versión de esa instancia. Al abrir el launcher no se selecciona una instancia por defecto; las pantallas de vinculación, cuenta y otros pasos tampoco muestran la barra.

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
cargo test -p icaro-core --test device_real -- --ignored
```

Las pruebas locales cubren la solicitud de códigos, la espera de confirmación, la ampliación del intervalo, el vencimiento, PKCE, validación del retorno, cancelación, intercambio de credenciales con respuestas controladas, titularidad, perfil, límites de respuestas, redirecciones y escrituras de sesión obsoletas. La prueba opcional de catálogo consulta la API pública de Modrinth. `vault_native`, disponible en Windows, escribe, lee y elimina una credencial de prueba en un espacio separado de las cuentas del launcher. `device_real` comprueba que Microsoft emita un código para Ícaro sin imprimirlo, abrir el navegador ni autorizar una cuenta.

El 11 de octubre de 2026 se completó una prueba manual del login con retorno local y una cuenta real en Windows: el navegador devolvió la autorización y el launcher mostró el perfil de Minecraft Java y la sesión recordada. También se confirmó la recuperación de la cuenta al cerrar y volver a abrir Ícaro. La solicitud de código real a Microsoft pasó la prueba automatizada; falta completar la vinculación con una cuenta desde la interfaz. Quedan pendientes las pruebas manuales de renovación y eliminación al cerrar sesión, así como la comprobación del almacén de Linux.

Para comprobar la recuperación, cierra Ícaro y vuelve a abrirlo: debe recuperar el mismo perfil sin solicitar autorización en el navegador. Para comprobar el cierre de sesión, pulsa **Cerrar sesión** y reinicia: debe aparecer como invitado. La renovación automática requiere una prueba con la sesión próxima a vencer.

Las pruebas automatizadas no modifican las cuentas guardadas del launcher: usan un almacén en memoria o una entrada de prueba aislada en Windows.
