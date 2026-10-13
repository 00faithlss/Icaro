# Organización de datos y configuración

Investigación del **12-10-2026**. La estructura objetivo de este documento está pendiente de implementación; no describe una migración ya realizada.

## Qué hace Prism Launcher

Prism usa `instance.cfg` como archivo de ajustes INI: texto con claves y valores. Guarda nombre, icono y ajustes específicos de la instancia, con opciones para heredar o sustituir valores globales de Java, memoria y ejecución. La extensión `.cfg` no aporta por sí sola rendimiento ni compatibilidad. Las versiones de Minecraft y los componentes del loader se describen por separado en `mmc-pack.json`.

Sus archivos reutilizables están en el directorio de datos del launcher: assets, bibliotecas, runtimes Java, caché y metadatos. La carpeta de cada instancia contiene sus datos particulares del juego. Las carpetas de temas, traducciones y catpacks corresponden a funciones de Prism; no necesitamos crearlas en Ícaro sin un uso real.

Prism sí conserva cachés y registros. Al eliminar una instancia, su código llama a la papelera del sistema mediante `QFile::moveToTrash`, con una adaptación al portal de Flatpak. No necesita una carpeta `.papelera` dentro de `instances`. El historial para deshacer guarda el destino devuelto por esa operación; la recuperación depende de la plataforma. Si enviar la instancia a la papelera falla, esa operación devuelve un error, sin recurrir al borrado permanente de la instancia.

Fuentes primarias consultadas:

- [Ubicación y finalidad de las carpetas de Prism](https://prismlauncher.org/wiki/getting-started/data-location/).
- [Lectura y escritura de ajustes INI](https://github.com/PrismLauncher/PrismLauncher/blob/develop/launcher/settings/INISettingsObject.cpp).
- [Ajustes propios y heredados de una instancia](https://github.com/PrismLauncher/PrismLauncher/blob/develop/launcher/BaseInstance.cpp) y [ajustes de Minecraft, Java y memoria](https://github.com/PrismLauncher/PrismLauncher/blob/develop/launcher/minecraft/MinecraftInstance.cpp).
- [Perfil de componentes y mmc-pack.json](https://github.com/PrismLauncher/PrismLauncher/blob/develop/launcher/minecraft/PackProfile.cpp).
- [Eliminación y recuperación de instancias](https://github.com/PrismLauncher/PrismLauncher/blob/develop/launcher/InstanceList.cpp) y [papelera del sistema](https://github.com/PrismLauncher/PrismLauncher/blob/develop/launcher/FileSystem.cpp).

## Estado comprobado en Ícaro

Las carpetas de instancia ya tienen nombres legibles y un ID interno estable. Se escriben `instance.json` y `instance.cfg`; el JSON tiene prioridad y el CFG sirve como recuperación si falta el JSON. El CFG actual tiene un esquema propio, no el de Prism.

El instalador Vanilla guarda assets, bibliotecas y perfiles en `instances/<nombre>/.icaro-minecraft`. Por tanto, dos instancias pueden tener copias de los mismos archivos. Java ya se administra en `Icaro/java`, y existe una caché global. Los registros del juego pertenecen a `minecraft/logs`.

Eliminar una instancia mueve su carpeta completa a `instances/.papelera`. Hay también mecanismos separados para archivos, mods y capturas: `.papelera`, `.icaro-mods-trash` y respaldos de operaciones. No existe una política unificada de caducidad y limpieza. Son datos de recuperación y archivos de trabajo; no se deben borrar indiscriminadamente como si todos fueran descargas desechables.

La organización de archivos compartidos debe cambiar en el instalador del launcher y en la construcción del lanzamiento. No aparecerá automáticamente al integrar los instaladores oficiales de loaders, y afecta también a Vanilla.

## Estructura objetivo

En Windows, conservar `%APPDATA%\Icaro` como raíz de datos. La instalación del ejecutable y los datos del usuario deben poder actualizarse de forma independiente.

```text
Icaro/
  assets/                   # Objetos e índices reutilizables de Minecraft
  libraries/                # Bibliotecas verificadas de Minecraft y loaders
  java/                     # Runtimes administrados, ya existente
  meta/                     # Manifiestos y perfiles por versión
  cache/                    # Respuestas y descargas temporales recuperables
  logs/                     # Registros del launcher con rotación
  estado.json               # Preferencias globales; revisar su separación
  instances/
    boys/
      instance.cfg          # Configuración propia de la instancia
      instance.json         # Compatibilidad durante la transición
      icon.png
      icono.json
      minecraft/
        mods/
        resourcepacks/
        shaderpacks/
        saves/
        screenshots/
        logs/               # Registros de Minecraft de esta instancia
        options.txt
```

El perfil local de instalación, sus nativos extraídos y el registro de procedencia de recursos también necesitan una ubicación explícita. No mezclar estos datos con la configuración editable ni considerar los registros de procedencia archivos temporales. La ubicación final del perfil debe definirse con el instalador compartido, sin prometer compatibilidad Prism por usar nombres parecidos.

## Trabajo pendiente y criterios

- [ ] Unificar la resolución de rutas para instalación, lanzamiento, verificación, cambio de versión, importación y exportación.
- [ ] Compartir assets y bibliotecas por ruta e identidad verificadas. Coordinar instalaciones simultáneas y publicar archivos completos mediante escrituras atómicas.
- [ ] Migrar las instalaciones antiguas: verificar los archivos antes de reutilizarlos, actualizar el perfil y comprobar el lanzamiento antes de retirar las copias antiguas. Mantener recuperación ante error o cancelación.
- [ ] Definir una sola fuente de configuración por instancia, con versión de esquema, herencia global y ajustes de Java/memoria portables. Conservar campos desconocidos durante la migración de JSON/CFG; evitar dos fuentes editables que puedan contradecirse.
- [ ] Usar la papelera del sistema para las eliminaciones del usuario y conectar recuperación donde la plataforma lo permita. Si falla, mostrar el error; no convertirla silenciosamente en borrado permanente.
- [ ] Ofrecer recuperación o traslado explícito de los datos de las papeleras internas antiguas. No vaciarlas al actualizar ni cargar el launcher.
- [ ] Separar respaldos de transacciones de las eliminaciones del usuario. Limpiar temporales al finalizar una operación y recuperar operaciones interrumpidas antes de retirar sus respaldos.
- [ ] Incorporar tamaños reales, limpieza explícita de caché y rotación de registros en Almacenamiento. Excluir mundos, mods, iconos y configuración de la limpieza automática.
- [ ] Verificar dos instancias de la misma versión reutilizando assets/bibliotecas, cambios de nombre y versión, cancelación, descarga corrupta, instancias antiguas y exportaciones sin rutas absolutas del equipo.

La integración de loaders debe apoyarse en estas rutas compartidas. La migración y las eliminaciones necesitan comprobaciones tanto en Windows como en Linux antes de considerarlas completas.
