// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Bordes para cambiar el tamaño de la ventana sin una barra de título del sistema.

use crate::Mensaje;
use iced::{Element, Task};

#[derive(Debug, Clone, Copy)]
pub enum Borde {
    Izquierdo,
    Derecho,
    Superior,
    Inferior,
    SuperiorIzquierdo,
    SuperiorDerecho,
    InferiorIzquierdo,
    InferiorDerecho,
}

pub fn con_bordes(contenido: Element<'_, Mensaje>, maximizada: bool) -> Element<'_, Mensaje> {
    #[cfg(windows)]
    if !maximizada {
        use iced::widget::{column, mouse_area, row, stack, Space};
        use iced::{mouse::Interaction, Length};
        let zona = |borde, ancho, alto, cursor| {
            mouse_area(Space::new(ancho, alto))
                .on_press(Mensaje::Redimensionar(borde))
                .interaction(cursor)
        };
        let esquina = Length::Fixed(12.0);
        let grosor = Length::Fixed(6.0);
        let bordes = column![
            row![
                zona(
                    Borde::SuperiorIzquierdo,
                    esquina,
                    grosor,
                    Interaction::ResizingDiagonallyDown
                ),
                zona(
                    Borde::Superior,
                    Length::Fill,
                    grosor,
                    Interaction::ResizingVertically
                ),
                zona(
                    Borde::SuperiorDerecho,
                    esquina,
                    grosor,
                    Interaction::ResizingDiagonallyUp
                ),
            ],
            row![
                zona(
                    Borde::Izquierdo,
                    grosor,
                    Length::Fill,
                    Interaction::ResizingHorizontally
                ),
                Space::new(Length::Fill, Length::Fill),
                zona(
                    Borde::Derecho,
                    grosor,
                    Length::Fill,
                    Interaction::ResizingHorizontally
                ),
            ]
            .height(Length::Fill),
            row![
                zona(
                    Borde::InferiorIzquierdo,
                    esquina,
                    grosor,
                    Interaction::ResizingDiagonallyUp
                ),
                zona(
                    Borde::Inferior,
                    Length::Fill,
                    grosor,
                    Interaction::ResizingVertically
                ),
                zona(
                    Borde::InferiorDerecho,
                    esquina,
                    grosor,
                    Interaction::ResizingDiagonallyDown
                ),
            ],
        ]
        .width(Length::Fill)
        .height(Length::Fill);
        return stack![contenido, bordes].into();
    }
    #[cfg(not(windows))]
    let _ = maximizada;
    contenido
}

pub fn redimensionar(borde: Borde) -> Task<Mensaje> {
    #[cfg(windows)]
    {
        use iced::window::{self, raw_window_handle::RawWindowHandle};
        use windows_sys::Win32::{
            Foundation::POINT,
            UI::{Input::KeyboardAndMouse::ReleaseCapture, WindowsAndMessaging::*},
        };
        let direccion = match borde {
            Borde::Izquierdo => HTLEFT,
            Borde::Derecho => HTRIGHT,
            Borde::Superior => HTTOP,
            Borde::Inferior => HTBOTTOM,
            Borde::SuperiorIzquierdo => HTTOPLEFT,
            Borde::SuperiorDerecho => HTTOPRIGHT,
            Borde::InferiorIzquierdo => HTBOTTOMLEFT,
            Borde::InferiorDerecho => HTBOTTOMRIGHT,
        };
        return window::get_oldest().and_then(move |id| {
            window::run_with_handle(id, move |handle| {
                if let RawWindowHandle::Win32(handle) = handle.as_raw() {
                    let hwnd = handle.hwnd.get() as _;
                    let mut posicion = POINT { x: 0, y: 0 };
                    // El handle pertenece a la ventana viva y el callback corre en su hilo.
                    // Windows gestiona el arrastre, el DPI y los límites de tamaño.
                    unsafe {
                        if IsZoomed(hwnd) == 0 && GetCursorPos(&mut posicion) != 0 {
                            let coordenadas =
                                (posicion.x as u16 as u32) | ((posicion.y as u16 as u32) << 16);
                            ReleaseCapture();
                            // El bucle de arrastre consume la liberación física del botón.
                            // Libera también el estado del botón en la interfaz antes de entrar.
                            PostMessageW(hwnd, WM_LBUTTONUP, 0, 0);
                            PostMessageW(
                                hwnd,
                                WM_NCLBUTTONDOWN,
                                direccion as usize,
                                coordenadas as isize,
                            );
                        }
                    }
                }
                Mensaje::Nada
            })
        });
    }
    #[cfg(not(windows))]
    {
        let _ = borde;
        Task::none()
    }
}
