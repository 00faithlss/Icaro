// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

use icaro_core::auth::{begin_device, CancellationToken};

#[tokio::test]
#[ignore = "Solicita un código real a Microsoft; no abre el navegador ni autoriza cuentas"]
async fn microsoft_emite_codigo_para_icaro() {
    let grant = begin_device(CancellationToken::new())
        .await
        .expect("Microsoft debe aceptar la configuración de cliente público de Ícaro");
    assert!(!grant.info().code().is_empty());
    assert!(!grant.info().expired());
    assert!(grant.info().page().starts_with("https://"));
    // No se imprime el código ni se intercambia por una sesión.
}
