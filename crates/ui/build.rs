fn main() {
    println!("cargo:rerun-if-changed=assets/icono/icaro.ico");
    println!("cargo:rerun-if-changed=assets/icono/icaro.rc");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let _ = embed_resource::compile_for_examples("assets/icono/icaro.rc", embed_resource::NONE)
            .manifest_optional();
    }
}
