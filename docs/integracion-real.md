# Integración del launcher

Plan de implementación de la autenticación, la instalación del juego y las tiendas de mods. Las funciones pendientes y sus condiciones de aceptación se detallan a continuación.

## Estado actual

| Parte | Implementación actual |
| --- | --- |
| Núcleo | `crates/core/src/manifest.rs` consulta el manifiesto de Mojang y verifica el SHA-1 del JSON de una versión. |
| Inicio de sesión | `launcher_aux/pantallas.rs` muestra un código fijo y permite simular la confirmación. `sesion.rs` usa cuentas de ejemplo. |
| Tienda | `launcher_aux/mods.rs` contiene un catálogo estático. Las descargas de `red.rs` están simuladas. |
| Persistencia | `launcher_aux/estado.rs` guarda grupos y ajustes en `icaro-demo/estado.json`, dentro del directorio temporal. No es un almacén de credenciales. |
| Juego | Faltan el instalador de Minecraft, los loaders y el proceso real de lanzamiento. |

La aplicación tiene un Client ID público aprobado y la URI de redirección `http://localhost` registrada en Azure para aplicaciones móviles y de escritorio. Falta verificar los tipos de cuenta admitidos e integrar y probar el intercambio real.

## Inicio de sesión

La interfaz actual muestra una demostración del flujo de código de dispositivo. Su integración requiere obtener un código real, respetar el intervalo y la caducidad recibidos, permitir cancelar y sustituir la confirmación simulada. Este flujo no requiere URI de redirección y necesita configuración de cliente público. Véase [registro de aplicaciones de escritorio](https://learn.microsoft.com/en-us/entra/identity-platform/scenario-desktop-overview) y [protocolo de dispositivo](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code).

El flujo previsto para producción utiliza el navegador del sistema y el retorno a `http://localhost` mediante autorización con PKCE S256, comprobación de `state` y un receptor local temporal. Este receptor debe escuchar solo en loopback, aceptar la respuesta correspondiente al intento iniciado y cerrarse al completar, cancelar o vencer. El navegador del sistema recibe la contraseña; Ícaro recibe la autorización. Una aplicación de escritorio no debe incorporar un secreto de cliente. Véase [flujo de autorización de Microsoft](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow) y [reglas de las URI locales](https://learn.microsoft.com/en-us/entra/identity-platform/reply-url).

La autenticación de Microsoft debe continuar hasta obtener acceso a Minecraft y comprobar el perfil y la titularidad del juego. Una sesión de Microsoft por sí sola no debe marcar la cuenta como lista para jugar. Los intercambios específicos de Xbox y Minecraft necesitan validación con la aplicación aprobada antes de considerar terminado el login.

## Seguridad que debe implementar el núcleo

- Almacenar la credencial de renovación mediante el almacén del sistema, por cuenta: Credential Manager en Windows y Secret Service en Linux. Si no está disponible, ofrecer una sesión sin persistencia. Nunca recurrir a JSON sin protección. [API de credenciales de Windows](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew).
- Mantener secretos fuera de mensajes de interfaz, serialización de ajustes, `Debug`, errores, URLs, registros y reportes de fallos. Reducir copias y limpiar memoria cuando sea posible.
- Usar clientes HTTP separados para identidad, catálogo y archivos. Restringir destinos de credenciales; bloquear redirecciones de autenticación; conservar la validación TLS y aplicar límites de tiempo y tamaño.
- Cancelar solicitudes al salir o cerrar sesión, descartar respuestas de sesiones anteriores y borrar la credencial persistida al cerrar sesión. Renovar sin duplicar peticiones concurrentes.
- Nunca registrar la línea de lanzamiento con el token de Minecraft. El juego necesita recibir su credencial: hay que revisar por separado la exposición local de argumentos y los registros del proceso.

Estas medidas reducen exposición accidental. El almacén del sistema no garantiza aislamiento frente a un programa malicioso ejecutado con la misma identidad del usuario. Los hashes verifican integridad; no demuestran que un mod sea inocuo.

## Modrinth

Integrar primero el catálogo público con la API documentada. No necesita el token de Minecraft. El cliente debe identificar Ícaro mediante `User-Agent`, respetar límites y tratar errores sin bloquear la interfaz. [Descripción de la API](https://docs.modrinth.com/api/).

El `User-Agent` debe incluir el nombre, la versión de Ícaro y un enlace al proyecto, por ejemplo `Icaro/0.1.0 (+https://github.com/00faithlss/Icaro)`.

La búsqueda usa texto, tipo de proyecto, versión de Minecraft y loader. Se pagina y se descartan respuestas antiguas al cambiar filtros. La selección final de versión vuelve a comprobar compatibilidad, porque encontrar un proyecto no garantiza que cualquier archivo sirva para la instancia. [Búsqueda](https://docs.modrinth.com/api/operations/searchprojects/) y [versiones](https://docs.modrinth.com/api/operations/getprojectversions/).

Antes de descargar, resolver dependencias obligatorias y conflictos; después verificar tamaño y SHA-512, escribir en un archivo temporal y publicar el resultado en la instancia. Validar nombres, rutas y archivos comprimidos para impedir escrituras fuera del destino. Guardar los IDs estables de proyecto y versión junto al archivo instalado para actualizaciones y reconocimiento posteriores.

Los modpacks crean instancias desde sus manifiestos; los mods se instalan dentro de cada instancia.

## CurseForge

La integración es técnicamente posible, pero necesita una clave de desarrollador y respetar la distribución autorizada por cada proyecto. [Solicitud de acceso](https://support.curseforge.com/support/solutions/articles/9000208346-about-the-curseforge-api-and-how-to-apply-for-a-key) y [API](https://docs.curseforge.com/rest-api/).

El [anuncio del 10 de junio de 2026](https://blog.curseforge.com/introducing-api-key-authentication-for-curseforge-file-downloads/) fija el 16 de julio como fecha de exigencia de clave en el CDN. Propone transmitirla mediante cabecera para reducir su aparición en registros e historial.

No publicar la clave en Git ni asumir que incluirla en el ejecutable la mantiene privada. Antes de elegir el mecanismo de distribución o un servicio intermediario, confirmar con CurseForge las condiciones aplicables a Ícaro: sus [términos públicos](https://support.curseforge.com/support/solutions/articles/9000207405-curseforge-3rd-party-api-terms-and-conditions) contienen restricciones de divulgación, caché y acceso mediante intermediarios. Un proxy no es una solución automáticamente autorizada. Mantener este proveedor opcional y explicar cuando un archivo requiere descarga manual.

## Orden y condiciones de aceptación

| Entrega | Comprobación necesaria |
| --- | --- |
| Autenticación y credenciales | Cuenta real, cancelación, rechazo, caducidad, renovación, reinicio y cierre de sesión; un secreto de prueba no aparece en archivos ni registros. |
| Instancia Vanilla | Instalar y lanzar con una cuenta válida; verificar archivos, Java, argumentos y aislamiento entre instancias. |
| Loader y Modrinth | Instalar una versión compatible con dependencias reales; probar hash incorrecto, red interrumpida, falta de espacio y HTTP 429. |
| Modpacks | Importar manifiestos y rechazar rutas peligrosas, archivos desproporcionados y conflictos sin dejar una instalación parcial activa. |
| CurseForge | Acceso y distribución acordados, protección de clave y tratamiento claro de descargas restringidas. |

La implementación debe situar los servicios en `icaro-core` y usar iced para presentar su estado y ejecutar trabajo asíncrono. Los ejemplos de diseño deben mantenerse separados del ejecutable de producción para impedir que una cuenta o descarga simulada se confunda con una real.
