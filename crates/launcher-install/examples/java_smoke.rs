use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Mutex},
    time::{Duration, Instant},
};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() == 3,
        "usage: java_smoke <temporary-directory> <major>"
    );
    let root = PathBuf::from(&args[1]);
    std::fs::create_dir_all(&root).unwrap();
    let root = root.canonicalize().unwrap();
    assert!(root.starts_with(std::env::temp_dir()));
    let clock = Mutex::new((Instant::now() - Duration::from_secs(10), String::new()));
    let result = launcher_install::java::install_managed_java(
        &root,
        args[2].parse().unwrap(),
        &AtomicBool::new(false),
        &|p| {
            let mut c = clock.lock().unwrap();
            if c.0.elapsed() > Duration::from_secs(10) || c.1 != p.phase {
                println!("{}: {} / {}", p.phase, p.completed, p.total);
                *c = (Instant::now(), p.phase);
            }
        },
    );
    match result {
        Ok(j) => println!(
            "Validated Java {} {} at {}",
            j.major,
            j.version,
            j.path.display()
        ),
        Err(e) => {
            eprintln!("FAILED: {e}");
            std::process::exit(1);
        }
    }
}
