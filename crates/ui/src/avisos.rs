// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Un único aviso transitorio, reemplazado por el siguiente mensaje.
use std::time::{Duration, Instant};

pub const DURACION: Duration = Duration::from_secs(3);
pub const SALIDA: Duration = Duration::from_millis(180);

pub struct Aviso<M> {
    pub texto: String,
    pub accion: Option<M>,
    pub desde: Instant,
    retirada: Option<Instant>,
    duracion: Duration,
}

impl<M> Aviso<M> {
    pub fn nuevo(texto: String, accion: Option<M>, desde: Instant) -> Self {
        Self {
            texto,
            accion,
            desde,
            retirada: None,
            duracion: DURACION,
        }
    }

    pub fn con_duracion(mut self, duracion: Duration) -> Self {
        self.duracion = duracion;
        self
    }

    pub fn vencido(&self, ahora: Instant) -> bool {
        ahora.saturating_duration_since(self.desde) >= self.duracion
            || self
                .retirada
                .is_some_and(|inicio| ahora.saturating_duration_since(inicio) >= SALIDA)
    }

    pub fn cerrar(&mut self, ahora: Instant) {
        self.retirada.get_or_insert(ahora);
    }

    /// El fundido ocupa los últimos 180 ms de los tres segundos del aviso.
    pub fn opacidad(&self, ahora: Instant, reducir: bool) -> f32 {
        if reducir {
            return 1.0;
        }
        let inicio = self.retirada.unwrap_or(self.desde + self.duracion - SALIDA);
        let t = (ahora.saturating_duration_since(inicio).as_secs_f32() / SALIDA.as_secs_f32())
            .clamp(0.0, 1.0);
        1.0 - t * t * (3.0 - 2.0 * t)
    }

    pub fn saliendo(&self, ahora: Instant) -> bool {
        self.retirada.is_some()
            || ahora.saturating_duration_since(self.desde) >= self.duracion - SALIDA
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_aviso_caduca_a_los_tres_segundos_y_el_siguiente_reinicia_el_plazo() {
        let inicio = Instant::now();
        let mut aviso = Aviso::<()>::nuevo("Primero".into(), None, inicio);
        assert!(!aviso.vencido(inicio + Duration::from_millis(2999)));
        assert!(aviso.vencido(inicio + DURACION));
        aviso = Aviso::nuevo("Segundo".into(), None, inicio + Duration::from_secs(2));
        assert!(!aviso.vencido(inicio + Duration::from_secs(3)));
        assert!(aviso.vencido(inicio + Duration::from_secs(5)));
        assert_eq!(aviso.texto, "Segundo");
    }

    #[test]
    fn la_salida_es_gradual_y_cerrar_no_reinicia_el_fundido() {
        let inicio = Instant::now();
        let mut aviso = Aviso::<()>::nuevo("Aviso".into(), None, inicio);
        assert_eq!(aviso.opacidad(inicio + DURACION - SALIDA, false), 1.0);
        assert!((aviso.opacidad(inicio + DURACION - SALIDA / 2, false) - 0.5).abs() < 0.001);
        assert_eq!(aviso.opacidad(inicio + DURACION, false), 0.0);
        assert_eq!(aviso.opacidad(inicio + DURACION - SALIDA / 2, true), 1.0);
        aviso.cerrar(inicio);
        aviso.cerrar(inicio + SALIDA / 2);
        assert!((aviso.opacidad(inicio + SALIDA / 2, false) - 0.5).abs() < 0.001);
        assert!(aviso.vencido(inicio + SALIDA));
        let siguiente = Aviso::<()>::nuevo("Siguiente".into(), None, inicio + SALIDA);
        assert_eq!(siguiente.opacidad(inicio + SALIDA, false), 1.0);
    }
    #[test]
    fn deshacer_tiene_diez_segundos_y_conserva_el_fundido() {
        let inicio = Instant::now();
        let a = Aviso::nuevo("Retirados".into(), Some(()), inicio)
            .con_duracion(Duration::from_secs(10));
        assert!(!a.vencido(inicio + Duration::from_secs(9)));
        assert_eq!(a.opacidad(inicio + Duration::from_secs(9), false), 1.);
        assert!(a.opacidad(inicio + Duration::from_secs(10) - SALIDA / 2, false) < 1.);
        assert!(a.vencido(inicio + Duration::from_secs(10)));
    }
}
