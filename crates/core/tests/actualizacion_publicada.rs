// SPDX-License-Identifier: GPL-3.0-only
//! Comprueba el contrato público de una release; no descarga ni aplica ejecutables.

#[tokio::test]
#[ignore = "Consulta GitHub; requiere ICARO_QA_VERSION_ACTUAL e ICARO_QA_VERSION_ESPERADA"]
async fn release_publicada_se_ofrece_a_la_version_anterior() {
    let actual =
        std::env::var("ICARO_QA_VERSION_ACTUAL").expect("Falta la versión instalada de prueba");
    let esperada =
        std::env::var("ICARO_QA_VERSION_ESPERADA").expect("Falta la versión publicada esperada");
    let disponible = icaro_core::actualizacion::comprobar_compilacion(&actual, 0)
        .await
        .expect("La release pública debe superar las validaciones del actualizador")
        .expect("Debe existir una actualización para la versión anterior");
    assert_eq!(disponible.version, esperada);
    assert!(disponible.compilacion > 0);
    assert!(disponible.bytes > 0);
    assert!(!disponible.notas.is_empty());
    println!(
        "{} -> {} (compilación {}): {} notas, {} bytes; sin descargar ni instalar.",
        actual,
        disponible.version,
        disponible.compilacion,
        disponible.notas.len(),
        disponible.bytes
    );
}
