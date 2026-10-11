# Cuenta, estudio de skin y estados de sesión

Referencia de las pantallas de cuenta y skin del modo de demostración (`ICARO_DEMO=1`). El login conectado se describe en [inicio de sesión y catálogo](login-y-catalogo.md).

## Pantallas

| Pantalla | Qué muestra |
| --- | --- |
| Cuenta | Cuerpo de la skin de frente y de espalda, nombre de usuario con la fecha del próximo cambio, historial de skins y cierre de sesión. |
| Estudio de skin | Escena con dos cuerpos sobre un suelo de líneas, modelo (clásico o delgado), capas visibles y acciones para cargar una `.png` y aplicarla. |
| Menú de cuentas | Se abre desde el bloque de cuenta de la barra lateral. Lista las cuentas, el modo invitado y las acciones de cuenta. |

## Estados de sesión

| Estado | Dónde se ve | Acción |
| --- | --- | --- |
| Invitado | Bloque de cuenta con cabeza rayada, pantalla de Cuenta | Iniciar sesión |
| Cuenta vencida | Insignia en el bloque de cuenta, aviso en Cuenta y al pulsar Jugar | Volver a entrar, que abre el código directamente |
| Código vencido | Celdas rayadas en el flujo de código | Pedir otro código |
| Sin Minecraft | Pantalla de Cuenta con la explicación | Comprar Minecraft o jugar como invitado |

## Comportamiento del estudio

- Girar con las flechas del teclado, con los botones de la escena o arrastrando horizontalmente. Cada paso es de 45° y no hay interpolación.
- Soltar una `.png` sobre la escena la carga. Se valida que sea PNG de 64 × 64 píxeles leyendo solo la cabecera del archivo.
- Aplicar muestra un indicador de carga en el botón y la escena no cambia hasta terminar. La subida es simulada.
- Con la opción de reducir movimiento activa, el menú de cuentas aparece sin caída.

## Abrir cada estado en la maqueta

```
cargo run -p icaro-ui --example launcher
```

Variables de entorno opcionales:

| Variable | Valores |
| --- | --- |
| `ICARO_PANTALLA` | `cuenta`, `estudio`, `codigo` |
| `ICARO_SESION` | `invitado`, `vencida`, `sinjuego` |
| `ICARO_CODIGO` | `vencido` |
| `ICARO_MENU_CUENTA` | cualquier valor abre el menú de cuentas |
| `ICARO_BANNER_JUGAR` | cualquier valor muestra el aviso de cuenta vencida sobre Jugar |
| `ICARO_GIRO`, `ICARO_MODELO`, `ICARO_CAPAS`, `ICARO_SUBIENDO` | estado inicial del estudio |
| `ICARO_PIEDRA` | cualquier valor usa el tema claro |

## Pendiente

El login conectado consulta el perfil de Minecraft; todavía requiere prueba con una cuenta real. La textura de la skin, el cambio de nombre y la subida siguen pendientes de integración. Las etapas y comprobaciones se detallan en el [plan de integración](integracion-real.md).
