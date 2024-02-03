fn main() {
    capnpc::CompilerCommand::new()
        .file("schema/app.capnp")
        .run()
        .expect("schema compiler command");

    tauri_build::build()
}
