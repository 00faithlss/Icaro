// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

use serde::de::DeserializeOwned;
use std::time::Duration;
use zeroize::Zeroizing;

pub(crate) const USER_AGENT: &str = concat!(
    "00faithlss/Icaro/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/00faithlss/Icaro)"
);

/// Los errores conservan únicamente etapa y código HTTP; nunca cuerpos ni URLs.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("No se pudo conectar con {0}. Revisa tu conexión e inténtalo de nuevo.")]
    Network(&'static str),
    #[error("{service} respondió con HTTP {status}.")]
    Status { service: &'static str, status: u16 },
    #[error("{0} envió una respuesta inválida o demasiado grande.")]
    Response(&'static str),
    #[error("Demasiadas solicitudes. Espera {0} segundos y vuelve a intentarlo.")]
    RateLimited(u64),
}

pub(crate) fn client() -> Result<reqwest::Client, Error> {
    reqwest::Client::builder()
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .user_agent(USER_AGENT)
        .build()
        .map_err(|_| Error::Network("el servicio"))
}

pub(crate) async fn json<T: DeserializeOwned>(
    request: reqwest::RequestBuilder,
    service: &'static str,
    limit: usize,
) -> Result<T, Error> {
    let mut response = request.send().await.map_err(|_| Error::Network(service))?;
    if response.status().as_u16() == 429 {
        let delay = response
            .headers()
            .get("retry-after")
            .or_else(|| response.headers().get("x-ratelimit-reset"))
            .and_then(|s| s.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);
        return Err(Error::RateLimited(delay));
    }
    if !response.status().is_success() {
        return Err(Error::Status {
            service,
            status: response.status().as_u16(),
        });
    }
    if response.content_length().is_some_and(|n| n > limit as u64) {
        return Err(Error::Response(service));
    }
    let mut bytes = Zeroizing::new(Vec::new());
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| Error::Network(service))?
    {
        if bytes.len().saturating_add(chunk.len()) > limit {
            return Err(Error::Response(service));
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| Error::Response(service))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    async fn response(reply: &str, limit: usize) -> Result<serde_json::Value, Error> {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let reply = reply.to_owned();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0u8; 4096];
            stream.read(&mut request).await.unwrap();
            stream.write_all(reply.as_bytes()).await.unwrap();
        });
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let result = json(client.get(format!("http://{address}/")), "Prueba", limit).await;
        server.await.unwrap();
        result
    }
    #[tokio::test]
    async fn errors_do_not_expose_remote_secrets_and_redirects_are_rejected() {
        for (status, extra) in [
            (401, ""),
            (
                302,
                "Location: https://attacker.example/?token=CANARY_SECRET\r\n",
            ),
        ] {
            let reply = format!("HTTP/1.1 {status} Error\r\n{extra}Content-Length: 13\r\nConnection: close\r\n\r\nCANARY_SECRET");
            let error = response(&reply, 1024).await.unwrap_err();
            assert!(!format!("{error:?} {error}").contains("CANARY_SECRET"));
            assert!(matches!(error, Error::Status { status: code, .. } if code == status));
        }
    }
    #[tokio::test]
    async fn caps_chunked_bodies_and_rejects_bad_json() {
        let body = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n4\r\n1234\r\n4\r\n5678\r\n0\r\n\r\n";
        assert!(matches!(response(body, 6).await, Err(Error::Response(_))));
        assert!(matches!(
            response(
                "HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nSECRET",
                1024
            )
            .await,
            Err(Error::Response(_))
        ));
    }
    #[tokio::test]
    async fn observes_retry_after_without_exposing_error_body() {
        assert_eq!(response("HTTP/1.1 429 Too Many Requests\r\nRetry-After: 123\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", 1024).await, Err(Error::RateLimited(123)));
    }
}
