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
    fn instalador_recibe_parametros_silenciosos_y_se_verifica_el_ejecutable_resultante() {
        let ruta = std::env::temp_dir().join(format!("icaro-setup-test-{}", std::process::id()));
        fs::create_dir(&ruta).unwrap();
        let carpeta = Carpeta(ruta);
        let compilador = carpeta.0.join("compilar.ps1");
        fs::write(&compilador, r#"param([string]$Root)
$ErrorActionPreference = 'Stop'
Add-Type -OutputAssembly (Join-Path $Root 'setup.exe') -OutputType ConsoleApplication -TypeDefinition @'
using System;
using System.IO;
using System.Reflection;
[assembly: AssemblyInformationalVersion("0.2.1")]
public class Setup {
    public static int Main(string[] args) {
        string dir = null;
        foreach (var arg in args) if (arg.StartsWith("/DIR=")) dir = arg.Substring(5).Trim('"');
        if (dir == null) return 2;
        File.WriteAllLines(Path.Combine(dir, "argumentos.txt"), args);
        File.Copy(Assembly.GetExecutingAssembly().Location, Path.Combine(dir, "IcaroLauncher.exe"), true);
        return 0;
    }
}
'@
"#).unwrap();
        let compilar = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(&compilador)
            .arg("-Root")
            .arg(&carpeta.0)
            .creation_flags(0x08000000)
            .output()
            .unwrap();
        assert!(
            compilar.status.success(),
            "{}",
            String::from_utf8_lossy(&compilar.stderr)
        );
        let fuente = include_str!("../src/actualizacion.rs");
        let script = fuente
            .split_once("const SCRIPT: &str = r#\"")
            .unwrap()
            .1
            .split_once("\"#;")
            .unwrap()
            .0;
        let ayudante = carpeta.0.join("ayudante.ps1");
        fs::write(&ayudante, script).unwrap();
        let origen = carpeta.0.join("setup.exe");
        let destino = carpeta.0.join("IcaroLauncher.exe");
        let resultado = carpeta.0.join("resultado.json");
        let hash = format!("{:x}", Sha256::digest(fs::read(&origen).unwrap()));
        for (version, correcto) in [("0.2.1", true), ("0.9.9", false)] {
            let salida = Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                ])
                .arg(&ayudante)
                .args(["-ProcessId", "2147483647", "-Installer"])
                .arg("-Source")
                .arg(&origen)
                .arg("-Target")
                .arg(&destino)
                .arg("-Hash")
                .arg(&hash)
                .arg("-Outcome")
                .arg(&resultado)
                .arg("-Version")
                .arg(version)
                .args(["-BuildNumber", "42"])
                .creation_flags(0x08000000)
                .output()
                .unwrap();
            assert!(
                salida.status.success(),
                "{}",
                String::from_utf8_lossy(&salida.stderr)
            );
            let recibo: Resultado = serde_json::from_slice(&fs::read(&resultado).unwrap()).unwrap();
            assert_eq!(recibo.correcto, correcto, "{:?}", recibo.detalle);
            assert_eq!(recibo.detalle.is_some(), !correcto);
        }
        let argumentos = fs::read_to_string(carpeta.0.join("argumentos.txt")).unwrap();
        assert!(argumentos.lines().any(|a| a == "/VERYSILENT"));
        assert!(argumentos.lines().any(|a| a == "/NORESTARTAPPLICATIONS"));
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
        // Dos ventanas antiguas no pueden delegar dos instalaciones simultáneas.
        let mut espera = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 4",
            ])
            .creation_flags(0x08000000)
            .spawn()
            .unwrap();
        let argumentos = [
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ];
        let mut primero = Command::new("powershell.exe")
            .args(argumentos)
            .arg(&ayudante)
            .arg("-ProcessId")
            .arg(espera.id().to_string())
            .arg("-Source")
            .arg(&origen)
            .arg("-Target")
            .arg(&destino)
            .arg("-Hash")
            .arg(format!("{:x}", Sha256::digest(b"version nueva")))
            .arg("-Outcome")
            .arg(&resultado)
            .args(["-Version", "0.2.0", "-BuildNumber", "42"])
            .creation_flags(0x08000000)
            .spawn()
            .unwrap();
        let nombre = format!(
            "Local\\IcaroUpdate-{:X}",
            Sha256::digest(destino.to_string_lossy().to_lowercase().as_bytes())
        );
        let sondeo = carpeta.0.join("esperar-mutex.ps1");
        fs::write(&sondeo, "param([string]$Name)\nfor ($i=0; $i -lt 40; $i++) { try { $m=[Threading.Mutex]::OpenExisting($Name); $m.Dispose(); exit 0 } catch { Start-Sleep -Milliseconds 25 } }; exit 1").unwrap();
        assert!(Command::new("powershell.exe")
            .args(argumentos)
            .arg(&sondeo)
            .arg("-Name")
            .arg(nombre)
            .creation_flags(0x08000000)
            .status()
            .unwrap()
            .success());
        let segundo_resultado = carpeta.0.join("segundo.json");
        let segundo = Command::new("powershell.exe")
            .args(argumentos)
            .arg(&ayudante)
            .args(["-ProcessId", "2147483647"])
            .arg("-Source")
            .arg(&origen)
            .arg("-Target")
            .arg(&destino)
            .arg("-Hash")
            .arg("0".repeat(64))
            .arg("-Outcome")
            .arg(&segundo_resultado)
            .args(["-Version", "0.2.0", "-BuildNumber", "42"])
            .creation_flags(0x08000000)
            .output()
            .unwrap();
        assert!(segundo.status.success());
        assert!(
            !segundo_resultado.exists(),
            "El segundo ayudante debe salir antes de aplicar o reiniciar"
        );
        assert!(primero.wait().unwrap().success());
        let _ = espera.wait();
    }
}
