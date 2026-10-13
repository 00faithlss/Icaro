// SPDX-License-Identifier: GPL-3.0-only
//! Consulta acotada de ZeroTier mediante su CLI oficial, sin modificar redes.
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::io::AsyncReadExt;
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
