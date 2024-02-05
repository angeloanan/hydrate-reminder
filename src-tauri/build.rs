fn main() {
    capnpc::CompilerCommand::new()
        .file("schema/app.capnp")
        .run()
        .expect("schema compiler command");

    tauri_build::build();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=schema/app.capnp");
}
