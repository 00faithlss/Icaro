// SPDX-License-Identifier: GPL-3.0-only
//! Preferencia de apariencia del escritorio, sin trabajo externo al dibujar.
use crate::Mensaje;
pub fn sistema_claro() -> bool {
    #[cfg(windows)] {
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};
        return RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
            .and_then(|key|key.get_value::<u32,_>("AppsUseLightTheme")).map(|v|v!=0).unwrap_or(false);
    }
    #[cfg(not(windows))] {
        let preferencia=std::process::Command::new("gsettings").args(["get","org.gnome.desktop.interface","color-scheme"]).output();
        return preferencia.ok().filter(|o|o.status.success()).is_some_and(|o|String::from_utf8_lossy(&o.stdout).contains("prefer-light"));
    }
}
pub fn comprobar() -> iced::Task<Mensaje> {
    iced::Task::perform(async {tokio::task::spawn_blocking(sistema_claro).await.unwrap_or(false)},Mensaje::TemaSistema)
}
