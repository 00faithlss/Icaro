// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Atajos globales de búsqueda y navegación de la barra lateral.

use crate::Mensaje;
use icaro_ui::shell::Seccion;
use iced::keyboard::{Key, Modifiers};

pub fn global(key: &Key, modifiers: Modifiers) -> Option<Mensaje> {
    if !modifiers.control() || modifiers.alt() || modifiers.logo() {
        return None;
    }
    match key.as_ref() {
        Key::Character("k" | "K") => Some(Mensaje::AbrirBusqueda),
        Key::Character(n) if !modifiers.shift() => {
            let index = match n {
                "1" => 0,
                "2" => 1,
                "3" => 2,
                "4" => 3,
                "5" => 4,
                "6" => 5,
                "7" => 6,
                _ => return None,
            };
            Some(Mensaje::AtajoSeccion(Seccion::TODAS[index]))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_numbers_follow_the_sidebar_and_do_not_capture_plain_typing() {
        for (i, section) in Seccion::TODAS.into_iter().enumerate() {
            let key = Key::Character((i + 1).to_string().into());
            assert!(
                matches!(global(&key, Modifiers::CTRL), Some(Mensaje::AtajoSeccion(s)) if s == section)
            );
            for modifiers in [
                Modifiers::empty(),
                Modifiers::CTRL | Modifiers::ALT,
                Modifiers::CTRL | Modifiers::SHIFT,
                Modifiers::CTRL | Modifiers::LOGO,
            ] {
                assert!(global(&key, modifiers).is_none());
            }
        }
        for n in ["0", "8", "9"] {
            assert!(global(&Key::Character(n.into()), Modifiers::CTRL).is_none());
        }
        assert!(matches!(
            global(&Key::Character("k".into()), Modifiers::CTRL),
            Some(Mensaje::AbrirBusqueda)
        ));
    }
}
