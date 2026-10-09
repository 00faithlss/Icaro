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
}
