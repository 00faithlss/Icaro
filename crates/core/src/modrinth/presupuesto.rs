// SPDX-License-Identifier: GPL-3.0-only
//! Presupuesto preventivo de la API de Modrinth: espacia las peticiones antes
//! de agotar la cuota en lugar de esperar a recibir un 429.
//!
//! Solo se aplica a la API. Las descargas del CDN y del anfitrión tienen sus
//! propios límites y no consumen este presupuesto.
use std::time::{Duration, Instant};

/// 240 peticiones por minuto, por debajo de las 300 documentadas por IP: otro
/// equipo detrás del mismo router comparte esa cuota.
const POR_SEGUNDO: f64 = 4.0;
const RAFAGA: f64 = 8.0;
/// Con esta cantidad restante o menos se espera al reinicio de la ventana.
const RESERVA: u64 = 10;
/// Nunca se retiene una petición más de esto por el presupuesto local.
const MAX_ESPERA: Duration = Duration::from_secs(60);

/// Cuota informada por las cabeceras `X-Ratelimit-*` de una respuesta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Cuota {
    pub restantes: u64,
    /// Segundos hasta que se renueva la ventana; puede ser 0.
    pub reinicio: u64,
}

#[derive(Debug)]
pub(super) struct Presupuesto {
    /// Puede ser negativo: representa peticiones ya reservadas que esperan turno.
    fichas: f64,
    actualizado: Instant,
    pausa_hasta: Option<Instant>,
}

impl Presupuesto {
    pub fn nuevo(ahora: Instant) -> Self {
        Self { fichas: RAFAGA, actualizado: ahora, pausa_hasta: None }
    }

    /// Reserva una petición y devuelve cuánto debe esperar antes de enviarla.
    pub fn reservar(&mut self, ahora: Instant) -> Duration {
        let transcurrido = ahora.saturating_duration_since(self.actualizado).as_secs_f64();
        self.fichas = (self.fichas + transcurrido * POR_SEGUNDO).min(RAFAGA);
        self.actualizado = ahora;
        self.fichas -= 1.0;
        let mut espera = if self.fichas >= 0.0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f64(-self.fichas / POR_SEGUNDO)
        };
        match self.pausa_hasta {
            Some(hasta) if hasta > ahora => espera = espera.max(hasta - ahora),
            Some(_) => self.pausa_hasta = None,
            None => {}
        }
        espera.min(MAX_ESPERA)
    }

    /// Ajusta el ritmo con la cuota real: cerca del límite, pausa hasta el reinicio.
    pub fn observar(&mut self, cuota: Cuota, ahora: Instant) {
        if cuota.restantes > RESERVA {
            return;
        }
        // Un reinicio 0 no se usa como divisor ni como espera nula.
        let reinicio = Duration::from_secs(cuota.reinicio.max(1)).min(MAX_ESPERA);
        let hasta = ahora + reinicio;
        self.pausa_hasta = Some(self.pausa_hasta.map_or(hasta, |actual| actual.max(hasta)));
    }
}

/// Lee `X-Ratelimit-Remaining` y `X-Ratelimit-Reset`; sin ambas no hay cuota.
pub(crate) fn cuota(headers: &reqwest::header::HeaderMap) -> Option<Cuota> {
    let numero = |nombre: &str| headers.get(nombre)?.to_str().ok()?.trim().parse::<u64>().ok();
    Some(Cuota {
        restantes: numero("x-ratelimit-remaining")?,
        reinicio: numero("x-ratelimit-reset")?,
    })
}

/// Hasta un 10 % adicional para que varios clientes no reintenten a la vez.
pub(super) fn con_jitter(espera: Duration) -> Duration {
    let mut azar = [0u8; 2];
    let fraccion = if getrandom::fill(&mut azar).is_ok() {
        f64::from(u16::from_le_bytes(azar)) / f64::from(u16::MAX)
    } else {
        0.0
    };
    espera + espera.mul_f64(0.1 * fraccion)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_rafaga_inicial_no_espera_y_despues_se_espacia() {
        let t = Instant::now();
        let mut p = Presupuesto::nuevo(t);
        for _ in 0..8 {
            assert_eq!(p.reservar(t), Duration::ZERO);
        }
        // La novena espera un cuarto de segundo; la décima, medio.
        assert_eq!(p.reservar(t), Duration::from_millis(250));
        assert_eq!(p.reservar(t), Duration::from_millis(500));
        // Tras un segundo se recuperan cuatro fichas: se salda la deuda y sobran.
        assert_eq!(p.reservar(t + Duration::from_secs(1)), Duration::ZERO);
        assert_eq!(p.reservar(t + Duration::from_secs(10)), Duration::ZERO);
    }

    #[test]
    fn cuota_baja_pausa_hasta_el_reinicio_y_un_reinicio_cero_no_es_nulo() {
        let t = Instant::now();
        let mut p = Presupuesto::nuevo(t);
        p.observar(Cuota { restantes: 200, reinicio: 30 }, t);
        assert_eq!(p.reservar(t), Duration::ZERO);
        p.observar(Cuota { restantes: 5, reinicio: 20 }, t);
        assert_eq!(p.reservar(t), Duration::from_secs(20));
        assert_eq!(p.reservar(t + Duration::from_secs(21)), Duration::ZERO);
        p.observar(Cuota { restantes: 0, reinicio: 0 }, t + Duration::from_secs(30));
        assert_eq!(p.reservar(t + Duration::from_secs(30)), Duration::from_secs(1));
        // Una cabecera exagerada no retiene más del máximo.
        p.observar(Cuota { restantes: 0, reinicio: 86_400 }, t + Duration::from_secs(40));
        assert_eq!(p.reservar(t + Duration::from_secs(40)), MAX_ESPERA);
    }

    #[test]
    fn lee_cabeceras_y_el_jitter_queda_acotado() {
        let mut headers = reqwest::header::HeaderMap::new();
        assert_eq!(cuota(&headers), None);
        headers.insert("x-ratelimit-remaining", "7".parse().unwrap());
        headers.insert("x-ratelimit-reset", " 42 ".parse().unwrap());
        assert_eq!(cuota(&headers), Some(Cuota { restantes: 7, reinicio: 42 }));
        for _ in 0..100 {
            let j = con_jitter(Duration::from_secs(60));
            assert!(j >= Duration::from_secs(60) && j <= Duration::from_secs(66));
        }
    }
}
