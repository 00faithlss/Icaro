// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Contenido público del catálogo: Markdown nativo e imágenes acotadas.

use crate::{servicios::Accion, Mensaje};
use icaro_core::modrinth::{self, Hit};
use icaro_ui::{
    fuentes,
    iconos::{icono, Icono, Tam},
    tema::{espacio, Paleta},
};
use iced::{
    widget::{button, column, container, image, rich_text, row, span, text, Space},
    Element, Font, Length,
};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::collections::{HashMap, HashSet, VecDeque};

/// El texto de autores puede requerir fuentes de respaldo para CJK, árabe o emoji.
pub fn texto_publico<'a>(
    contenido: impl iced::advanced::text::IntoFragment<'a>,
) -> iced::widget::Text<'a> {
    text(contenido).shaping(iced::advanced::text::Shaping::Advanced)
}

pub fn insignia_loader<'a>(p: Paleta, loader: &str) -> Element<'a, Mensaje> {
    icaro_ui::componentes::insignia(
        p,
        icaro_ui::componentes::Estado::Info,
        &format!("Loader · {}", crate::ficha::loader(loader)),
    )
}

pub fn tarjeta_proyecto<'a>(p: Paleta, hit: &Hit, images: &Imagenes) -> Element<'a, Mensaje> {
    tarjeta_proyecto_con_accion(p, hit, images, None)
}
pub fn tarjeta_proyecto_con_accion<'a>(p: Paleta, hit: &Hit, images: &Imagenes, accion: Option<Element<'a, Mensaje>>) -> Element<'a, Mensaje> {
    let con_accion = accion.is_some();
    let loaders = row(hit
        .categories
        .iter()
        .filter(|c| matches!(c.as_str(), "fabric" | "forge" | "neoforge" | "quilt"))
        .map(|l| insignia_loader(p, l))
        .collect::<Vec<_>>())
    .spacing(espacio::S2)
    .wrap();
    let body = row![
        images.vista(p, hit.icon_url.as_deref(), 88.),
        column![
            texto_publico(hit.title.chars().take(256).collect::<String>())
                .font(fuentes::TITULO)
                .size(icaro_ui::tema::texto::HEADING.0)
                .color(p.text),
            texto_publico(hit.description.chars().take(240).collect::<String>())
                .color(p.text_muted),
            loaders,
            texto_publico(format!(
                "{} · {} descargas",
                hit.author,
                numero(hit.downloads)
            ))
            .color(p.text_muted),
        ]
        .spacing(espacio::S3)
        .width(Length::Fill),
    ]
    .spacing(espacio::S5)
    .width(Length::Fill);
    let card: Element<'a, Mensaje> = button(body)
        .padding(espacio::S4)
        .width(Length::Fill)
        .on_press(Mensaje::Servicio(Accion::Proyecto(hit.project_id.clone())))
        .style(move |_, status| iced::widget::button::Style {
            background: Some(iced::Background::Color(
                if matches!(
                    status,
                    iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
                ) {
                    p.surface_hover
                } else {
                    p.surface
                },
            )),
            text_color: p.text,
            border: iced::Border {
                color: if matches!(
                    status,
                    iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
                ) {
                    p.text
                } else {
                    p.border
                },
                width: if con_accion { 0. } else { 2. },
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into();
    if let Some(action) = accion { container(row![card, container(action).padding(espacio::S4)].align_y(iced::Alignment::Center)).width(Length::Fill).style(move |_| container::Style { background: Some(p.surface.into()), border: iced::Border { color: p.border, width: 2., radius: 0.0.into() }, ..Default::default() }).into() } else { card }
}

#[derive(Debug, Clone)]
pub struct Imagen {
    handle: image::Handle,
    bytes: usize,
}
impl Imagen {
    pub fn handle(&self) -> image::Handle {
        self.handle.clone()
    }
}
pub fn decodificar(bytes: Vec<u8>) -> Option<Imagen> {
    decodificar_hasta(bytes, 512)
}
pub fn decodificar_visor(bytes: Vec<u8>) -> Option<Imagen> {
    decodificar_hasta(bytes, 4096)
}
fn decodificar_hasta(bytes: Vec<u8>, lado: u32) -> Option<Imagen> {
    let mut lector = ::image::io::Reader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limites = ::image::io::Limits::default();
    limites.max_image_width = Some(4096);
    limites.max_image_height = Some(4096);
    limites.max_alloc = Some(64 * 1024 * 1024);
    lector.limits(limites);
    let original = lector.decode().ok()?;
    let pixels = if original.width() > lado || original.height() > lado {
        original.thumbnail(lado, lado)
    } else {
        original
    }
    .to_rgba8();
    let size = pixels.len();
    Some(Imagen {
        handle: image::Handle::from_rgba(pixels.width(), pixels.height(), pixels.into_raw()),
        bytes: size,
    })
}

#[derive(Default)]
pub struct Imagenes {
    valores: HashMap<String, Option<Imagen>>,
    orden: VecDeque<String>,
    pendientes: HashSet<String>,
    bytes: usize,
}
impl Imagenes {
    pub fn pedir(&mut self, urls: impl IntoIterator<Item = String>) -> Vec<String> {
        urls.into_iter()
            .filter(|url| {
                self.pendientes.len() < 64
                    && modrinth::supported_image(url)
                    && !self.valores.contains_key(url)
                    && self.pendientes.insert(url.clone())
            })
            .collect()
    }
    pub fn recibir(&mut self, url: String, imagen: Option<Imagen>) {
        self.pendientes.remove(&url);
        if self.valores.contains_key(&url) {
            return;
        }
        self.bytes += imagen.as_ref().map_or(0, |i| i.bytes);
        self.orden.push_back(url.clone());
        self.valores.insert(url, imagen);
        while self.valores.len() > 64 || self.bytes > 32 * 1024 * 1024 {
            if let Some(url) = self.orden.pop_front() {
                self.bytes -= self.valores.remove(&url).flatten().map_or(0, |i| i.bytes);
            } else {
                break;
            }
        }
    }
    pub fn vista<'a>(&self, p: Paleta, url: Option<&str>, lado: f32) -> Element<'a, Mensaje> {
        self.vista_opacidad(p,url,lado,1.)
    }
    pub fn vista_opacidad<'a>(&self, mut p: Paleta, url: Option<&str>, lado: f32, opacidad: f32) -> Element<'a, Mensaje> {
        if let Some(imagen) = url
            .and_then(|u| self.valores.get(u))
            .and_then(Option::as_ref)
        {
            return container(
                image(imagen.handle.clone())
                    .width(lado)
                    .height(lado)
                    .opacity(opacidad)
                    .content_fit(iced::ContentFit::Contain),
            )
            .width(lado)
            .height(lado)
            .center_x(lado)
            .center_y(lado)
            .into();
        }
        p.text_muted.a *= opacidad;
        container(icono(Icono::Mods, Tam::Lg, p.text_muted))
            .width(lado)
            .height(lado)
            .center_x(lado)
            .center_y(lado)
            .style(icaro_ui::estilo::marco(p))
            .into()
    }
    fn ilustracion<'a>(&self, url: &str) -> Option<Element<'a, Mensaje>> {
        let imagen = self.valores.get(url)?.as_ref()?;
        Some(
            image(imagen.handle.clone())
                .width(Length::Fill)
                .content_fit(iced::ContentFit::Contain)
                .into(),
        )
    }
    pub fn pendiente(&self, url: &str) -> bool {
        self.pendientes.contains(url)
    }
    pub fn portada_alta<'a>(&self, url: &str, alto: f32) -> Option<Element<'a, Mensaje>> {
        let imagen = self.valores.get(url)?.as_ref()?;
        Some(
            image(imagen.handle.clone())
                .width(Length::Fill)
                .height(alto)
                .content_fit(iced::ContentFit::Contain)
                .into(),
        )
    }
}

#[derive(Default, Debug, Clone)]
struct Fragmento {
    texto: String,
    fuerte: bool,
    cursiva: bool,
    tachado: bool,
    codigo: bool,
    enlace: Option<String>,
}
#[derive(Debug, Clone)]
enum Bloque {
    Texto {
        partes: Vec<Fragmento>,
        nivel: Option<u8>,
        prefijo: String,
        cita: bool,
    },
    Imagen {
        url: String,
        texto: String,
        enlace: Option<String>,
    },
    Regla,
}
#[derive(Default, Debug, Clone)]
pub struct Documento {
    bloques: Vec<Bloque>,
}

impl Documento {
    pub fn parse(body: &str) -> Self {
        let mut documento = Self::default();
        let mut partes = Vec::new();
        let mut estilo = Fragmento::default();
        let mut nivel = None;
        let mut prefijo = String::new();
        let mut citas = 0usize;
        let mut listas: Vec<Option<u64>> = Vec::new();
        let mut imagen: Option<(String, String, Option<String>)> = None;
        let mut enlaces = Vec::new();
        let mut codigo = false;
        let emitir =
            |doc: &mut Self, partes: &mut Vec<Fragmento>, nivel, prefijo: &mut String, cita| {
                if !partes.is_empty() {
                    doc.bloques.push(Bloque::Texto {
                        partes: std::mem::take(partes),
                        nivel,
                        prefijo: std::mem::take(prefijo),
                        cita,
                    });
                }
            };
        // Se acota en una frontera UTF-8. No se ejecuta HTML ni se cargan sus recursos.
        let fin = body
            .char_indices()
            .nth(60_000)
            .map_or(body.len(), |(i, _)| i);
        for event in Parser::new_ext(
            &body[..fin],
            Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES,
        ) {
            let inline = matches!(&event, Event::Code(_));
            match event {
                Event::Start(Tag::Heading { level, .. }) => nivel = Some(level as u8),
                Event::Start(Tag::Strong) => estilo.fuerte = true,
                Event::End(TagEnd::Strong) => estilo.fuerte = false,
                Event::Start(Tag::Emphasis) => estilo.cursiva = true,
                Event::End(TagEnd::Emphasis) => estilo.cursiva = false,
                Event::Start(Tag::Strikethrough) => estilo.tachado = true,
                Event::End(TagEnd::Strikethrough) => estilo.tachado = false,
                Event::Start(Tag::Link { dest_url, .. }) => {
                    enlaces.push(estilo.enlace.take());
                    estilo.enlace = Some(dest_url.into_string());
                }
                Event::End(TagEnd::Link) => estilo.enlace = enlaces.pop().flatten(),
                Event::Start(Tag::Image { dest_url, .. }) => {
                    emitir(&mut documento, &mut partes, nivel, &mut prefijo, citas > 0);
                    imagen = Some((dest_url.into_string(), String::new(), estilo.enlace.clone()));
                }
                Event::End(TagEnd::Image) => {
                    if let Some((url, texto, enlace)) = imagen.take() {
                        documento
                            .bloques
                            .push(Bloque::Imagen { url, texto, enlace });
                    }
                }
                Event::Start(Tag::BlockQuote(_)) => citas += 1,
                Event::End(TagEnd::BlockQuote) => citas = citas.saturating_sub(1),
                Event::Start(Tag::List(inicio)) => listas.push(inicio),
                Event::End(TagEnd::List(_)) => {
                    listas.pop();
                }
                Event::Start(Tag::Item) => {
                    prefijo = "  ".repeat(listas.len().saturating_sub(1).min(8));
                    match listas.last_mut() {
                        Some(Some(n)) => {
                            prefijo.push_str(&format!("{n}. "));
                            *n += 1;
                        }
                        _ => prefijo.push_str("• "),
                    }
                }
                Event::Start(Tag::CodeBlock(_)) => codigo = true,
                Event::End(TagEnd::CodeBlock) => {
                    emitir(&mut documento, &mut partes, nivel, &mut prefijo, citas > 0);
                    codigo = false;
                }
                Event::Text(s) | Event::Code(s) => {
                    if let Some((_, alt, _)) = imagen.as_mut() {
                        alt.push_str(&s);
                    } else {
                        partes.push(Fragmento {
                            texto: s.into_string(),
                            codigo: codigo || inline,
                            ..estilo.clone()
                        });
                    }
                }
                Event::SoftBreak | Event::HardBreak => partes.push(Fragmento {
                    texto: "\n".into(),
                    ..estilo.clone()
                }),
                Event::End(TagEnd::TableCell) => partes.push(Fragmento {
                    texto: "  |  ".into(),
                    ..estilo.clone()
                }),
                Event::End(
                    TagEnd::Paragraph | TagEnd::Item | TagEnd::TableRow | TagEnd::TableHead,
                ) => emitir(&mut documento, &mut partes, nivel, &mut prefijo, citas > 0),
                Event::End(TagEnd::Heading(_)) => {
                    emitir(&mut documento, &mut partes, nivel, &mut prefijo, citas > 0);
                    nivel = None;
                }
                Event::Rule => documento.bloques.push(Bloque::Regla),
                _ => {}
            }
        }
        emitir(&mut documento, &mut partes, nivel, &mut prefijo, citas > 0);
        documento
    }
    pub fn imagenes(&self) -> Vec<String> {
        self.bloques
            .iter()
            .filter_map(|b| {
                if let Bloque::Imagen { url, .. } = b {
                    Some(url.clone())
                } else {
                    None
                }
            })
            .take(8)
            .collect()
    }
    pub fn vista<'a>(&'a self, p: Paleta, imagenes: &Imagenes) -> Element<'a, Mensaje> {
        let mut contenido = column![].spacing(espacio::S4).width(Length::Fill);
        for bloque in &self.bloques {
            match bloque {
                Bloque::Texto {
                    partes,
                    nivel,
                    prefijo,
                    cita,
                } => {
                    let mut spans = Vec::new();
                    if !prefijo.is_empty() {
                        spans.push(span(prefijo.clone()).font(fuentes::MONO));
                    }
                    for parte in partes {
                        let font = if parte.codigo {
                            fuentes::MONO
                        } else {
                            Font {
                                weight: if parte.fuerte || nivel.is_some() {
                                    iced::font::Weight::Bold
                                } else {
                                    iced::font::Weight::Normal
                                },
                                style: if parte.cursiva {
                                    iced::font::Style::Italic
                                } else {
                                    iced::font::Style::Normal
                                },
                                ..fuentes::CUERPO
                            }
                        };
                        let mut s = span(parte.texto.clone())
                            .font(font)
                            .strikethrough(parte.tachado);
                        if let Some(link) = &parte.enlace {
                            s = s.link(link.clone()).color(p.accent_text).underline(true);
                        }
                        spans.push(s);
                    }
                    let size = match nivel {
                        Some(1) => 30.,
                        Some(2) => 26.,
                        Some(3) => 22.,
                        Some(_) => 18.,
                        None => 15.,
                    };
                    let rich: Element<String> = rich_text(spans)
                        .size(size)
                        .color(p.text)
                        .width(Length::Fill)
                        .into();
                    let rich = rich.map(|url| Mensaje::Servicio(Accion::Enlace(url)));
                    if *cita {
                        contenido = contenido.push(
                            row![
                                container(Space::new(3, Length::Fill))
                                    .style(icaro_ui::estilo::bloque(p.border)),
                                container(rich).padding(espacio::S3)
                            ]
                            .height(Length::Shrink),
                        );
                    } else {
                        contenido = contenido.push(rich);
                    }
                }
                Bloque::Imagen { url, texto, enlace } => {
                    if let Some(img) = imagenes.ilustracion(url) {
                        contenido = contenido.push(img);
                    }
                    if !texto.is_empty() {
                        let enlace = enlace.clone().unwrap_or_else(|| url.clone());
                        let caption: Element<String> = rich_text(vec![span(texto.clone())
                            .link(enlace)
                            .color(p.accent_text)
                            .underline(true)])
                        .size(15)
                        .width(Length::Fill)
                        .into();
                        contenido = contenido
                            .push(caption.map(|url| Mensaje::Servicio(Accion::Enlace(url))));
                    }
                }
                Bloque::Regla => {
                    contenido = contenido.push(
                        container(Space::new(Length::Fill, 1))
                            .style(icaro_ui::estilo::bloque(p.border)),
                    )
                }
            }
        }
        contenido.into()
    }
}

pub fn numero(n: u64) -> String {
    let cifras = n.to_string();
    cifras
        .chars()
        .enumerate()
        .fold(String::new(), |mut s, (i, c)| {
            if i > 0 && (cifras.len() - i) % 3 == 0 {
                s.push('.');
            }
            s.push(c);
            s
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn markdown_resolves_images_links_and_styles_without_executing_html() {
        let d = Documento::parse("# Título\n\n**Negrita** y *cursiva* [ayuda][a].\n\n[![Instalar](https://cdn.modrinth.com/data/x/banner.png)](https://example.com/install)\n\n> Cita\n\n1. Uno\n2. Dos\n\n<script>PRIVATE_SCRIPT</script>\n\n[a]: https://example.com/help");
        assert!(matches!(
            &d.bloques[0],
            Bloque::Texto { nivel: Some(1), .. }
        ));
        let parts: Vec<_> = d
            .bloques
            .iter()
            .filter_map(|b| match b {
                Bloque::Texto { partes, .. } => Some(partes),
                _ => None,
            })
            .flatten()
            .collect();
        assert!(parts.iter().any(|p| p.fuerte && p.texto == "Negrita"));
        assert!(parts.iter().any(|p| p.cursiva && p.texto == "cursiva"));
        assert!(parts
            .iter()
            .any(|p| p.enlace.as_deref() == Some("https://example.com/help")));
        assert!(d.bloques.iter().any(|b| matches!(b, Bloque::Imagen { enlace: Some(link), .. } if link == "https://example.com/install")));
        assert!(!parts.iter().any(|p| p.texto.contains("PRIVATE_SCRIPT")));
        assert!(d
            .bloques
            .iter()
            .any(|b| matches!(b, Bloque::Texto { prefijo, .. } if prefijo == "2. ")));
        assert_eq!(d.imagenes().len(), 1);
    }
    #[test]
    fn images_are_bounded_deduplicated_and_invalid_formats_have_a_fallback() {
        assert!(decodificar(b"<svg><script>bad()</script></svg>".to_vec()).is_none());
        let mut bytes = std::io::Cursor::new(Vec::new());
        ::image::DynamicImage::new_rgba8(1200, 600)
            .write_to(&mut bytes, ::image::ImageOutputFormat::Png)
            .unwrap();
        let imagen = decodificar(bytes.into_inner()).unwrap();
        assert!(imagen.bytes <= 512 * 512 * 4);
        let mut cache = Imagenes::default();
        let url = "https://cdn.modrinth.com/data/x/icon.png".to_owned();
        assert_eq!(cache.pedir([url.clone(), url.clone()]), vec![url.clone()]);
        assert!(cache
            .pedir(["https://localhost/private.png".into()])
            .is_empty());
        cache.recibir(url.clone(), Some(imagen));
        assert!(cache.pedir([url]).is_empty());
        for i in 0..70 {
            cache.recibir(format!("https://cdn.modrinth.com/data/{i}/icon.png"), None);
        }
        assert!(cache.valores.len() <= 64 && cache.bytes <= 32 * 1024 * 1024);
        assert_eq!(numero(18332198), "18.332.198");
    }
    #[test]
    fn viewer_keeps_more_detail_than_thumbnails_without_upscaling_small_images() {
        fn png(w: u32, h: u32) -> Vec<u8> {
            let mut bytes = std::io::Cursor::new(Vec::new());
            ::image::DynamicImage::new_rgba8(w, h)
                .write_to(&mut bytes, ::image::ImageOutputFormat::Png)
                .unwrap();
            bytes.into_inner()
        }
        let original = png(1200, 600);
        assert_eq!(
            decodificar_visor(original.clone()).unwrap().bytes,
            1200 * 600 * 4
        );
        assert!(decodificar(original).unwrap().bytes < 1200 * 600 * 4);
        assert_eq!(decodificar_visor(png(24, 16)).unwrap().bytes, 24 * 16 * 4);
        assert_eq!(
            decodificar_visor(png(3000, 1800)).unwrap().bytes,
            3000 * 1800 * 4
        );
        assert!(decodificar_visor(png(4097, 20)).is_none());
        assert!(decodificar_visor(b"<svg><script/></svg>".to_vec()).is_none());
    }

    #[tokio::test]
    #[ignore = "Consulta de lectura a la galería pública de Modrinth; requiere red"]
    async fn viewer_loads_the_original_instead_of_the_350_pixel_preview() {
        let catalog = modrinth::Catalog::new().unwrap();
        let project = catalog.project("fabulously-optimized").await.unwrap();
        let image = project
            .gallery
            .iter()
            .find(|g| g.title.as_deref() == Some("FPS in high render distance"))
            .expect("La imagen de FPS de la galería");
        assert_ne!(image.original_url(), image.url);
        let preview = catalog.image(&image.url).await.unwrap();
        let original = catalog.gallery_image(image.original_url()).await.unwrap();
        let dimensions = |bytes: &[u8]| {
            ::image::io::Reader::new(std::io::Cursor::new(bytes))
                .with_guessed_format()
                .unwrap()
                .into_dimensions()
                .unwrap()
        };
        let (pw, ph) = dimensions(&preview);
        let (ow, oh) = dimensions(&original);
        assert!(ow > pw && oh > ph);
        assert_eq!(
            decodificar_visor(original).unwrap().bytes,
            ow as usize * oh as usize * 4
        );
        println!("Vista previa: {pw}×{ph}; original del visor: {ow}×{oh}");
    }
}
