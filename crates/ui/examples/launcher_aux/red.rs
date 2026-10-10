// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Estado y comportamiento de ejemplo de Servidores y Descargas: orden por
//! arrastre, servidores que se agregan con Colocar y una cola de descargas
//! que avanza, se pausa, falla, se reintenta y pasa al historial.

use std::time::{Duration, Instant};

use icaro_ui::descargas::{Descarga, EntradaHistorial, EstadoDescarga};
use icaro_ui::servidores::{Arrastre, EstadoServidor, Servidor, ALTO_FILA, SEPARACION};
use icaro_ui::tema::duracion;

/// Cuántos MB descargados hacen que cancelar pida confirmación.
const MB_CONFIRMAR: f32 = 100.0;
/// Tiempo que una descarga completada se queda en la cola antes de pasar al
/// historial.
const SEGUNDOS_COMPLETADA: u64 = 5;

#[derive(Debug, Clone)]
pub enum Mensaje {
    Jugar(usize),
    Tomar(usize),
    Sobre(Option<usize>),
    Soltar,
    CancelarOrden,
    Agregar,
    Pausar(usize),
    Cancelar(usize),
    Seguir(usize),
    Reintentar(usize),
    AlternarHistorial,
    LimpiarHistorial,
    /// Devuelve la lista de servidores a como estaba antes del último cambio.
    Deshacer,
}

/// Descarga simulada: lo que la vista necesita más cómo avanza.
struct Sim {
    nombre: String,
    destino: String,
    total: f32,
    hecho: f32,
    /// MB por segundo cuando está activa.
    vel: f32,
    modo: Modo,
    /// Pide confirmación antes de cancelar.
    confirmando: bool,
    completada_en: Option<Instant>,
}

#[derive(Clone, Copy, PartialEq)]
enum Modo {
    Activa,
    Pausada,
    EnCola,
    Fallo,
    Completada,
}

pub struct Estado {
    pub servidores: Vec<Servidor>,
    pub arrastre: Option<Arrastre>,
    pub sobre: Option<usize>,
    /// Fila recién agregada y cuándo.
    pub colocando: Option<(usize, Instant)>,
    pub historial: Vec<EntradaHistorial>,
    pub historial_abierto: bool,
    cola: Vec<Sim>,
    /// Altura del puntero y fila al empezar el arrastre.
    toma: (f32, usize),
    /// Lista anterior al último cambio, para Deshacer en el toast.
    pub anterior: Option<Vec<Servidor>>,
    ultimo: Instant,
    nuevos: usize,
}

fn sim(nombre: &str, destino: &str, total: f32, hecho: f32, vel: f32, modo: Modo) -> Sim {
    Sim {
        nombre: nombre.into(),
        destino: destino.into(),
        total,
        hecho,
        vel,
        modo,
        confirmando: false,
        completada_en: None,
    }
}

/// Número en formato chileno: coma decimal y punto de miles.
fn numero(x: f32, decimales: usize) -> String {
    let s = format!("{x:.decimales$}");
    let (entero, dec) = s.split_once('.').map_or((s.as_str(), None), |(e, d)| (e, Some(d)));
    let mut miles = String::new();
    for (i, c) in entero.chars().enumerate() {
        if i > 0 && (entero.len() - i) % 3 == 0 {
            miles.push('.');
        }
        miles.push(c);
    }
    match dec {
        Some(d) => format!("{miles},{d}"),
        None => miles,
    }
}

fn tiempo_restante(segundos: f32) -> String {
    let s = segundos.ceil() as u32;
    if s >= 60 {
        format!("{} min {:02} s", s / 60, s % 60)
    } else {
        format!("{s} s")
    }
}

impl Estado {
    pub fn nuevo() -> Self {
        let servidores = vec![
            Servidor {
                nombre: "Amigos".into(),
                direccion: "10.147.17.1:25565".into(),
                motd: Some("Temporada 4: el nether está abierto".into()),
                estado: EstadoServidor::EnLinea { jugadores: 12, maximo: 20, ms: 24 },
                vinculo: Some(("Supervivencia".into(), true)),
                version_distinta: None,
            },
            Servidor {
                nombre: "Creativo de la U".into(),
                direccion: "mc.creativo-u.cl".into(),
                motd: None,
                estado: EstadoServidor::FueraDeLinea("desde las 03:12".into()),
                vinculo: None,
                version_distinta: Some("1.20.1".into()),
            },
            Servidor {
                nombre: "Hermandad".into(),
                direccion: "mc.hermandad.cl:25570".into(),
                motd: None,
                estado: EstadoServidor::EnLinea { jugadores: 3, maximo: 40, ms: 96 },
                vinculo: Some(("Better MC".into(), false)),
                version_distinta: None,
            },
        ];
        Self {
            servidores,
            arrastre: None,
            sobre: None,
            colocando: None,
            historial: vec![
                EntradaHistorial {
                    nombre: "Fabric API 0.112.0".into(),
                    destino: "Supervivencia".into(),
                    tamano: "2,3 MB".into(),
                    fecha: "Hoy 01:12".into(),
                },
                EntradaHistorial {
                    nombre: "Java 21 (Temurin)".into(),
                    destino: "Sistema".into(),
                    tamano: "46,1 MB".into(),
                    fecha: "Ayer 22:40".into(),
                },
            ],
            historial_abierto: true,
            cola: vec![
                sim("Assets de Minecraft 1.21.4", "Skyblock", 687.0, 412.8, 4.2, Modo::Activa),
                sim("Create 0.5.1j", "Supervivencia", 18.2, 4.0, 1.9, Modo::Activa),
                sim("Fabric API 0.112.0", "Supervivencia", 2.3, 0.8, 0.9, Modo::Pausada),
                sim("Iris Shaders 1.8.1", "Supervivencia", 8.4, 0.0, 2.2, Modo::Fallo),
                sim("Java 21 (Temurin)", "Sistema", 46.1, 0.0, 5.0, Modo::EnCola),
            ],
            toma: (0.0, 0),
            anterior: None,
            ultimo: Instant::now(),
            nuevos: 0,
        }
    }

    /// Mueve el hueco del arrastre según la altura del puntero.
    pub fn mover(&mut self, y: f32) {
        if let Some(a) = &mut self.arrastre {
            let (y0, i0) = self.toma;
            let pasos = ((y - y0) / (ALTO_FILA + SEPARACION)).round() as i64;
            let destino = (i0 as i64 + pasos).clamp(0, self.servidores.len() as i64 - 1);
            a.destino = destino as usize;
        }
    }

    /// Aplica un mensaje. Devuelve el aviso que debe mostrar la aplicación.
    pub fn actualizar(&mut self, m: Mensaje, cursor_y: f32, ahora: Instant) -> Option<String> {
        if !matches!(m, Mensaje::Soltar | Mensaje::Deshacer | Mensaje::Sobre(_) | Mensaje::Tomar(_)) {
            self.anterior = None;
        }
        match m {
            Mensaje::Deshacer => {
                if let Some(lista) = self.anterior.take() {
                    self.servidores = lista;
                    self.colocando = None;
                    return Some("Se deshizo el cambio.".into());
                }
            }
            Mensaje::Jugar(i) => {
                return Some(format!("Abriendo {} en la instancia vinculada.", self.servidores[i].nombre));
            }
            Mensaje::Tomar(i) => {
                self.toma = (cursor_y, i);
                self.arrastre = Some(Arrastre { origen: i, destino: i });
            }
            Mensaje::Sobre(i) => self.sobre = i,
            Mensaje::Soltar => {
                if let Some(a) = self.arrastre.take() {
                    if a.origen != a.destino {
                        self.anterior = Some(self.servidores.clone());
                        let s = self.servidores.remove(a.origen);
                        let nombre = s.nombre.clone();
                        self.servidores.insert(a.destino, s);
                        return Some(format!(
                            "{nombre} quedó en la posición {} de {}. El menú de bandeja sigue este orden.",
                            a.destino + 1,
                            self.servidores.len()
                        ));
                    }
                }
            }
            Mensaje::CancelarOrden => self.arrastre = None,
            Mensaje::Agregar => {
                self.anterior = Some(self.servidores.clone());
                self.nuevos += 1;
                self.servidores.push(Servidor {
                    nombre: format!("Servidor nuevo {}", self.nuevos),
                    direccion: format!("play.ejemplo-{}.net", self.nuevos),
                    motd: None,
                    estado: EstadoServidor::EnLinea { jugadores: 0, maximo: 20, ms: 48 },
                    vinculo: None,
                    version_distinta: None,
                });
                self.colocando = Some((self.servidores.len() - 1, ahora));
                return Some("Servidor agregado al final de la lista.".into());
            }
            Mensaje::Pausar(i) => {
                if let Some(d) = self.cola.get_mut(i) {
                    d.modo = if d.modo == Modo::Pausada { Modo::Activa } else { Modo::Pausada };
                }
            }
            Mensaje::Cancelar(i) => {
                let d = self.cola.get_mut(i)?;
                let con_datos = matches!(d.modo, Modo::Activa | Modo::Pausada) && d.hecho > MB_CONFIRMAR;
                if con_datos && !d.confirmando {
                    d.confirmando = true;
                } else {
                    let nombre = self.cola.remove(i).nombre;
                    return Some(format!("Se canceló la descarga de {nombre}."));
                }
            }
            Mensaje::Seguir(i) => {
                if let Some(d) = self.cola.get_mut(i) {
                    d.confirmando = false;
                }
            }
            Mensaje::Reintentar(i) => {
                if let Some(d) = self.cola.get_mut(i) {
                    d.hecho = 0.0;
                    d.modo = Modo::EnCola;
                }
            }
            Mensaje::AlternarHistorial => self.historial_abierto = !self.historial_abierto,
            Mensaje::LimpiarHistorial => {
                self.historial.clear();
                return Some("Historial limpio. Los archivos descargados no se borraron.".into());
            }
        }
        None
    }

    /// Hay algo en movimiento que pide redibujar cada cuadro.
    pub fn animando(&self, descargas_visibles: bool) -> bool {
        self.colocando.is_some()
            || (descargas_visibles
                && self.cola.iter().any(|d| matches!(d.modo, Modo::Activa | Modo::Completada)))
    }

    /// Avanza la simulación: progreso, promoción de la cola y paso al
    /// historial de lo completado.
    pub fn tick(&mut self, ahora: Instant, simultaneas: i32) {
        let dt = ahora.saturating_duration_since(self.ultimo).as_secs_f32().min(0.1);
        self.ultimo = ahora;
        if let Some((_, t0)) = self.colocando {
            if ahora.saturating_duration_since(t0) > Duration::from_millis(duracion::FAST) {
                self.colocando = None;
            }
        }
        for d in &mut self.cola {
            if d.modo == Modo::Activa {
                d.hecho = (d.hecho + d.vel * dt).min(d.total);
                if d.hecho >= d.total {
                    d.modo = Modo::Completada;
                    d.completada_en = Some(ahora);
                }
            }
        }
        // La cola promueve al primero que espera mientras haya cupo.
        let mut activas = self.cola.iter().filter(|d| d.modo == Modo::Activa).count() as i32;
        for d in &mut self.cola {
            if activas >= simultaneas {
                break;
            }
            if d.modo == Modo::EnCola {
                d.modo = Modo::Activa;
                activas += 1;
            }
        }
        // Lo completado pasa al historial tras 5 s.
        let mut i = 0;
        while i < self.cola.len() {
            let lista = self.cola[i].completada_en.is_some_and(|t| {
                ahora.saturating_duration_since(t) >= Duration::from_secs(SEGUNDOS_COMPLETADA)
            });
            if lista {
                let d = self.cola.remove(i);
                self.historial.insert(
                    0,
                    EntradaHistorial {
                        nombre: d.nombre,
                        destino: d.destino,
                        tamano: format!("{} MB", numero(d.total, 1)),
                        fecha: "Ahora".into(),
                    },
                );
            } else {
                i += 1;
            }
        }
    }

    /// Resumen para la banda: "2 activas · 6,1 MB/s".
    pub fn resumen(&self) -> String {
        let activas: Vec<&Sim> = self.cola.iter().filter(|d| d.modo == Modo::Activa).collect();
        if activas.is_empty() {
            return if self.cola.is_empty() {
                "Sin descargas".into()
            } else {
                "Sin descargas activas".into()
            };
        }
        let vel: f32 = activas.iter().map(|d| d.vel).sum();
        let n = activas.len();
        format!(
            "{n} {} · {} MB/s",
            if n == 1 { "activa" } else { "activas" },
            numero(vel, 1)
        )
    }

    pub fn hay_cola(&self) -> bool {
        !self.cola.is_empty()
    }

    /// Cola lista para la vista.
    pub fn descargas(&self) -> Vec<Descarga> {
        let mut espera = 0;
        self.cola
            .iter()
            .map(|d| {
                let avance = d.hecho / d.total;
                let tamanos = format!("{} de {} MB", numero(d.hecho, 1), numero(d.total, 1));
                let (estado, detalle) = match d.modo {
                    _ if d.confirmando => (
                        EstadoDescarga::Confirmando(format!("{} MB", numero(d.hecho, 1))),
                        String::new(),
                    ),
                    Modo::Activa => (
                        EstadoDescarga::Activa(avance),
                        format!(
                            "{tamanos} · {} MB/s · quedan {}",
                            numero(d.vel, 1),
                            tiempo_restante((d.total - d.hecho) / d.vel)
                        ),
                    ),
                    Modo::Pausada => (EstadoDescarga::Pausada(avance), format!("En pausa · {tamanos}")),
                    Modo::EnCola => {
                        espera += 1;
                        (EstadoDescarga::EnCola(espera), format!("{} MB", numero(d.total, 1)))
                    }
                    Modo::Fallo => (
                        EstadoDescarga::Fallo("Modrinth no responde. Reintenta ahora o espera unos segundos.".into()),
                        String::new(),
                    ),
                    Modo::Completada => (EstadoDescarga::Completada, format!("{} MB", numero(d.total, 1))),
                };
                Descarga { nombre: d.nombre.clone(), destino: d.destino.clone(), estado, detalle }
            })
            .collect()
    }
}
