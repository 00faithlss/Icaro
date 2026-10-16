// SPDX-License-Identifier: GPL-3.0-only
//! Perfiles locales sin tokens; el almacén de Microsoft permanece separado.
use crate::{almacenamiento, auth::Session};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cuentas {
    pub nombres: Vec<String>,
    pub activa: Option<String>,
    #[serde(default)]
    pub skins: std::collections::BTreeMap<String, Skin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Skin {
    pub sha256: String,
    pub delgada: bool,
}

impl Cuentas {
    pub fn cargar(raiz: &Path) -> Result<Self, String> {
        let ruta = raiz.join("accounts/locales.json");
        let metadata = match std::fs::symlink_metadata(&ruta) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(_) => return Err("No se pudieron leer las cuentas locales.".into()),
        };
        if !metadata.is_file() || metadata.len() > 16 * 1024 {
            return Err("El archivo de cuentas locales no es válido.".into());
        }
        almacenamiento::regular(&ruta)?;
        let valor: Self = serde_json::from_slice(
            &std::fs::read(ruta).map_err(|_| "No se pudieron leer las cuentas locales.")?,
        )
        .map_err(|_| "El archivo de cuentas locales está dañado; se conserva para recuperarlo.")?;
        if valor.nombres.len() > 32
            || valor.skins.len() > 32
            || valor.skins.iter().any(|(id, s)| {
                s.sha256.len() != 64
                    || !s.sha256.bytes().all(|b| b.is_ascii_hexdigit())
                    || !valor
                        .nombres
                        .iter()
                        .any(|n| Session::local(n).is_ok_and(|p| p.profile.id == *id))
            })
            || valor.nombres.iter().any(|n| Session::local(n).is_err())
            || valor
                .activa
                .as_ref()
                .is_some_and(|n| !valor.nombres.contains(n))
        {
            return Err("El archivo de cuentas locales no es válido.".into());
        }
        Ok(valor)
    }

    pub(crate) fn guardar(&self, raiz: &Path) -> Result<(), String> {
        almacenamiento::escribir_json(
            &raiz.join("accounts/locales.json"),
            &serde_json::to_value(self).map_err(|e| e.to_string())?,
        )
    }

    pub fn activar(raiz: &Path, nombre: &str) -> Result<Self, String> {
        Session::local(nombre)?;
        let mut cuentas = Self::cargar(raiz)?;
        if !cuentas.nombres.iter().any(|n| n == nombre) {
            if cuentas
                .nombres
                .iter()
                .any(|n| n.eq_ignore_ascii_case(nombre))
            {
                return Err(
                    "Ya tienes una cuenta sin conexión con ese nombre. Úsala desde la lista."
                        .into(),
                );
            }
            if cuentas.nombres.len() >= 32 {
                return Err("Puedes guardar hasta 32 cuentas locales.".into());
            }
            cuentas.nombres.insert(0, nombre.into());
        }
        cuentas.activa = Some(nombre.into());
        cuentas.guardar(raiz)?;
        Ok(cuentas)
    }

    pub fn desactivar(raiz: &Path) -> Result<(), String> {
        let mut cuentas = Self::cargar(raiz)?;
        cuentas.activa = None;
        cuentas.guardar(raiz)
    }

    /// Solo quita el perfil del launcher; no accede a mundos ni instancias.
    pub fn eliminar(raiz: &Path, nombre: &str) -> Result<Self, String> {
        let mut cuentas = Self::cargar(raiz)?;
        if let Ok(sesion) = Session::local(nombre) {
            cuentas.skins.remove(&sesion.profile.id);
        }
        cuentas.nombres.retain(|n| n != nombre);
        if cuentas.activa.as_deref() == Some(nombre) {
            cuentas.activa = cuentas.nombres.first().cloned();
        }
        cuentas.guardar(raiz)?;
        Ok(cuentas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identidad_local_es_estable_y_no_acredita_microsoft() {
        let s = Session::local("Notch").unwrap();
        assert_eq!(s.profile.id, "b50ad385829d3141a2167e7d7539ba7f");
        assert!(s.es_local() && s.puede_jugar() && !s.needs_refresh());
        assert!(!s.has_minecraft_access());
        let mut alterada = Session::local("Jugador_1").unwrap();
        alterada.profile.id = "0".repeat(32);
        assert!(!alterada.puede_jugar());
        assert_ne!(s.profile.id, Session::local("notch").unwrap().profile.id);
        for n in [
            "",
            "ab",
            "nombre con espacio",
            "../Jugador",
            "Jugador\n",
            "abcdefghijklmnopq",
            "niño",
        ] {
            assert!(Session::local(n).is_err(), "{n:?}");
        }
    }
    #[test]
    fn restaura_y_cambia_cuentas_sin_escribir_credenciales() {
        let raiz = std::env::temp_dir().join(format!(
            "icaro-local-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        Cuentas::activar(&raiz, "Jugador_1").unwrap();
        Cuentas::activar(&raiz, "Jugador_2").unwrap();
        let cuentas = Cuentas::cargar(&raiz).unwrap();
        assert_eq!(cuentas.nombres.len(), 2);
        assert_eq!(cuentas.activa.as_deref(), Some("Jugador_2"));
        assert!(Cuentas::activar(&raiz, "jugador_1").is_err());
        std::fs::write(raiz.join("mundo-no-tocar"), b"mundo original").unwrap();
        let cuentas = Cuentas::eliminar(&raiz, "Jugador_2").unwrap();
        assert_eq!(cuentas.activa.as_deref(), Some("Jugador_1"));
        assert_eq!(
            std::fs::read(raiz.join("mundo-no-tocar")).unwrap(),
            b"mundo original"
        );
        Cuentas::eliminar(&raiz, "Jugador_1").unwrap();
        assert!(Cuentas::cargar(&raiz).unwrap().activa.is_none());
        Cuentas::desactivar(&raiz).unwrap();
        assert!(Cuentas::cargar(&raiz).unwrap().activa.is_none());
        std::fs::remove_file(raiz.join("accounts/locales.json")).unwrap();
        std::fs::remove_file(raiz.join("mundo-no-tocar")).unwrap();
        std::fs::remove_dir(raiz.join("accounts")).unwrap();
        std::fs::remove_dir(raiz).unwrap();
    }
    #[test]
    fn archivo_corrupto_no_se_sobrescribe_al_crear_una_cuenta() {
        let raiz = std::env::temp_dir().join(format!(
            "icaro-local-corrupto-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(raiz.join("accounts")).unwrap();
        let ruta = raiz.join("accounts/locales.json");
        std::fs::write(&ruta, b"contenido a recuperar").unwrap();
        assert!(Cuentas::activar(&raiz, "Jugador_1").is_err());
        assert_eq!(std::fs::read(&ruta).unwrap(), b"contenido a recuperar");
        std::fs::remove_file(ruta).unwrap();
        std::fs::remove_dir(raiz.join("accounts")).unwrap();
        std::fs::remove_dir(raiz).unwrap();
    }
}
