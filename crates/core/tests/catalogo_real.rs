// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

use icaro_core::modrinth::{Catalog, Kind, Search};

#[tokio::test]
#[ignore = "Consulta de lectura a la API pública de Modrinth; requiere red"]
async fn consulta_catalogo_proyecto_y_versiones_compatibles() {
    let catalog = Catalog::new().unwrap();
    let results = catalog
        .search(&Search {
            query: "sodium".into(),
            kind: Kind::Mod,
            minecraft: Some("1.21.4".into()),
            loader: Some("fabric".into()),
            offset: 0,
        })
        .await
        .unwrap();
    let hit = results
        .hits
        .iter()
        .find(|hit| hit.slug == "sodium")
        .expect("Sodium en el catálogo público");
    let project = catalog.project(&hit.project_id).await.unwrap();
    assert_eq!(project.id, hit.project_id);
    let versions = catalog
        .versions(&project.id, Some("1.21.4"), Some("fabric"))
        .await
        .unwrap();
    assert!(!versions.is_empty());
    assert!(versions.iter().all(
        |version| version.game_versions.iter().any(|v| v == "1.21.4")
            && version.loaders.iter().any(|l| l == "fabric")
    ));
    assert!(versions.iter().all(|version| version
        .files
        .iter()
        .any(|f| f.primary && f.hashes.sha512.len() == 128)));
}
