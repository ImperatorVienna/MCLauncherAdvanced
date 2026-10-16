//! Explicit isolated integration check. Never defaults to the user's real game directory.
use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Mutex},
    time::{Duration, Instant},
};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() == 3,
        "usage: install_smoke <test-directory> <minecraft-version>"
    );
    let root = PathBuf::from(&args[1]);
    std::fs::create_dir_all(&root).unwrap();
    let root = root.canonicalize().unwrap();
    assert!(
        root.starts_with(std::env::temp_dir()),
        "smoke tests must use a temporary directory"
    );
    let game = root.join("game");
    std::fs::create_dir_all(&game).unwrap();
    let clock = Mutex::new((Instant::now() - Duration::from_secs(10), String::new()));
    let result = launcher_install::minecraft::install(
        &game,
        None,
        &args[2],
        &AtomicBool::new(false),
        &|p| {
            let mut clock = clock.lock().unwrap();
            if clock.0.elapsed() > Duration::from_secs(10) || clock.1 != p.phase {
                println!("{}: {} / {}", p.phase, p.completed, p.total);
                *clock = (Instant::now(), p.phase);
            }
        },
    );
    match result {
        Ok(r) => {
            assert!(launcher_install::minecraft::installed(&game, &args[2]).installed);
            println!(
                "Installed {} (Java selection: {:?}): {} files",
                r.minecraft_version,
                r.java,
                r.files.len()
            );
        }
        Err(e) => {
            eprintln!("FAILED: {e}");
            std::process::exit(1);
        }
    }
}
