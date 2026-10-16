//! Read-only validation of an explicitly supplied instance library.
fn main() {
    let path = std::env::args_os().nth(1).expect("usage: check_library <library.json>");
    match launcher_core::Store::new(path.into()).load() {
        Ok(library) => println!("Valid schema {}: {} instances, revision {}", library.schema_version, library.instances.len(), library.revision),
        Err(error) => { eprintln!("{error}"); std::process::exit(1); }
    }
}
