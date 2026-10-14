// SPDX-License-Identifier: GPL-3.0-only
//! Consulta y pertenencia a redes ZeroTier mediante su CLI oficial.
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::io::AsyncReadExt;
use tokio_util::sync::CancellationToken;
#[derive(Debug, Clone)]
pub struct Red {
    pub id: String,
    pub nombre: String,
    pub estado: String,
    pub ips: Vec<String>,
}
#[derive(Debug, Clone)]
pub enum Estado {
    Ausente,
    Disponible {
        version: String,
        online: bool,
        redes: Vec<Red>,
    },
    SinAcceso,
}
fn candidatos() -> Vec<(PathBuf, bool)> {
    let mut v = vec![];
    if let Some(paths) = std::env::var_os("PATH") {
        for p in std::env::split_paths(&paths).take(128) {
            if p.is_absolute() {
                v.push((
                    p.join(if cfg!(windows) {
                        "zerotier-cli.exe"
                    } else {
                        "zerotier-cli"
                    }),
                    false,
                ));
            }
        }
    }
    #[cfg(windows)]
    for key in ["ProgramData", "ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(p) = std::env::var_os(key) {
            let base = PathBuf::from(p);
            v.push((base.join("ZeroTier/One/zerotier-one_x64.exe"), true));
            v.push((base.join("ZeroTier/One/zerotier-one_x86.exe"), true));
        }
    }
    #[cfg(not(windows))]
    for p in [
        "/usr/sbin/zerotier-cli",
        "/usr/bin/zerotier-cli",
        "/usr/local/bin/zerotier-cli",
    ] {
        v.push((p.into(), false));
    }
    v
}
async fn comando(path: &PathBuf, one: bool, args: &[&str]) -> Result<Vec<u8>, ()> {
    let mut cmd = tokio::process::Command::new(path);
    if one {
        cmd.arg("-q");
    }
    cmd.arg("-j")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    let mut child = cmd.spawn().map_err(|_| ())?;
    let mut out = child.stdout.take().ok_or(())?.take(256 * 1024 + 1);
    let mut bytes = vec![];
    out.read_to_end(&mut bytes).await.map_err(|_| ())?;
    if bytes.len() > 256 * 1024 || !child.wait().await.map_err(|_| ())?.success() {
        return Err(());
    }
    Ok(bytes)
}
fn analizar(info: &[u8], nets: &[u8]) -> Result<Estado, ()> {
    let v: serde_json::Value = serde_json::from_slice(info).map_err(|_| ())?;
    let version = v["version"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() < 80 && s.chars().all(|c| !c.is_control()))
        .ok_or(())?
        .to_owned();
    let online = v["online"].as_bool().ok_or(())?;
    let array: Vec<serde_json::Value> = serde_json::from_slice(nets).map_err(|_| ())?;
    if array.len() > 256 {
        return Err(());
    }
    let mut redes = vec![];
    for n in array {
        let id = n["nwid"]
            .as_str()
            .or(n["id"].as_str())
            .filter(|s| s.len() == 16 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or(())?
            .to_owned();
        let text = |key: &str| max_text(n[key].as_str().unwrap_or(""));
        let ips = n["assignedAddresses"]
            .as_array()
            .map(|a| {
                a.iter()
                    .take(32)
                    .filter_map(|s| s.as_str())
                    .map(max_text)
                    .collect()
            })
            .unwrap_or_default();
        redes.push(Red {
            id,
            nombre: text("name"),
            estado: text("status"),
            ips,
        });
    }
    Ok(Estado::Disponible {
        version,
        online,
        redes,
    })
}
fn max_text(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).take(160).collect()
}
#[derive(Debug, Clone, Copy)]
pub enum Accion {
    Unirse,
    Salir,
}
pub fn id_red(id: &str) -> Result<String, String> {
    let id = id.trim();
    if id.len() != 16 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("El ID de red debe tener 16 caracteres hexadecimales.".into());
    }
    Ok(id.to_ascii_lowercase())
}
pub async fn cambiar_red(id: &str, a: Accion, cancel: CancellationToken) -> Result<Estado, String> {
    let id = id_red(id)?;
    if cancel.is_cancelled() {
        return Err("La solicitud fue cancelada antes de enviarse.".into());
    }
    let Some((p, one)) = candidatos().into_iter().find(|(p, _)| p.is_file()) else {
        return Err("ZeroTier no está instalado.".into());
    };
    let command = match a {
        Accion::Unirse => "join",
        Accion::Salir => "leave",
    };
    let args = [command, id.as_str()];
    tokio::select! {
        _=cancel.cancelled()=>return Err("La solicitud se interrumpió. Actualiza el estado para comprobar la pertenencia a la red.".into()),
        r=tokio::time::timeout(Duration::from_secs(8),comando(&p,one,&args))=>{
            r.map_err(|_|"El servicio de ZeroTier no respondió. Actualiza el estado antes de reintentar.".to_string())?
                .map_err(|_|"No se pudo modificar la red. Comprueba el servicio y los permisos de ZeroTier.".to_string())?;
        }
    }
    Ok(detectar().await)
}
pub async fn detectar() -> Estado {
    let Some((p, one)) = candidatos().into_iter().find(|(p, _)| p.is_file()) else {
        return Estado::Ausente;
    };
    tokio::time::timeout(Duration::from_secs(5), async {
        let info = comando(&p, one, &["info"]).await?;
        let redes = comando(&p, one, &["listnetworks"]).await?;
        analizar(&info, &redes)
    })
    .await
    .ok()
    .and_then(Result::ok)
    .unwrap_or(Estado::SinAcceso)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn id_de_red_no_admite_comandos_ni_valores_ambiguos() {
        assert_eq!(id_red(" ABCDEF0123456789 ").unwrap(), "abcdef0123456789");
        for value in [
            "",
            "123",
            "abcdef012345678g",
            "abcdef0123456789 --help",
            "../../auth",
        ] {
            assert!(id_red(value).is_err());
        }
    }
    #[tokio::test]
    async fn cancelar_antes_de_unirse_no_invoca_el_servicio() {
        let c = CancellationToken::new();
        c.cancel();
        assert!(cambiar_red("abcdef0123456789", Accion::Unirse, c)
            .await
            .unwrap_err()
            .contains("antes"));
    }
    #[test]
    fn datos_oficiales_no_inventan_estado_ni_autorizacion() {
        let s=analizar(br#"{"version":"1.16.0","online":true}"#,br#"[{"nwid":"1234567890abcdef","name":"Amigos","status":"ACCESS_DENIED","assignedAddresses":[]}]"#).unwrap();
        let Estado::Disponible { redes, .. } = s else {
            panic!()
        };
        assert_eq!(redes[0].estado, "ACCESS_DENIED");
        assert!(redes[0].ips.is_empty());
        assert!(analizar(br#"{"version":"1"}"#, b"[]").is_err());
    }
}
