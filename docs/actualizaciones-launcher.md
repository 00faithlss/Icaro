# Actualizaciones de Icaro Launcher

La pantalla sigue `ActualizacionLauncher` del System Design: Saturno en el lateral, versión y tamaño reales, notas agrupadas y acciones **Actualizar ahora**, **Al cerrar Icaro Launcher** y **Más tarde**. Se abre desde **Ajustes → Acerca de → Actualización**; la comprobación por sí sola no interrumpe al usuario. Una nueva versión se anuncia con el banner informativo existente.

La comprobación consulta las releases estables de `00faithlss/Icaro-launcher` en la API oficial de GitHub. No ofrece borradores, versiones previas ni versiones iguales o anteriores. Si el repositorio todavía no tiene releases, informa que no hay una versión publicada más reciente; no usa los ejemplos 1.3 y 1.2.4 del diseño como versiones reales.

Para publicar una actualización portable de Windows x86-64, una release necesita:

- Un tag con la versión completa, como `v0.2.0`, coherente con el `version` del manifiesto y con la versión compilada.
- Un ejecutable portable `.exe` adjunto como asset de esa misma release.
- Un asset `latest.json` con el contrato siguiente. El hash debe ser el SHA-256 real del ejecutable; los valores de este ejemplo no se publican ni se consumen como una actualización.

```json
{
  "version": "0.2.0",
  "notas": [
    {"tipo": "nuevo", "texto": "Descripción de la función para quien juega."},
    {"tipo": "mejora", "texto": "Descripción de una mejora visible."},
    {"tipo": "correccion", "texto": "Descripción del problema resuelto."}
  ],
  "plataformas": {
    "windows-x86_64": {
      "archivo": "icaro-windows-x86_64.exe",
      "sha256": "REEMPLAZAR_POR_EL_SHA256_REAL_DE_64_CARACTERES"
    }
  }
}
```

Las notas deben describir cambios para el usuario. El programa rechaza manifiestos incoherentes, rutas fuera de los assets del repositorio, hashes inválidos, archivos de más de 1 GiB y descargas cuyo tamaño o hash no coincidan. La consulta usa HTTPS; no ejecuta contenido del manifiesto ni mensajes de release.

**Actualizar ahora** descarga a una carpeta aleatoria dentro de `%LOCALAPPDATA%\Icaro\updates`, rechaza enlaces simbólicos y puntos de redirección, comprueba tamaño y SHA-256 y vuelve a comprobar el archivo antes de aplicarlo. Un ayudante oculto espera a que cierre el proceso de Icaro Launcher, bloquea la escritura del archivo mientras lo verifica y copia, reemplaza el ejecutable de forma atómica y lo vuelve a abrir. Conserva el ejecutable anterior con un sufijo `.icaro-anterior-…` para recuperar el launcher si falla el reemplazo. Las instancias y sus archivos no se modifican.

**Al cerrar Icaro Launcher** descarga y verifica en ese momento, y aplica el reemplazo al salir normalmente del launcher, sin abrirlo de nuevo. La programación permanece en la sesión abierta; si el proceso termina de forma abrupta, no se ejecuta el reemplazo. **Más tarde** devuelve a la pantalla anterior sin descargar y cancela una programación previa al cierre. Durante la descarga las acciones quedan deshabilitadas para evitar solicitudes contradictorias. Nunca aplica una actualización mientras Minecraft esté abierto o se esté preparando su instalación o lanzamiento.

Al 11 de octubre de 2026, el repositorio no tiene releases publicadas ni un pipeline de distribución. La pantalla, la comprobación y el contrato están preparados; una actualización completa requiere publicar una release con esos assets. La aplicación automática se implementa para el ejecutable portable de Windows x86-64; otros sistemas pueden comprobar versiones, pero no instalar este artefacto.
