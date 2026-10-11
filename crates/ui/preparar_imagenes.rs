// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Preparación de recursos durante la compilación, fuera de la aplicación.
use image::RgbaImage;

pub fn preparar(
    fuente: &[u8],
    recorte: Option<[f32; 4]>,
    invertir: bool,
    circular: bool,
) -> image::ImageResult<RgbaImage> {
    let mut imagen = image::load_from_memory(fuente)?;
    if let Some([x0, y0, x1, y1]) = recorte {
        let (w, h) = (imagen.width() as f32, imagen.height() as f32);
        let (x, y) = ((x0 * w).round() as u32, (y0 * h).round() as u32);
        imagen = imagen.crop_imm(
            x,
            y,
            (x1 * w).round() as u32 - x,
            (y1 * h).round() as u32 - y,
        );
    }
    if invertir {
        imagen.invert();
    }
    if imagen.width().max(imagen.height()) > 2048 {
        imagen = imagen.resize(2048, 2048, image::imageops::FilterType::Lanczos3);
    }
    let mut imagen = imagen.into_rgba8();
    if circular {
        let (w, h) = (imagen.width() as f32, imagen.height() as f32);
        let (cx, cy, rx, ry) = (w * 0.503, h * 0.503, w * 0.477, h * 0.474);
        for (x, y, pixel) in imagen.enumerate_pixels_mut() {
            let dx = (x as f32 + 0.5 - cx) / rx;
            let dy = (y as f32 + 0.5 - cy) / ry;
            let distancia = (dx * dx + dy * dy).sqrt();
            let cobertura = ((1.0 - distancia) * rx.min(ry) + 0.5).clamp(0.0, 1.0);
            pixel.0[3] = (pixel.0[3] as f32 * cobertura).round() as u8;
        }
    }
    Ok(imagen)
}
