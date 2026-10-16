use sha2::{Digest, Sha256, Sha512};
use std::{hint::black_box, io::Read, time::Instant};

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_some_and(|v| v == "sha256") {
        let datos = vec![0x5au8; 32 * 1024 * 1024];
        for muestra in 0..3 {
            let t = Instant::now();
            let mut hash = Sha256::new();
            for bloque in datos.chunks(64 * 1024) {
                hash.update(black_box(bloque));
            }
            let checksum = black_box(hash.finalize());
            let segundos = t.elapsed().as_secs_f64();
            println!("{{\"debug\":{},\"muestra\":{muestra},\"bytes\":{},\"seconds\":{segundos},\"MiB_s\":{},\"checksum\":\"{:x}\"}}", cfg!(debug_assertions), datos.len(), 32.0/segundos, checksum);
        }
        return;
    }
    if args.get(1).is_some_and(|v| v == "io") {
        medir_io(std::path::Path::new(&args[2]));
        return;
    }
    let cantidad = 32 * 1024 * 1024;
    let datos = vec![0x5au8; cantidad];
    for muestra in 0..3 {
        let t = Instant::now();
        let mut entrada = std::io::Cursor::new(&datos);
        let mut hash = Sha512::new();
        let mut buf = [0; 64 * 1024];
        loop {
            let n = entrada.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            hash.update(black_box(&buf[..n]));
        }
        let digest = black_box(hash.finalize());
        let segundos = t.elapsed().as_secs_f64();
        println!("{{\"debug\":{},\"muestra\":{muestra},\"bytes\":{cantidad},\"seconds\":{segundos},\"MiB_s\":{},\"checksum\":\"{:x}\"}}", cfg!(debug_assertions), 32.0 / segundos, digest);
    }
}

fn medir_io(origen: &std::path::Path) {
    use std::fs;
    let propuesta = std::env::current_dir()
        .unwrap()
        .join("target/investigacion-rendimiento");
    fs::create_dir_all(&propuesta).unwrap();
    let raiz = propuesta.canonicalize().unwrap();
    assert!(raiz.starts_with(
        std::env::current_dir()
            .unwrap()
            .canonicalize()
            .unwrap()
            .join("target")
    ));
    let banco = raiz.join(format!("io-{}", std::process::id()));
    assert_eq!(banco.parent(), Some(raiz.as_path()));
    fs::create_dir(&banco).unwrap();
    let copia = banco.join("copia");
    let enlaces = banco.join("enlaces");
    let respaldo = banco.join("respaldo");
    for p in [&copia, &enlaces, &respaldo] {
        fs::create_dir(p).unwrap();
    }
    let archivos: Vec<_> = fs::read_dir(origen)
        .unwrap()
        .map(Result::unwrap)
        .map(|e| e.path())
        .filter(|p| fs::symlink_metadata(p).unwrap().is_file())
        .collect();
    let muestra = || {
        archivos
            .iter()
            .map(|p| {
                let m = fs::metadata(p).unwrap();
                (m.len(), m.modified().unwrap())
            })
            .collect::<Vec<_>>()
    };
    let inicial = muestra();
    let bytes: u64 = inicial.iter().map(|m| m.0).sum();
    let t = Instant::now();
    for _ in 0..100 {
        black_box(muestra());
    }
    println!(
        "{{\"operation\":\"metadata_snapshot\",\"files\":{},\"seconds\":{}}}",
        archivos.len(),
        t.elapsed().as_secs_f64() / 100.0
    );
    let t = Instant::now();
    for p in &archivos {
        fs::copy(p, copia.join(p.file_name().unwrap())).unwrap();
    }
    println!(
        "{{\"operation\":\"copy_all\",\"bytes\":{bytes},\"seconds\":{}}}",
        t.elapsed().as_secs_f64()
    );
    let t = Instant::now();
    for p in &archivos {
        let mut file = fs::File::open(p).unwrap();
        let mut h = Sha512::new();
        let mut buf = [0; 64 * 1024];
        loop {
            let n = file.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            h.update(&buf[..n]);
        }
        black_box(h.finalize());
    }
    println!(
        "{{\"operation\":\"hash_all_release\",\"bytes\":{bytes},\"seconds\":{}}}",
        t.elapsed().as_secs_f64()
    );
    let t = Instant::now();
    for p in &archivos {
        let n = p.file_name().unwrap();
        fs::hard_link(copia.join(n), enlaces.join(n)).unwrap();
    }
    println!(
        "{{\"operation\":\"hardlink_sandbox\",\"files\":{},\"seconds\":{}}}",
        archivos.len(),
        t.elapsed().as_secs_f64()
    );
    let t = Instant::now();
    for p in archivos.iter().take(5) {
        let n = p.file_name().unwrap();
        fs::rename(copia.join(n), respaldo.join(n)).unwrap();
    }
    println!(
        "{{\"operation\":\"move_five_sandbox\",\"seconds\":{}}}",
        t.elapsed().as_secs_f64()
    );
    assert_eq!(inicial, muestra(), "El corpus cambió durante la medición");
    // Limpieza no recursiva, limitada a los archivos creados por este proceso.
    for dir in [&copia, &enlaces, &respaldo] {
        for p in fs::read_dir(dir).unwrap() {
            fs::remove_file(p.unwrap().path()).unwrap();
        }
        fs::remove_dir(dir).unwrap();
    }
    fs::remove_dir(banco).unwrap();
}
