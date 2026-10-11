// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss
//! Capturas de widgets reales con fixtures; sin login/red/guardado de usuario.
use crate::*;
pub fn iniciar(piedra: bool) -> (App, Task<Mensaje>) {
    let (mut a, _) = iniciar_con_guardado(
        Seccion::Instancias,
        piedra,
        None,
        false,
        servicios::Estado::nuevo(true),
        None,
    );
    a.reducir = true;
    a.pantalla = None;
    a.umbral = None;
    let scene = std::env::var("ICARO_VISTA_DISENO").unwrap_or_else(|_| "asistente".into());
    match scene.as_str() {
        "asistente" => {
            a.asistente = Some(pantallas::Asistente {
                paso: 1,
                tipo_version: 0,
                busqueda: String::new(),
                version: 0,
                loader: 1,
                nombre: String::new(),
                portada: 0,
            })
        }
        "exportar" => {
            a.exportar = Some(pantallas::Exportar {
                instancia: 0,
                nombre: "Supervivencia".into(),
                carpetas: [true, true, false, false],
                formato: 0,
                destino: "C:\\Users\\Mineral\\Descargas".into(),
                medidas: Some(icaro_core::exportar::Medidas {
                    bytes: [
                        312 * 1048576,
                        1_600_000_000,
                        48 * 1024,
                        412 * 1048576,
                        22 * 1048576,
                    ],
                    mundos: 3,
                    capturas: 41,
                }),
                error: None,
                guardando: false,
            })
        }
        "archivos" | "archivos-renombrar" => {
            a.editor = Some(7);
            a.editor_inst = 0;
            a.exp = Explorador::abrir("qa-diseno");
            a.exp.elegir(1);
            if scene == "archivos-renombrar" {
                a.exp.iniciar_renombrar();
            }
        }
        _ => preparar_mods(&mut a, &scene),
    }
    (
        a,
        Task::perform(
            async {
                tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            },
            |_| Mensaje::CapturarDiseno,
        ),
    )
}
fn preparar_mods(a: &mut App, scene: &str) {
    use icaro_core::{
        instancias::Instancia,
        modrinth::{Hit, Project, Version},
        mods::Instalado,
    };
    a.editor = Some(2);
    a.editor_inst = 0;
    a.servicios.demo = false;
    a.servicios.restoring = false;
    a.servicios.instancias.almacen = icaro_core::instancias::Almacen::nuevo(
        std::env::temp_dir().join("icaro-design-fixture-instances"),
    );
    let instance = Instancia {
        id: "00000000000000000000000000000001".into(),
        nombre: "Supervivencia".into(),
        minecraft: "1.21.4".into(),
        loader: "fabric".into(),
        portada: 0,
        creada: 0,
    };
    a.servicios.instancias.registros = vec![instance.clone()];
    if let Some(card) = crate::lista_instancias().lock().unwrap().first_mut() {
        card.mods = "4 mods".into();
    }
    a.servicios
        .mods
        .tamanos
        .insert(instance.id.clone(), 312 * 1048576);
    let mut hits = vec![];
    let mut installed = vec![];
    let mut versions = vec![];
    for (id, title, author, description, v, downloads) in [
        (
            "sodium",
            "Sodium",
            "CaffeineMC",
            "Render más rápido sin cambiar el aspecto del juego.",
            "0.6.5",
            78_412_330,
        ),
        (
            "lithium",
            "Lithium",
            "CaffeineMC",
            "Optimiza la lógica del mundo sin cambiar mecánicas.",
            "0.14.3",
            39_880_551,
        ),
        (
            "iris",
            "Iris Shaders",
            "IMS",
            "Shaders compatibles con Sodium.",
            "1.8.1",
            12_000_000,
        ),
        (
            "replay",
            "Replay Mod",
            "john0702",
            "Graba y reproduce partidas.",
            "2.6.20",
            9_000_000,
        ),
    ] {
        let p:Project=serde_json::from_value(serde_json::json!({"id":id,"slug":id,"title":title,"description":description,"body":"## Rendimiento\n\nRender más rápido sin cambiar el aspecto del juego.\n\n### Características\n\n- Compatibilidad con tu instancia.\n- Opciones de calidad y rendimiento.","license":{"id":"MIT","name":"MIT"},"downloads":downloads,"followers":41288,"game_versions":["1.21.4","1.21.3","1.21.1"],"loaders":["fabric","quilt","neoforge"],"categories":["optimization"],"published":"2021-08-20T00:00:00Z","updated":"2026-10-06T00:00:00Z","client_side":"required","server_side":"unsupported","source_url":"https://github.com/CaffeineMC/sodium","issues_url":"https://github.com/CaffeineMC/sodium/issues"})).unwrap();
        let version:Version=serde_json::from_value(serde_json::json!({"id":format!("{id}065"),"project_id":id,"name":v,"version_number":v,"version_type":"release","game_versions":["1.21.4"],"loaders":["fabric"],"downloads":2_110_000,"date_published":"2026-10-06T00:00:00Z","files":[],"dependencies":[],"changelog":"## Cambios\n\n- Corrección de errores."})).unwrap();
        hits.push(Hit {
            project_id: id.into(),
            slug: id.into(),
            title: title.into(),
            description: description.into(),
            author: author.into(),
            downloads,
            icon_url: None,
            categories: vec!["optimization".into()],
            versions: vec!["1.21.4".into()],
        });
        installed.push(Instalado {
            project_id: id.into(),
            nombre: title.into(),
            version_id: version.id.clone(),
            version: v.into(),
            publicada: version.date_published.clone(),
            archivo: format!("{id}.jar"),
            sha512: String::new(),
            activo: id != "replay",
            dependencias: vec![],
        });
        a.servicios
            .mods
            .diseno
            .proyectos
            .insert(id.into(), (p, author.into()));
        versions.push(version);
    }
    if matches!(scene, "mods" | "seleccion") {
        installed[2].dependencias=serde_json::from_value(serde_json::json!([{"project_id":"sodium","version_id":null,"dependency_type":"incompatible"}])).unwrap();
    }
    let mut newer = versions[1].clone();
    newer.id = "lithiumnew".into();
    newer.version_number = "0.14.5".into();
    a.servicios
        .mods
        .updates
        .insert(instance.id.clone(), vec![newer]);
    a.servicios
        .mods
        .listas
        .insert(instance.id.clone(), installed);
    if scene == "seleccion" {
        a.servicios.mods.diseno.seleccion.insert(
            instance.id.clone(),
            std::collections::BTreeSet::from(["sodium.jar".into(), "lithium.jar".into()]),
        );
    }
    if scene == "tienda" || scene == "ficha" || scene == "pagina" {
        a.mods_vista = 1;
        a.servicios.order = icaro_core::modrinth::Order::Downloads;
        a.servicios.results = Some(icaro_core::modrinth::Results {
            hits,
            offset: 0,
            limit: 20,
            total_hits: 1284,
        });
        let v = versions[0].clone();
        a.servicios.mods.diseno.cestas.insert(
            instance.id.clone(),
            crate::diseno_mods::Cesta::para_prueba(v.clone(), "Sodium".into()),
        );
        a.servicios
            .mods
            .listas
            .get_mut(&instance.id)
            .unwrap()
            .retain(|m| m.project_id != "sodium");
        if scene != "tienda" {
            let mut p = a.servicios.mods.diseno.proyectos["sodium"].0.clone();
            for (n, file) in ["piranesi-arco.webp", "faeton.webp", "tivoli.webp"]
                .iter()
                .enumerate()
            {
                let url = format!("https://cdn.modrinth.com/data/fixture/images/{n}.webp");
                if let Ok(bytes) = std::fs::read(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("assets/originales")
                        .join(file),
                ) {
                    a.servicios
                        .images
                        .recibir(url.clone(), catalogo::decodificar(bytes));
                }
                p.gallery.push(
                    serde_json::from_value(
                        serde_json::json!({"url":url,"featured":n==0,"title":file}),
                    )
                    .unwrap(),
                );
            }
            a.servicios.ficha = ficha::Estado::nuevo(None, &versions[..1]);
            a.servicios.ficha.minecraft = "1.21.4".into();
            a.servicios.ficha.loader = "Fabric".into();
            a.servicios.ficha.miembros=serde_json::from_value(serde_json::json!([{"user":{"id":"caffeine","username":"CaffeineMC","name":"CaffeineMC"},"role":"Owner","is_owner":true}])).unwrap();
            a.servicios.document = catalogo::Documento::parse(&p.body);
            let mut vv = versions[..1].to_vec();
            for n in 0..7 {
                let mut old = v.clone();
                old.id = format!("sodiumold{n}");
                old.version_number = format!("0.6.{}", 4_usize.saturating_sub(n));
                vv.push(old);
            }
            vv[0].dependencies=serde_json::from_value(serde_json::json!([{"project_id":"fabric-api","version_id":null,"dependency_type":"required"},{"project_id":"sodium-extra","version_id":null,"dependency_type":"optional"}])).unwrap();
            a.servicios.dependencias.insert(
                "p:fabric-api".into(),
                Ok(("fabric-api".into(), "Fabric API".into())),
            );
            a.servicios.dependencias.insert(
                "p:sodium-extra".into(),
                Ok(("sodium-extra".into(), "Sodium Extra".into())),
            );
            a.servicios.detail = Some((p, vv));
            if scene == "ficha" {
                a.mod_panel = Some("sodium".into());
            } else {
                a.servicios.ficha.tab = ficha::Pestana::Versiones;
            }
        }
    }
}
