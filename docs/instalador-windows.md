# Instalador de Windows y distribución

El instalador forma parte del roadmap y todavía no está implementado. La descarga portable y el contrato de actualización existentes siguen disponibles como recorridos independientes.

## Alcance

Instalar Ícaro para el usuario actual, con icono, accesos directos, versión identificable y entrada de desinstalación. La ubicación del programa debe quedar separada de los datos en AppData; reinstalar, actualizar o desinstalar conserva mundos, mods, capturas, ajustes y cuentas.

La desinstalación retira únicamente archivos y accesos que pertenezcan a esa instalación. Borrar los datos personales requiere una acción independiente y explícita; no debe incorporarse como efecto silencioso.

## Empaquetado

Generar el ejecutable portable y el instalador desde la misma versión y revisión del código. Comprobar los requisitos de Windows y las dependencias de ejecución, incluir los avisos de licencia necesarios y calcular hashes reales de los artefactos.

La herramienta de instalación se elegirá al implementar el paquete. Cualquier pantalla personalizada debe utilizar el diseño entregado o aprobarse antes de incorporarla; el asistente estándar del sistema de instalación puede mantener sus controles nativos.

No se declara firma digital disponible sin certificado configurado. Publicar una release es una operación separada de generar y probar el instalador.

## Validación

Probar en un equipo limpio: primera instalación, inicio del launcher, accesos directos, instalación sobre una versión anterior, reinstalación y desinstalación. Comprobar rutas con espacios y nombres de usuario con acentos; verificar que los datos existentes y la sesión recordada permanezcan intactos.

La aplicación instalada debe tener un recorrido de actualización comprobado. El actualizador actual está documentado para el ejecutable portable; no se presentará automáticamente como validado para cualquier ubicación o método de instalación.

El roadmap cuenta tres tareas: creación del instalador, automatización del empaquetado y validación completa. Consulta el [contrato de releases y actualización](actualizaciones-launcher.md).
