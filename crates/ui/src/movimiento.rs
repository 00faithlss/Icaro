// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Vocabulario de movimiento: pasos duros y peso de piedra, sin rebotes.
//! Las funciones son puras: reciben el progreso de 0 a 1 y devuelven el valor
//! a dibujar, de modo que se prueban sin ventana.

use std::time::{Duration, Instant};

use crate::tema::duracion;

/// Curva de easing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Curva {
    /// Entradas: salida exponencial (`cubic-bezier(0.16, 1, 0.3, 1)`).
    Salida,
    /// Salidas y cierres (`cubic-bezier(0.7, 0, 0.84, 0)`).
    Entrada,
    /// Gravedad (`cubic-bezier(0.55, 0, 1, 0.45)`).
    Caida,
    Lineal,
    /// Pasos duros: el valor avanza a saltos.
    Pasos(u8),
}

impl Curva {
    /// Aplica la curva a un progreso de 0 a 1.
    pub fn aplicar(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Curva::Lineal => t,
            Curva::Pasos(n) => {
                let n = f32::from(n.max(1));
                // `steps(n, end)`: el último salto llega justo al final.
                (t * n).floor() / n
            }
            Curva::Salida => bezier(0.16, 1.0, 0.3, 1.0, t),
            Curva::Entrada => bezier(0.7, 0.0, 0.84, 0.0, t),
            Curva::Caida => bezier(0.55, 0.0, 1.0, 0.45, t),
        }
    }
}

/// Resuelve `cubic-bezier(x1, y1, x2, y2)` por bisección sobre x.
fn bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    let punto = |a: f32, b: f32, s: f32| {
        let u = 1.0 - s;
        3.0 * u * u * s * a + 3.0 * u * s * s * b + s * s * s
    };
    let (mut bajo, mut alto) = (0.0_f32, 1.0_f32);
    for _ in 0..24 {
        let medio = (bajo + alto) / 2.0;
        if punto(x1, x2, medio) < x {
            bajo = medio;
        } else {
            alto = medio;
        }
    }
    punto(y1, y2, (bajo + alto) / 2.0)
}

/// Preferencia de movimiento del usuario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Preferencia {
    pub reducir: bool,
}

impl Preferencia {
    /// Duración efectiva. Con movimiento reducido todo pasa a un fundido corto.
    pub fn duracion(self, ms: u64) -> Duration {
        if self.reducir {
            Duration::from_millis(ms.min(duracion::FAST))
        } else {
            Duration::from_millis(ms)
        }
    }
}

/// Valor que se mueve de un punto a otro con una curva.
#[derive(Debug, Clone, Copy)]
pub struct Transicion {
    desde: f32,
    hasta: f32,
    inicio: Instant,
    duracion: Duration,
    curva: Curva,
}

impl Transicion {
    /// Transición ya terminada en `valor`.
    pub fn fija(valor: f32, ahora: Instant) -> Self {
        Self {
            desde: valor,
            hasta: valor,
            inicio: ahora,
            duracion: Duration::ZERO,
            curva: Curva::Lineal,
        }
    }

    /// Valor en el instante `ahora`.
    pub fn valor(&self, ahora: Instant) -> f32 {
        if self.duracion.is_zero() {
            return self.hasta;
        }
        let t = ahora.saturating_duration_since(self.inicio).as_secs_f32()
            / self.duracion.as_secs_f32();
        self.desde + (self.hasta - self.desde) * self.curva.aplicar(t)
    }

    /// Si todavía falta movimiento por dibujar.
    pub fn animando(&self, ahora: Instant) -> bool {
        ahora.saturating_duration_since(self.inicio) < self.duracion
    }

    /// Redirige la transición desde el valor actual hacia `destino`.
    pub fn ir_a(&mut self, destino: f32, ahora: Instant, ms: u64, curva: Curva, pref: Preferencia) {
        let actual = self.valor(ahora);
        *self = Self {
            desde: actual,
            hasta: destino,
            inicio: ahora,
            duracion: pref.duracion(ms),
            curva,
        };
    }
}

/// Golpe: lo que se presiona se estampa 2 px hacia abajo, sin transición.
pub const GOLPE_PX: f32 = 2.0;

/// Losa: desplazamiento vertical de un diálogo que cae. Devuelve píxeles por
/// encima de su posición final (36 al inicio, 0 al llegar) y un impacto de 3 px
/// hacia abajo en el último paso.
pub fn losa(t: f32) -> f32 {
    const ALTURA: f32 = 36.0;
    const IMPACTO: f32 = 3.0;
    let t = t.clamp(0.0, 1.0);
    if t >= 1.0 {
        return 0.0;
    }
    if t >= 0.9 {
        return -IMPACTO;
    }
    ALTURA * (1.0 - Curva::Caida.aplicar(t / 0.9))
}

/// Colocar: escala de un elemento nuevo, desde 88 % en tres pasos.
pub fn colocar(t: f32) -> f32 {
    0.88 + 0.12 * Curva::Pasos(3).aplicar(t)
}

/// Prensa: avance de una lámina que se imprime de arriba abajo en ocho pasos.
pub fn prensa(t: f32) -> f32 {
    Curva::Pasos(8).aplicar(t)
}

/// Rastrillo y Muro: avance en seis pasos duros.
pub fn rastrillo(t: f32) -> f32 {
    Curva::Pasos(6).aplicar(t)
}

/// Preferencia de movimiento reducido compartida con los widgets que se
/// animan por su cuenta (láminas). La aplicación la fija desde Ajustes.
static REDUCIR: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Fija la preferencia de movimiento reducido para todo el programa.
pub fn fijar_reducir(reducir: bool) {
    REDUCIR.store(reducir, std::sync::atomic::Ordering::Relaxed);
}

/// Preferencia actual.
pub fn preferencia() -> Preferencia {
    Preferencia { reducir: REDUCIR.load(std::sync::atomic::Ordering::Relaxed) }
}

/// Prensa de una lámina a los `ms` de cargarse: `(fracción impresa, opacidad
/// de la imagen, barra visible)`. Con movimiento reducido es un fundido de
/// 120 ms sin barra.
pub fn prensa_lamina(ms: f32, pref: Preferencia) -> (f32, f32, bool) {
    if pref.reducir {
        return (1.0, (ms / duracion::FAST as f32).clamp(0.0, 1.0), false);
    }
    let t = (ms / duracion::PRENSA as f32).clamp(0.0, 1.0);
    (prensa(t), 1.0, t < 1.0)
}

/// Rastrillo: la reja sube en 320 ms y baja en 240 ms.
pub const RASTRILLO_ABRIR_MS: u64 = 320;
pub const RASTRILLO_CERRAR_MS: u64 = 240;

/// Prensa: barra de tinta de 3 px que baja con la lámina. Devuelve la fracción
/// ya impresa (0 a 1) y la opacidad de la barra, que desaparece al terminar.
pub fn prensa_barra(t: f32) -> (f32, f32) {
    let avance = prensa(t);
    (avance, 1.0 - avance)
}

/// Colocar: grosor del aro de 2 px que rodea al elemento mientras se asienta.
pub fn colocar_aro(t: f32) -> f32 {
    if t >= 1.0 {
        0.0
    } else {
        2.0
    }
}

// --- Umbral ---------------------------------------------------------------

/// Lámina El Umbral en su tamaño original (ancho, alto).
pub const UMBRAL_LAMINA: (f32, f32) = (760.0, 507.0);
/// Arco de la lámina en coordenadas originales: x0, y0, x1, y1.
pub const UMBRAL_ARCO: [f32; 4] = [445.0, 98.0, 535.0, 220.0];

/// Rectángulo donde `object-fit: cover` deja la lámina dentro de `caja`.
pub fn cubrir(caja: iced::Size, lamina: (f32, f32)) -> iced::Rectangle {
    let escala = (caja.width / lamina.0).max(caja.height / lamina.1);
    let (w, h) = (lamina.0 * escala, lamina.1 * escala);
    iced::Rectangle {
        x: (caja.width - w) / 2.0,
        y: (caja.height - h) / 2.0,
        width: w,
        height: h,
    }
}

/// Arco del Umbral convertido al sistema de coordenadas de `caja`.
pub fn arco_en(caja: iced::Size) -> iced::Rectangle {
    let lamina = cubrir(caja, UMBRAL_LAMINA);
    let escala = lamina.width / UMBRAL_LAMINA.0;
    let [x0, y0, x1, y1] = UMBRAL_ARCO;
    iced::Rectangle {
        x: lamina.x + x0 * escala,
        y: lamina.y + y0 * escala,
        width: (x1 - x0) * escala,
        height: (y1 - y0) * escala,
    }
}

/// Interpola un rectángulo entre `a` (k = 0) y `b` (k = 1).
pub fn mezclar(a: iced::Rectangle, b: iced::Rectangle, k: f32) -> iced::Rectangle {
    let m = |x: f32, y: f32| x + (y - x) * k;
    iced::Rectangle {
        x: m(a.x, b.x),
        y: m(a.y, b.y),
        width: m(a.width, b.width),
        height: m(a.height, b.height),
    }
}

/// Estado de la secuencia del Umbral en un instante.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaseUmbral {
    /// Opacidad del velo de tinta (hasta 0,62).
    pub velo: f32,
    /// Opacidad del halo del arco.
    pub halo: f32,
    /// Escala del halo respecto de su tamaño base (0,6 a 3,2).
    pub escala_halo: f32,
    /// Avance de las líneas del arco al borde de la ventana (0 a 1).
    pub lineas: f32,
    /// Avance de la luz sólida (0 a 1).
    pub luz: f32,
    /// Opacidad de la luz sólida.
    pub luz_alfa: f32,
    /// Si ya se muestra la pantalla de salida sobre la luz.
    pub salida: bool,
    /// Líneas del registro de arranque visibles (de 0 a 4).
    pub registro: usize,
}

/// Líneas del registro de arranque.
pub const UMBRAL_REGISTRO: usize = 4;

/// Secuencia del Umbral a los `ms` de pulsar Jugar. Con movimiento reducido
/// es un fundido directo a `fulgor` en 120 ms y el registro aparece completo.
pub fn umbral(ms: f32, pref: Preferencia) -> FaseUmbral {
    let tramo = |inicio: f32, dur: f32| ((ms - inicio) / dur).clamp(0.0, 1.0);
    if pref.reducir {
        let salida = ms >= duracion::FAST as f32;
        return FaseUmbral {
            velo: 0.0,
            halo: 0.0,
            escala_halo: 0.6,
            lineas: 0.0,
            luz: 1.0,
            luz_alfa: tramo(0.0, 120.0),
            salida,
            registro: if salida { UMBRAL_REGISTRO } else { 0 },
        };
    }
    // El halo recorre su secuencia completa con curva-entrada: se enciende
    // hasta 30 % del recorrido y después se expande a 3,2 veces.
    let e = Curva::Entrada.aplicar(tramo(0.0, 1000.0));
    let (halo, escala_halo) = if e < 0.3 {
        let k = e / 0.3;
        (k, 0.6 + 0.4 * k)
    } else {
        (1.0, 1.0 + 2.2 * (e - 0.3) / 0.7)
    };
    let salida = ms >= 1220.0;
    let registro = (0..UMBRAL_REGISTRO)
        .filter(|i| ms >= 1220.0 + 90.0 * (*i as f32 + 1.0))
        .count();
    FaseUmbral {
        velo: 0.62 * tramo(0.0, 260.0),
        halo,
        escala_halo,
        lineas: Curva::Entrada.aplicar(tramo(220.0, 700.0)),
        luz: Curva::Entrada.aplicar(tramo(320.0, 900.0)),
        luz_alfa: tramo(0.0, 120.0),
        salida,
        registro,
    }
}

/// Si el arranque falla, la luz se retira en 200 ms: opacidad de 1 a 0.
pub fn umbral_retirada(ms: f32) -> f32 {
    1.0 - (ms / 200.0).clamp(0.0, 1.0)
}

// --- Picar ----------------------------------------------------------------

/// Etapas de grietas al mantener presionado, una cada 120 ms.
pub const PICAR_ETAPAS: u32 = 10;
pub const PICAR_ETAPA_MS: f32 = 120.0;
/// Escombros al romper: rejilla de 8 × 6.
pub const ESCOMBROS_COLS: u32 = 8;
pub const ESCOMBROS_FILAS: u32 = 6;
/// Tiempo hasta que el escombro más lento termina de caer y aparece el aviso.
pub const PICAR_ROTURA_MS: f32 = 720.0;

/// Etapa de grietas tras `ms` manteniendo presionado.
pub fn picar_etapa(ms: f32) -> u32 {
    ((ms.max(0.0) / PICAR_ETAPA_MS) as u32).min(PICAR_ETAPAS)
}

/// Al soltar antes de tiempo las grietas se retiran rápido: dos etapas cada
/// 40 ms.
pub fn picar_soltar(desde: u32, ms: f32) -> u32 {
    desde.saturating_sub(2 * (ms.max(0.0) / 40.0) as u32)
}

fn hash_nombre(nombre: &str) -> u32 {
    let mut h: u32 = 2_166_136_261;
    for b in nombre.bytes() {
        h = (h ^ u32::from(b)).wrapping_mul(16_777_619);
    }
    h
}

/// Píxeles de grieta sobre una rejilla de 16 × 16, en el orden en que
/// aparecen. Se derivan del nombre, de modo que cada instancia se rompe
/// distinto (FNV-1a y xorshift, igual que el sello).
pub fn grietas(semilla: &str) -> Vec<(u8, u8)> {
    let mut r = hash_nombre(semilla);
    if r == 0 {
        r = 7;
    }
    let mut rnd = move || {
        r ^= r << 13;
        r ^= r >> 17;
        r ^= r << 5;
        r as f32 / 4_294_967_296.0
    };
    const DIRS: [(i8, i8); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)];
    let mut pts: Vec<(u8, u8)> = Vec::new();
    let mut vistos = [[false; 16]; 16];
    let mut frentes: Vec<(i8, i8)> = vec![(8, 8)];
    while pts.len() < 120 && !frentes.is_empty() {
        let i = (rnd() * frentes.len() as f32) as usize;
        let f = frentes[i];
        let d = DIRS[(rnd() * 8.0) as usize];
        let n = (f.0 + d.0, f.1 + d.1);
        if !(0..=15).contains(&n.0) || !(0..=15).contains(&n.1) {
            frentes.remove(i);
            continue;
        }
        if vistos[n.0 as usize][n.1 as usize] {
            if rnd() < 0.3 {
                frentes.remove(i);
            }
            continue;
        }
        vistos[n.0 as usize][n.1 as usize] = true;
        pts.push((n.0 as u8, n.1 as u8));
        frentes.push(n);
        if frentes.len() > 5 {
            frentes.remove(0);
        }
    }
    pts
}

/// Cuántos píxeles de grieta se ven en la etapa dada.
pub fn grietas_visibles(total: usize, etapa: u32) -> usize {
    (total as f32 * etapa.min(PICAR_ETAPAS) as f32 / PICAR_ETAPAS as f32).round() as usize
}

/// Escombro de la rejilla de rotura en el instante `ms`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Escombro {
    /// Desplazamiento respecto de su celda de origen, en píxeles.
    pub dx: f32,
    pub dy: f32,
    pub alfa: f32,
    /// Lado del cuadrado, en píxeles.
    pub lado: f32,
    /// Origen de la celda, en píxeles dentro de la tarjeta.
    pub x: f32,
    pub y: f32,
    /// Si usa `text` (verdadero) o `text-muted`.
    pub claro: bool,
}

/// Escombro (`cx`, `cy`) de una tarjeta de `ancho` × `alto` a los `ms` de
/// romperse: salta 14 px y cae con curva-caida. `None` si ya terminó.
pub fn escombro(cx: u32, cy: u32, ancho: f32, alto: f32, ms: f32) -> Option<Escombro> {
    let (w, h) = (ancho / ESCOMBROS_COLS as f32, alto / ESCOMBROS_FILAS as f32);
    let dur = 620.0 + (cx % 4) as f32 * 60.0;
    if ms >= dur {
        return None;
    }
    let lado = (w.min(h) * (0.25 + ((cx * 7 + cy * 3) % 5) as f32 * 0.08)).round().max(6.0);
    let dx = (cx as f32 - ESCOMBROS_COLS as f32 / 2.0) * (6 + (cy % 3) * 4) as f32;
    let caida = 140.0 + (ESCOMBROS_FILAS - cy) as f32 * 30.0 + (cx % 3) as f32 * 20.0;
    let e = Curva::Caida.aplicar(ms / dur);
    // Tres puntos de paso: origen, salto de 14 px a 20 % y caída final.
    let (px, py, alfa) = if e < 0.2 {
        let k = e / 0.2;
        (dx * 0.4 * k, -14.0 * k, 1.0)
    } else {
        let k = (e - 0.2) / 0.8;
        (dx * 0.4 + (dx - dx * 0.4) * k, -14.0 + (caida + 14.0) * k, 1.0 - k)
    };
    Some(Escombro {
        dx: px,
        dy: py,
        alfa,
        lado,
        x: cx as f32 * w + w / 2.0 - lado / 2.0,
        y: cy as f32 * h + h / 2.0 - lado / 2.0,
        claro: (cx + cy).is_multiple_of(3),
    })
}

// --- Lupa -----------------------------------------------------------------

/// Lado de la lente, en píxeles.
pub const LUPA_LADO: f32 = 200.0;
/// Menor tamaño de lámina sobre el que vive la lupa.
pub const LUPA_MIN_LAMINA: f32 = 240.0;

/// Lente que sigue al puntero: posición normalizada y aumento de 2× a 6×.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lupa {
    pub x: f32,
    pub y: f32,
    pub aumento: u8,
}

impl Default for Lupa {
    fn default() -> Self {
        Self { x: 0.5, y: 0.5, aumento: 3 }
    }
}

impl Lupa {
    /// Mueve la lente a la posición del puntero dentro de la lámina.
    pub fn apuntar(&mut self, x: f32, y: f32, caja: iced::Size) {
        self.x = (x / caja.width.max(1.0)).clamp(0.0, 1.0);
        self.y = (y / caja.height.max(1.0)).clamp(0.0, 1.0);
    }

    /// Rueda: hacia arriba amplía, hacia abajo reduce.
    pub fn rueda(&mut self, arriba: bool) {
        self.aumento = if arriba { (self.aumento + 1).min(6) } else { self.aumento.saturating_sub(1).max(2) };
    }

    /// Flechas: 2 % de la lámina, 10 % con Mayús.
    pub fn mover(&mut self, dx: i8, dy: i8, mayus: bool) {
        let paso = if mayus { 0.10 } else { 0.02 };
        self.x = (self.x + paso * f32::from(dx)).clamp(0.0, 1.0);
        self.y = (self.y + paso * f32::from(dy)).clamp(0.0, 1.0);
    }

    /// Rectángulo (relativo a la lente) donde se dibuja la lámina ampliada,
    /// de modo que el punto apuntado quede en el centro.
    pub fn imagen(&self, caja: iced::Size) -> iced::Rectangle {
        let z = f32::from(self.aumento);
        iced::Rectangle {
            x: LUPA_LADO / 2.0 - self.x * caja.width * z,
            y: LUPA_LADO / 2.0 - self.y * caja.height * z,
            width: caja.width * z,
            height: caja.height * z,
        }
    }

    /// Lectura bajo la lámina, en píxeles de la imagen original.
    pub fn lectura(&self, ancho: u32, alto: u32) -> String {
        format!(
            "×{} · x {} · y {} px",
            self.aumento,
            (self.x * ancho as f32).round() as u32,
            (self.y * alto as f32).round() as u32
        )
    }

    /// La lupa no vive sobre láminas menores a 240 px.
    pub fn cabe(caja: iced::Size) -> bool {
        caja.width >= LUPA_MIN_LAMINA && caja.height >= LUPA_MIN_LAMINA
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn las_curvas_parten_en_cero_y_terminan_en_uno() {
        for c in [Curva::Salida, Curva::Entrada, Curva::Caida, Curva::Lineal] {
            assert!(c.aplicar(0.0).abs() < 1e-3);
            assert!((c.aplicar(1.0) - 1.0).abs() < 1e-3);
        }
    }

    #[test]
    fn los_pasos_avanzan_a_saltos() {
        let c = Curva::Pasos(4);
        assert_eq!(c.aplicar(0.0), 0.0);
        assert_eq!(c.aplicar(0.24), 0.0);
        assert_eq!(c.aplicar(0.25), 0.25);
        assert_eq!(c.aplicar(1.0), 1.0);
    }

    #[test]
    fn la_caida_acelera() {
        assert!(Curva::Caida.aplicar(0.5) < 0.5);
        assert!(Curva::Salida.aplicar(0.5) > 0.5);
    }

    #[test]
    fn la_losa_cae_y_para_en_seco() {
        assert!((losa(0.0) - 36.0).abs() < 1e-3);
        assert_eq!(losa(0.95), -3.0);
        assert_eq!(losa(1.0), 0.0);
    }

    #[test]
    fn reducir_movimiento_acorta_la_duracion() {
        let normal = Preferencia { reducir: false };
        let reducida = Preferencia { reducir: true };
        assert_eq!(normal.duracion(280), Duration::from_millis(280));
        assert_eq!(reducida.duracion(280), Duration::from_millis(120));
        assert_eq!(reducida.duracion(60), Duration::from_millis(60));
    }

    #[test]
    fn la_transicion_llega_al_destino() {
        let t0 = Instant::now();
        let mut t = Transicion::fija(0.0, t0);
        t.ir_a(10.0, t0, 200, Curva::Lineal, Preferencia::default());
        assert!(t.animando(t0));
        assert!((t.valor(t0 + Duration::from_millis(100)) - 5.0).abs() < 0.1);
        let fin = t0 + Duration::from_millis(300);
        assert!(!t.animando(fin));
        assert_eq!(t.valor(fin), 10.0);
    }

    #[test]
    fn el_umbral_sigue_la_secuencia_del_sistema() {
        let normal = Preferencia::default();
        let a = umbral(0.0, normal);
        assert!(a.velo == 0.0 && a.lineas < 1e-3 && a.luz < 1e-3 && !a.salida && a.registro == 0);
        assert!((umbral(260.0, normal).velo - 0.62).abs() < 1e-3);
        // Las líneas arrancan a los 220 ms y la luz a los 320 ms.
        assert!(umbral(220.0, normal).lineas < 1e-3);
        assert!(umbral(600.0, normal).lineas > 0.0);
        assert!(umbral(320.0, normal).luz < 1e-3);
        assert!((umbral(920.0, normal).lineas - 1.0).abs() < 1e-3);
        assert!((umbral(1220.0, normal).luz - 1.0).abs() < 1e-3);
        // La salida aparece a 1220 ms y el registro suma una línea cada 90 ms.
        assert!(!umbral(1219.0, normal).salida);
        let s = umbral(1220.0, normal);
        assert!(s.salida && s.registro == 0);
        assert_eq!(umbral(1220.0 + 90.0, normal).registro, 1);
        assert_eq!(umbral(1220.0 + 360.0, normal).registro, 4);
    }

    #[test]
    fn el_umbral_con_movimiento_reducido_es_un_fundido() {
        let reducida = Preferencia { reducir: true };
        let f = umbral(60.0, reducida);
        assert_eq!((f.velo, f.halo, f.lineas, f.luz), (0.0, 0.0, 0.0, 1.0));
        assert!((f.luz_alfa - 0.5).abs() < 1e-3);
        assert!(!f.salida);
        let fin = umbral(120.0, reducida);
        assert!(fin.salida && fin.registro == UMBRAL_REGISTRO);
    }

    #[test]
    fn el_arco_se_convierte_con_object_fit_cover() {
        // Ventana con la proporción de la lámina: sin recorte.
        let arco = arco_en(iced::Size::new(760.0, 507.0));
        assert!((arco.x - 445.0).abs() < 1e-2 && (arco.y - 98.0).abs() < 1e-2);
        assert!((arco.width - 90.0).abs() < 1e-2 && (arco.height - 122.0).abs() < 1e-2);
        // Ventana más ancha: se amplía por el ancho y se recorta arriba y abajo.
        let ancha = arco_en(iced::Size::new(1520.0, 800.0));
        assert!((ancha.width - 180.0).abs() < 1e-2);
        let completa = iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(760.0, 507.0));
        let m = mezclar(arco, completa, 1.0);
        assert_eq!((m.x, m.y, m.width), (0.0, 0.0, 760.0));
    }

    #[test]
    fn picar_avanza_una_etapa_cada_120_ms() {
        assert_eq!(picar_etapa(0.0), 0);
        assert_eq!(picar_etapa(119.0), 0);
        assert_eq!(picar_etapa(120.0), 1);
        assert_eq!(picar_etapa(1200.0), PICAR_ETAPAS);
        assert_eq!(picar_etapa(5000.0), PICAR_ETAPAS);
        assert_eq!(picar_soltar(6, 40.0), 4);
        assert_eq!(picar_soltar(3, 200.0), 0);
    }

    #[test]
    fn las_grietas_dependen_del_nombre_y_caben_en_la_rejilla() {
        let a = grietas("Supervivencia");
        let b = grietas("Supervivencia");
        let c = grietas("Creativo");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(!a.is_empty() && a.len() <= 120);
        assert!(a.iter().all(|&(x, y)| x < 16 && y < 16));
        let mut unicos = a.clone();
        unicos.sort_unstable();
        unicos.dedup();
        assert_eq!(unicos.len(), a.len());
        assert_eq!(grietas_visibles(a.len(), 0), 0);
        assert_eq!(grietas_visibles(a.len(), PICAR_ETAPAS), a.len());
    }

    #[test]
    fn los_escombros_saltan_caen_y_terminan() {
        let e0 = escombro(3, 2, 300.0, 380.0, 0.0).unwrap();
        assert!(e0.dx.abs() < 1e-3 && e0.dy.abs() < 1e-3 && e0.alfa == 1.0);
        // Cada columna dura 620 ms más 60 ms por cada resto de 4.
        assert!(escombro(0, 0, 300.0, 380.0, 620.0).is_none());
        assert!(escombro(3, 0, 300.0, 380.0, 700.0).is_some());
        assert!(escombro(3, 0, 300.0, 380.0, 800.0).is_none());
        assert!(escombro(0, 0, 300.0, 380.0, 500.0).unwrap().dy > 0.0);
    }

    #[test]
    fn la_lupa_limita_aumento_y_posicion() {
        let mut l = Lupa::default();
        for _ in 0..10 {
            l.rueda(true);
        }
        assert_eq!(l.aumento, 6);
        for _ in 0..10 {
            l.rueda(false);
        }
        assert_eq!(l.aumento, 2);
        l.mover(-1, 0, true);
        assert!((l.x - 0.4).abs() < 1e-5);
        l.mover(1, 1, false);
        assert!((l.x - 0.42).abs() < 1e-5 && (l.y - 0.52).abs() < 1e-5);
        l.apuntar(-30.0, 9999.0, iced::Size::new(500.0, 400.0));
        assert_eq!((l.x, l.y), (0.0, 1.0));
        assert!(!Lupa::cabe(iced::Size::new(239.0, 500.0)));
        assert!(Lupa::cabe(iced::Size::new(240.0, 240.0)));
    }

    #[test]
    fn la_lupa_centra_el_punto_apuntado_y_lee_pixeles_originales() {
        let l = Lupa { x: 0.42, y: 0.42, aumento: 3 };
        let caja = iced::Size::new(560.0, 462.0);
        let r = l.imagen(caja);
        assert!((r.x + 0.42 * r.width - LUPA_LADO / 2.0).abs() < 1e-3);
        assert!((r.y + 0.42 * r.height - LUPA_LADO / 2.0).abs() < 1e-3);
        let l = Lupa { x: 0.52, y: 0.41, aumento: 3 };
        assert_eq!(l.lectura(1000, 1000), "×3 · x 520 · y 410 px");
    }

    #[test]
    fn la_prensa_imprime_en_ocho_pasos_y_apaga_la_barra() {
        assert_eq!(prensa_barra(0.0), (0.0, 1.0));
        assert_eq!(prensa_barra(0.12).0, 0.0);
        assert_eq!(prensa_barra(0.13).0, 0.125);
        assert_eq!(prensa_barra(1.0), (1.0, 0.0));
        assert_eq!(colocar_aro(0.5), 2.0);
        assert_eq!(colocar_aro(1.0), 0.0);
    }
}
