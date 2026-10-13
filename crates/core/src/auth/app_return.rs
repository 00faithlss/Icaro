// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! El enlace nativo entrega la respuesta al receptor temporal del intento original.
//! No persiste credenciales ni conoce el verificador PKCE del proceso principal.

use super::{Error, Url};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::time::Duration;
use zeroize::Zeroizing;

pub const APP_REDIRECT: &str = "icaro://oauth/microsoft";

pub fn register_app_return() -> Result<(), Error> {
    #[cfg(windows)]
    {
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};
        let executable = std::env::current_exe().map_err(|_| Error::Setup)?;
        let path = executable.to_str().ok_or(Error::Setup)?;
        if path.contains('"') || path.contains(['\r', '\n']) {
            return Err(Error::Setup);
        }
        let user = RegKey::predef(HKEY_CURRENT_USER);
        let (protocol, _) = user
            .create_subkey("Software\\Classes\\icaro")
            .map_err(|_| Error::Setup)?;
        protocol
            .set_value("", &"URL:Ícaro")
            .map_err(|_| Error::Setup)?;
        protocol
            .set_value("URL Protocol", &"")
            .map_err(|_| Error::Setup)?;
        let (icon, _) = protocol
            .create_subkey("DefaultIcon")
            .map_err(|_| Error::Setup)?;
        icon.set_value("", &format!("\"{path}\",0"))
            .map_err(|_| Error::Setup)?;
        let (command, _) = protocol
            .create_subkey("shell\\open\\command")
            .map_err(|_| Error::Setup)?;
        command
            .set_value("", &format!("\"{path}\" --oauth-return \"%1\""))
            .map_err(|_| Error::Setup)?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Err(Error::Setup)
    }
}

fn request(value: &str) -> Result<(u16, Zeroizing<String>), Error> {
    if value.len() > 8192 || value.contains(['\r', '\n', '\0']) {
        return Err(Error::Callback);
    }
    let url = Url::parse(value).map_err(|_| Error::Callback)?;
    if url.scheme() != "icaro"
        || url.host_str() != Some("oauth")
        || url.path() != "/microsoft"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.fragment().is_some()
    {
        return Err(Error::Callback);
    }
    let params: Vec<_> = url.query_pairs().collect();
    let values = |key| {
        params
            .iter()
            .filter(|(k, _)| k == key)
            .map(|(_, v)| v.as_ref())
            .collect::<Vec<_>>()
    };
    let states = values("state");
    let [state] = states.as_slice() else {
        return Err(Error::Callback);
    };
    let (port, random) = state.split_once('.').ok_or(Error::Callback)?;
    let port: u16 = port.parse().map_err(|_| Error::Callback)?;
    if port == 0
        || random.len() != 43
        || !random
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(Error::Callback);
    }
    let code = values("code");
    let error = values("error");
    let valid = matches!(code.as_slice(), [v] if !v.is_empty() && v.len() <= 4096)
        && error.is_empty()
        || matches!(error.as_slice(), [v] if !v.is_empty() && v.len() <= 128) && code.is_empty();
    if !valid {
        return Err(Error::Callback);
    }
    let query = url.query().ok_or(Error::Callback)?;
    Ok((
        port,
        Zeroizing::new(format!(
            "GET /?{query} HTTP/1.1\r\nHost: localhost:{port}\r\n\r\n"
        )),
    ))
}

/// Se ejecuta antes de iniciar la UI del proceso activado por Windows.
/// Solo el receptor original puede aceptar el state y canjear el código con PKCE.
pub fn forward_app_return(value: &str) -> Result<(), Error> {
    let (port, request) = request(value)?;
    let deadline = Duration::from_secs(3);
    let mut socket = TcpStream::connect_timeout(
        &SocketAddrV4::new(Ipv4Addr::LOCALHOST, port).into(),
        deadline,
    )
    .map_err(|_| Error::Callback)?;
    socket
        .set_read_timeout(Some(deadline))
        .map_err(|_| Error::Callback)?;
    socket
        .set_write_timeout(Some(deadline))
        .map_err(|_| Error::Callback)?;
    socket
        .write_all(request.as_bytes())
        .map_err(|_| Error::Callback)?;
    let mut reply = [0; 13];
    socket.read_exact(&mut reply).map_err(|_| Error::Callback)?;
    if &reply != b"HTTP/1.1 200 " {
        return Err(Error::Callback);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const STATE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopq";

    #[test]
    fn app_return_is_forwarded_only_to_loopback_with_the_original_state() {
        let (port, value) = request(&format!(
            "{APP_REDIRECT}?state=4321.{STATE}&code=CANARY_CODE"
        ))
        .unwrap();
        assert_eq!(port, 4321);
        assert!(value.starts_with("GET /?state=4321."));
        assert!(value.ends_with("Host: localhost:4321\r\n\r\n"));
    }
    #[test]
    fn rejects_foreign_routes_duplicates_and_malformed_states() {
        for url in [
            format!("https://oauth/microsoft?state=4321.{STATE}&code=x"),
            format!("icaro://attacker.test/microsoft?state=4321.{STATE}&code=x"),
            format!("icaro://user@oauth/microsoft?state=4321.{STATE}&code=x"),
            format!("icaro://oauth/microsoft?state=0.{STATE}&code=x"),
            format!("icaro://oauth/microsoft?state=65536.{STATE}&code=x"),
            format!("icaro://oauth/microsoft?state=4321.{STATE}&code=x&code=y"),
            format!("icaro://oauth/microsoft?state=4321.{STATE}&code=x&error=denied"),
            format!("icaro://oauth/microsoft?state=4321.{STATE}&state=4321.{STATE}&code=x"),
            format!("icaro://oauth/microsoft?state=4321.{STATE}&code=x#fragment"),
            "icaro://oauth/microsoft?state=4321.short&code=x".into(),
        ] {
            assert!(matches!(request(&url), Err(Error::Callback)));
        }
    }
    #[tokio::test]
    async fn forwarded_code_is_accepted_only_by_the_matching_pending_attempt() {
        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        let proof = super::super::Proof {
            state: Zeroizing::new(format!("{port}.{STATE}")),
            verifier: Zeroizing::new("PRIVATE_VERIFIER".into()),
        };
        let received =
            tokio::spawn(async move { super::super::receive(&listener, &proof, port).await });
        let wrong = format!("{APP_REDIRECT}?state={port}.{}&code=WRONG", "X".repeat(43));
        assert_eq!(
            tokio::task::spawn_blocking(move || forward_app_return(&wrong))
                .await
                .unwrap(),
            Err(Error::Callback)
        );
        let valid = format!("{APP_REDIRECT}?state={port}.{STATE}&code=CANARY_CODE");
        let forwarded = tokio::task::spawn_blocking(move || forward_app_return(&valid))
            .await
            .unwrap();
        let received = tokio::time::timeout(Duration::from_secs(5), received)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(&*received.unwrap(), "CANARY_CODE");
        assert_eq!(forwarded, Ok(()));
    }
}
