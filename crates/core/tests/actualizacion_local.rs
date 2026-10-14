// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Regresión del ayudante real con Windows PowerShell, sin red ni instalación de usuario.
#[cfg(windows)]
mod windows {
    use icaro_core::actualizacion::Resultado;
    use sha2::{Digest, Sha256};
    use std::{fs, os::windows::process::CommandExt, path::PathBuf, process::Command};

    struct Carpeta(PathBuf);
    impl Drop for Carpeta {
        fn drop(&mut self) {
            // Solo archivos dentro de la carpeta creada por esta prueba, sin recorrido recursivo.
            if let Ok(archivos) = fs::read_dir(&self.0) {
                for archivo in archivos.flatten() {
                    if archivo.path().parent() == Some(self.0.as_path()) {
                        let _ = fs::remove_file(archivo.path());
                    }
                }
            }
            let _ = fs::remove_dir(&self.0);
        }
    }

    #[test]
    fn powershell_reintenta_el_portable_y_sustituye_el_resultado_del_fallo() {
        let mut id = [0u8; 16];
        getrandom::fill(&mut id).unwrap();
        let id: String = id.iter().map(|b| format!("{b:02x}")).collect();
        let temporal = std::env::temp_dir().canonicalize().unwrap();
        let ruta = temporal.join(format!("icaro-update-ícaro-{id}"));
        assert_eq!(ruta.parent(), Some(temporal.as_path()));
        fs::create_dir(&ruta).unwrap();
        let carpeta = Carpeta(ruta);
        let fuente = include_str!("../src/actualizacion.rs");
        let script = fuente.split_once("const SCRIPT: &str = r#\"").unwrap().1.split_once("\"#;").unwrap().0;
        let ayudante = carpeta.0.join("ayudante real.ps1");
        fs::write(&ayudante, script).unwrap();
        let origen = carpeta.0.join("nuevo portable.exe");
        let destino = carpeta.0.join("Ícaro Launcher.exe");
        let resultado = carpeta.0.join("resultado.json");
        fs::write(&origen, b"version nueva").unwrap();
        fs::write(&destino, b"version anterior").unwrap();
        let hash = format!("{:x}", Sha256::digest(b"version nueva"));
        for (esperado, correcto) in [("0".repeat(64), false), (hash, true)] {
            let mut espera = Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command", "Start-Sleep -Milliseconds 200"])
                .creation_flags(0x08000000).spawn().unwrap();
            let salida = Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"])
                .arg(&ayudante).arg("-ProcessId").arg(espera.id().to_string())
                .arg("-Source").arg(&origen).arg("-Target").arg(&destino)
                .arg("-Hash").arg(esperado).arg("-Outcome").arg(&resultado)
                .args(["-Version", "0.2.0", "-BuildNumber", "42"])
                .creation_flags(0x08000000).output().unwrap();
            let _ = espera.wait();
            assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
            let recibo: Resultado = serde_json::from_slice(&fs::read(&resultado).unwrap()).unwrap();
            assert_eq!(recibo.correcto, correcto);
            assert_eq!((recibo.version.as_str(), recibo.compilacion), ("0.2.0", 42));
            assert_eq!(fs::read(&destino).unwrap(), if correcto { b"version nueva".as_slice() } else { b"version anterior" });
        }
    }
}
