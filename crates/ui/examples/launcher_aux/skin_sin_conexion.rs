// SPDX-License-Identifier: GPL-3.0-only
//! SkinSinConexion: PNG local, vista previa y modelo; sin capas de Microsoft.
use crate::{cuenta_sin_conexion as cuentas, App, Mensaje};
use icaro_core::{
    cuentas_locales::Cuentas,
    skin_local::{self, Imagen},
};
use icaro_ui::{
    componentes as c, cuenta, estilo, fuentes,
    iconos::{icono, Icono, Tam},
    tema::texto,
};
use iced::{
    widget::{column, container, row, text, Space},
    Alignment, Element, Length, Task,
};
use std::path::PathBuf;

pub struct Estado {
    pub nombre: String,
    generacion: u64,
    imagen: Option<Imagen>,
    archivo: Option<String>,
    delgada: bool,
    exterior: bool,
    error: Option<String>,
    guardada: bool,
    pub ocupada: bool,
    cargando: bool,
    encima: bool,
}
#[derive(Debug, Clone)]
pub enum Accion {
    Abrir(String),
    Cerrar,
    Elegir,
    Archivo(u64, Option<PathBuf>),
    Cargada(u64, Result<Option<(Imagen, bool, String)>, String>),
    Soltar(PathBuf),
    Encima(bool),
    Modelo(bool),
    Exterior(bool),
    Guardar,
    Quitar,
    Guardada(u64, bool, Result<Cuentas, String>),
}
pub fn m(a: Accion) -> Mensaje {
    cuentas::m(cuentas::Accion::Skin(a))
}

fn leer(app: &mut App, path: PathBuf) -> Task<Mensaje> {
    let Some(s) = app.servicios.local.skin.as_mut() else {
        return Task::none();
    };
    if s.ocupada {
        return Task::none();
    }
    s.cargando = true;
    s.encima = false;
    s.error = None;
    app.servicios.local.revision_skin += 1;
    s.generacion = app.servicios.local.revision_skin;
    let g = s.generacion;
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let nombre = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("skin.png")
                    .to_owned();
                let imagen = skin_local::cargar(&path)?;
                let modelo = imagen.sugerida_delgada;
                Ok(Some((imagen, modelo, nombre)))
            })
            .await
            .map_err(|_| "No se pudo abrir la imagen.".to_string())
            .and_then(|r| r)
        },
        move |r| m(Accion::Cargada(g, r)),
    )
}

pub fn actualizar(app: &mut App, a: Accion) -> Task<Mensaje> {
    if let Accion::Abrir(nombre) = a {
        if app.servicios.auth_busy
            || app.servicios.restoring
            || !app.servicios.local.cuentas.nombres.contains(&nombre)
        {
            return Task::none();
        }
        app.servicios.local.revision_skin += 1;
        let g = app.servicios.local.revision_skin;
        let previa = app.servicios.local.imagenes.get(&nombre).cloned();
        let guardada = icaro_core::auth::Session::local(&nombre)
            .ok()
            .is_some_and(|s| {
                app.servicios
                    .local
                    .cuentas
                    .skins
                    .contains_key(&s.profile.id)
            });
        app.servicios.local.abierta = true;
        app.sesion.menu = false;
        app.servicios.local.skin = Some(Estado {
            nombre: nombre.clone(),
            generacion: g,
            imagen: previa.as_ref().map(|p| p.0.clone()),
            delgada: previa.map_or(false, |p| p.1),
            archivo: guardada.then(|| "Skin guardada".into()),
            exterior: true,
            error: None,
            guardada,
            ocupada: false,
            cargando: guardada,
            encima: false,
        });
        if guardada {
            let root = crate::estado::directorio(false);
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        skin_local::leer(&root, &nombre)
                            .map(|p| p.map(|(i, m)| (i, m, "Skin guardada".into())))
                    })
                    .await
                    .map_err(|_| "No se pudo leer la skin guardada.".to_string())
                    .and_then(|r| r)
                },
                move |r| m(Accion::Cargada(g, r)),
            );
        }
        return Task::none();
    }
    let Some(s) = app.servicios.local.skin.as_mut() else {
        return Task::none();
    };
    match a {
        Accion::Cargada(g, r) if g == s.generacion && !s.ocupada => {
            s.cargando = false;
            match r {
                Ok(Some((imagen, modelo, nombre))) => {
                    s.imagen = Some(imagen);
                    s.delgada = modelo;
                    s.archivo = Some(nombre);
                    s.error = None;
                }
                Ok(None) => {}
                Err(e) => s.error = Some(e),
            }
        }
        Accion::Guardada(g, quitar, r) if g == s.generacion && s.ocupada => {
            s.ocupada = false;
            app.servicios.auth_busy = false;
            match r {
                Err(e) => s.error = Some(e),
                Ok(cuentas) => {
                    let nombre = s.nombre.clone();
                    if quitar {
                        app.servicios.local.imagenes.remove(&nombre);
                    } else if let Some(i) = &s.imagen {
                        app.servicios
                            .local
                            .imagenes
                            .insert(nombre.clone(), (i.clone(), s.delgada));
                    }
                    app.servicios.local.cuentas = cuentas;
                    app.servicios.local.skin = None;
                    app.avisar(if quitar {
                        "Skin local quitada.".into()
                    } else {
                        format!("Skin de {nombre} guardada. Se usará al abrir Minecraft.")
                    });
                }
            }
        }
        Accion::Cargada(_, _) | Accion::Guardada(_, _, _) => {}
        _ if s.ocupada => {}
        Accion::Cerrar => {
            app.servicios.local.skin = None;
            app.servicios.local.revision_skin += 1;
        }
        Accion::Elegir if !s.cargando => {
            s.cargando = true;
            let g = s.generacion;
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Elegir skin PNG")
                        .add_filter("Skin PNG", &["png"])
                        .pick_file()
                        .await
                        .map(|f| f.path().to_owned())
                },
                move |p| m(Accion::Archivo(g, p)),
            );
        }
        Accion::Archivo(g, p) if g == s.generacion => {
            s.cargando = false;
            if let Some(p) = p {
                return leer(app, p);
            }
        }
        Accion::Soltar(p) => return leer(app, p),
        Accion::Encima(v) => s.encima = v,
        Accion::Modelo(v) => s.delgada = v,
        Accion::Exterior(v) => s.exterior = v,
        Accion::Guardar | Accion::Quitar if !s.cargando => {
            let quitar = matches!(a, Accion::Quitar);
            if !app.servicios.juego.permite_cambiar_cuenta() {
                s.error = Some(
                    "Cierra Minecraft y espera a que termine su preparación para guardar la skin."
                        .into(),
                );
                return Task::none();
            }
            if quitar && !s.guardada {
                s.imagen = None;
                s.archivo = None;
                s.error = None;
                return Task::none();
            }
            if !quitar && (s.imagen.is_none() || s.error.is_some()) {
                return Task::none();
            }
            let imagen = s.imagen.clone();
            let nombre = s.nombre.clone();
            let modelo = s.delgada;
            let g = s.generacion;
            let root = crate::estado::directorio(false);
            s.ocupada = true;
            app.servicios.auth_busy = true;
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        if quitar {
                            skin_local::quitar(&root, &nombre)
                        } else {
                            skin_local::guardar(&root, &nombre, &imagen.unwrap(), modelo)
                        }
                    })
                    .await
                    .map_err(|_| "No se pudo guardar la skin; vuelve a intentarlo.".to_string())
                    .and_then(|r| r)
                },
                move |r| m(Accion::Guardada(g, quitar, r)),
            );
        }
        _ => {}
    }
    Task::none()
}

pub fn vista(app: &App) -> Element<'_, Mensaje> {
    let s = app
        .servicios
        .local
        .skin
        .as_ref()
        .expect("Panel de skin abierto");
    let p = app.paleta();
    let habilitado = !s.ocupada && !s.cargando;
    let escala = if app.ventana.width < 1000. { 6. } else { 7. };
    let imagen = s.imagen.as_ref().map(|i| i.textura.rgba.as_ref());
    let figura = |espalda| {
        column![
            cuenta::cuerpo_skin_local(p, espalda, escala, imagen, s.delgada, s.exterior),
            text(if espalda { "Espalda" } else { "Frente" })
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted)
        ]
        .spacing(12)
        .align_x(Alignment::Center)
    };
    let cuerpos = column![
        row![figura(false), figura(true)]
            .spacing(16)
            .align_y(Alignment::Center),
        c::casilla(p, "Capa exterior", s.exterior, move |v| if habilitado {
            m(Accion::Exterior(v))
        } else {
            Mensaje::Nada
        })
    ]
    .spacing(20)
    .align_x(Alignment::Center);
    let izquierda = container(cuerpos)
        .padding([24, 16])
        .center_x(Length::FillPortion(4))
        .center_y(408)
        .style(estilo::bloque(p.surface_sunken));
    let detalle = if s.cargando {
        "Leyendo la skin…".into()
    } else if let Some(e) = &s.error {
        e.clone()
    } else if s.archivo.is_some() {
        format!(
            "64 × {} px · haz clic o suelta otra para cambiarla",
            if s.imagen.as_ref().is_some_and(|i| i.antigua) {
                32
            } else {
                64
            }
        )
    } else {
        ".png de 64 × 64 o 64 × 32 px · o haz clic para elegir".into()
    };
    let titulo = if s.error.is_some() {
        "No se pudo usar el archivo"
    } else {
        s.archivo.as_deref().unwrap_or("Arrastra tu skin aquí")
    };
    let zona = icaro_ui::zona_soltar::zona_soltar_skin(
        p,
        s.encima,
        titulo.into(),
        detalle,
        s.error.is_some(),
        habilitado.then(|| m(Accion::Elegir)),
    );
    let sugerencia = if s.imagen.as_ref().is_some_and(|i| i.antigua) {
        "Formato antiguo: el brazo y la pierna izquierdos copian a los derechos."
    } else if s
        .imagen
        .as_ref()
        .is_some_and(|i| i.sugerida_delgada != s.delgada)
    {
        "Esta skin parece usar otro modelo de brazos; puedes elegir el que prefieras."
    } else if s.imagen.is_some() {
        "El modelo coincide con la skin."
    } else {
        "Elige el modelo después de subir la skin."
    };
    let modelos = row![
        c::boton(
            p,
            "Clásico · 4 px",
            if !s.delgada {
                c::Variante::Primario
            } else {
                c::Variante::Secundario
            },
            habilitado.then(|| m(Accion::Modelo(false)))
        ),
        c::boton(
            p,
            "Delgado · 3 px",
            if s.delgada {
                c::Variante::Primario
            } else {
                c::Variante::Secundario
            },
            habilitado.then(|| m(Accion::Modelo(true)))
        )
    ]
    .spacing(0);
    let info=container(row![icono(Icono::InfoCaja,Tam::Base,p.info),text("Tú la ves en tus mundos. Los demás jugadores solo la ven si el servidor tiene un plugin de skins.").size(texto::BODY_SM.0).color(p.text)].spacing(12)).padding([14,18]).width(Length::Fill).style(move|_:&iced::Theme|container::Style{background:Some(p.info_soft.into()),border:iced::Border{color:p.info,width:1.,radius:0.into()},..Default::default()});
    let derecha = column![
        zona,
        column![
            c::etiqueta(p, "Modelo"),
            modelos,
            text(sugerencia).size(texto::BODY_SM.0).color(p.text_muted)
        ]
        .spacing(10),
        info
    ]
    .spacing(24)
    .width(Length::FillPortion(5));
    let cuerpo = row![izquierda, derecha].spacing(24);
    let quitar = if s.guardada || s.imagen.is_some() {
        c::boton_con_icono(
            p,
            Icono::Papelera,
            "Quitar skin",
            c::Variante::Peligro,
            habilitado.then(|| m(Accion::Quitar)),
            false,
        )
    } else {
        text("Sin skin, el juego usa una por defecto")
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted)
            .into()
    };
    let pie = row![
        quitar,
        Space::with_width(Length::Fill),
        c::boton(
            p,
            "Cancelar",
            c::Variante::Secundario,
            (!s.ocupada).then(|| m(Accion::Cerrar))
        ),
        c::boton_con_icono(
            p,
            if s.ocupada {
                Icono::Espera
            } else {
                Icono::Check
            },
            if s.ocupada {
                "Guardando…"
            } else {
                "Guardar skin"
            },
            c::Variante::Primario,
            (habilitado && s.imagen.is_some() && s.error.is_none()).then(|| m(Accion::Guardar)),
            false
        )
    ]
    .spacing(12)
    .align_y(Alignment::Center);
    container(column![
        container(
            row![
                column![
                    text(format!("Skin de {}", s.nombre))
                        .font(fuentes::TITULO)
                        .size(24)
                        .color(p.text),
                    row![
                        c::insignia_contorno(p, "Sin conexión"),
                        text("Skin local, sin capas")
                            .size(texto::BODY_SM.0)
                            .color(p.text_muted)
                    ]
                    .spacing(10)
                ]
                .spacing(6)
                .width(Length::Fill),
                cuenta::boton_icono(
                    p,
                    Some(Icono::Cerrar),
                    "",
                    cuenta::Tipo::Fantasma,
                    28.,
                    (!s.ocupada).then(|| m(Accion::Cerrar))
                )
            ]
            .align_y(Alignment::Center)
        )
        .padding([24, 32]),
        container(crate::desplazable_pub(cuerpo.into()))
            .padding([0, 32])
            .height(Length::Fill),
        container(pie).padding([20, 32])
    ])
    .width((app.ventana.width - 48.).clamp(500., 820.))
    .height((app.ventana.height - 100.).clamp(480., 630.))
    .style(estilo::flotante(p))
    .into()
}

pub fn preparar_diseno(app: &mut App, escena: &str) {
    cuentas::preparar_diseno(app, "cuenta-local");
    let mut pixels = image::RgbaImage::new(64, 64);
    // Muestra gris del componente entregado; nunca se guarda en datos de usuario.
    for (x, y, w, h, color) in [
        (0, 8, 32, 8, [201, 201, 196, 255]),
        (0, 8, 32, 2, [43, 43, 42, 255]),
        (24, 8, 8, 8, [43, 43, 42, 255]),
        (16, 20, 24, 12, [110, 110, 105, 255]),
        (32, 20, 8, 12, [74, 74, 71, 255]),
        (40, 20, 16, 12, [110, 110, 105, 255]),
        (32, 52, 16, 12, [110, 110, 105, 255]),
        (44, 29, 4, 3, [201, 201, 196, 255]),
        (36, 61, 4, 3, [201, 201, 196, 255]),
        (0, 20, 16, 12, [58, 58, 56, 255]),
        (16, 52, 16, 12, [58, 58, 56, 255]),
        (0, 29, 16, 3, [28, 28, 27, 255]),
        (16, 61, 16, 3, [28, 28, 27, 255]),
        (9, 12, 2, 1, [244, 244, 240, 255]),
        (13, 12, 2, 1, [244, 244, 240, 255]),
        (10, 12, 1, 1, [17, 17, 17, 255]),
        (13, 12, 1, 1, [17, 17, 17, 255]),
        (11, 14, 2, 1, [138, 138, 133, 255]),
    ] {
        for py in y..y + h {
            for px in x..x + w {
                pixels.put_pixel(px, py, image::Rgba(color));
            }
        }
    }
    let mut png = std::io::Cursor::new(vec![]);
    image::DynamicImage::ImageRgba8(pixels)
        .write_to(&mut png, image::ImageOutputFormat::Png)
        .unwrap();
    let imagen = skin_local::decodificar(&png.into_inner()).unwrap();
    if !escena.contains("vacia") {
        app.servicios.local.imagenes.insert(
            "Jugador_1".into(),
            (imagen.clone(), escena.contains("delgada")),
        );
    }
    app.servicios.local.skin = Some(Estado {
        nombre: "Jugador_1".into(),
        generacion: 1,
        imagen: (!escena.contains("vacia")).then_some(imagen),
        archivo: (!escena.contains("vacia")).then(|| "piedra-roja.png".into()),
        delgada: escena.contains("delgada"),
        exterior: true,
        error: escena
            .contains("error")
            .then(|| "La skin debe medir 64 × 64 o 64 × 32 px; esta mide 100 × 100.".into()),
        guardada: !escena.contains("vacia"),
        ocupada: escena.contains("guardando"),
        cargando: false,
        encima: false,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_y_resultado_antiguo_conservan_la_skin_seleccionada() {
        let (mut app, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Instancias,
            false,
            None,
            false,
            crate::servicios::Estado::nuevo(true),
            None,
        );
        preparar_diseno(&mut app, "skin-local");
        let previa = app
            .servicios
            .local
            .skin
            .as_ref()
            .unwrap()
            .imagen
            .as_ref()
            .unwrap()
            .textura
            .png
            .clone();
        let _ = actualizar(&mut app, Accion::Cargada(0, Err("Antiguo".into())));
        assert!(app.servicios.local.skin.as_ref().unwrap().error.is_none());
        let _ = actualizar(&mut app, Accion::Cargada(1, Err("No es PNG".into())));
        let s = app.servicios.local.skin.as_ref().unwrap();
        assert_eq!(s.imagen.as_ref().unwrap().textura.png, previa);
        assert_eq!(s.error.as_deref(), Some("No es PNG"));
        let _ = actualizar(&mut app, Accion::Guardar);
        assert!(!app.servicios.auth_busy);
        let _ = actualizar(&mut app, Accion::Cerrar);
        let _ = actualizar(&mut app, Accion::Cargada(1, Ok(None)));
        assert!(app.servicios.local.skin.is_none());
    }

    #[test]
    fn estados_de_skin_sin_red_ni_guardado_de_usuario() {
        let (mut app, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Instancias,
            false,
            None,
            false,
            crate::servicios::Estado::nuevo(true),
            None,
        );
        for escena in [
            "skin-local",
            "skin-local-vacia",
            "skin-local-error",
            "skin-local-guardando",
            "skin-local-delgada",
        ] {
            preparar_diseno(&mut app, escena);
            let _ = crate::vista(&app);
        }
    }
}
