# Movimiento

La lógica de las animaciones está en `crates/ui/src/movimiento.rs` y se prueba sin ventana. El dibujo está en `crates/ui/src/efectos.rs` y en `laminas.rs`. Los tiempos y curvas salen del sistema de diseño (`tokens.json`, componentes Movimiento, Umbral, Picar y Lupa).

## Verbos implementados

| Verbo | Dónde se dispara | Tiempos |
| --- | --- | --- |
| Umbral | Botón Jugar (tarjeta, pie, editor y menú contextual) | 1400 ms: velo 260 ms, líneas 220 a 920 ms, luz 320 a 1220 ms, salida y registro desde 1220 ms (una línea cada 90 ms) |
| Picar | Mantener Supr sobre la instancia seleccionada | Una etapa cada 120 ms, 10 etapas; soltar retira dos etapas cada 40 ms; 48 escombros en 620 a 800 ms |
| Prensa | Bandas y portadas de instancia al aparecer | 420 ms, 8 pasos, barra de 3 px |
| Colocar | Tarjeta restituida al deshacer una eliminación | 120 ms, aro de 2 px |
| Lupa | Láminas a media ventana (informe de fallo, pantallas con lámina lateral) | Lente de 200 px, aumento de 2× a 6× |
| Losa, menús y panel lateral | Ya existían; no cambian | Sin cambios |

El cambio de sección conserva el fundido suave del contenido. No se usa el muro de bloques.

## Reducir movimiento

El ajuste del launcher se comunica con `movimiento::fijar_reducir`. Con la opción activa:

- El Umbral es un fundido a `fulgor` de 120 ms con el registro completo.
- La Prensa es un fundido de 120 ms sin barra.
- Picar conserva las grietas y omite los escombros.
- La Lupa no tiene transición.

## Lupa

Rueda o `+` y `-` cambian el aumento. Las flechas mueven la lente un 2 % de la lámina, o un 10 % con Mayús. Mientras la lente está activa, la lectura en píxeles de la imagen aparece en la esquina inferior de la lámina. Se usa la lámina empaquetada, no recortes de mayor resolución, porque el repositorio aún no incluye el grupo Detalles.

## Probar a mano

Variables del ejemplo `launcher`:

```
ICARO_UMBRAL=1        abre con el Umbral en marcha
ICARO_PICAR=7:1       detiene la eliminación de la instancia 1 en la etapa 7
ICARO_PANTALLA=fallo  muestra una lámina lateral con Lupa
```

```
cargo test -p icaro-ui
cargo run -p icaro-ui --example launcher
```
