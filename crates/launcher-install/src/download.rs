use crate::{check_cancel, err, safe_path, Progress, Report, Result};
use reqwest::{blocking::Client, Url};
use serde::{Deserialize, Serialize};
use sha1::Digest;
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};

pub fn trusted_url(raw: &str) -> bool {
    let Ok(url) = Url::parse(raw) else {
        return false;
    };
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && matches!(
            url.host_str(),
            Some(
                "cdn.modrinth.com" | "maven.minecraftforge.net" | "maven.neoforged.net" | "files.minecraftforge.net" | "meta.fabricmc.net" | "meta.quiltmc.org" | "maven.fabricmc.net" | "maven.quiltmc.org"
                    | "piston-meta.mojang.com"
                    | "piston-data.mojang.com"
                    | "launchermeta.mojang.com"
                    | "launcher.mojang.com"
                    | "libraries.minecraft.net"
                    | "resources.download.minecraft.net"
                    | "api.adoptium.net"
                    | "github.com"
                    | "api.github.com"
                    | "release-assets.githubusercontent.com"
                    | "objects.githubusercontent.com"
            )
        )
}
pub fn client() -> Result<Client> {
    client_with_timeout(Duration::from_secs(15))
}
/// Long transfers keep a bounded total deadline without inheriting metadata timeouts.
pub fn large_file_client() -> Result<Client> { client_with_timeout(Duration::from_secs(300)) }
fn client_with_timeout(timeout: Duration) -> Result<Client> {
    // Use blocking timeouts: async read_timeout creates Tokio timers on the caller
    // when reading the body, and ordinary installer workers have no Tokio reactor.
    Client::builder()
        .user_agent("mclauncher-advanced/0.1.0")
        .connect_timeout(Duration::from_secs(15))
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 8 || !trusted_url(attempt.url().as_str()) {
                attempt.error("Unsupported download redirect")
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(err)
}
pub fn bytes(client: &Client, url: &str, limit: u64) -> Result<Vec<u8>> {
    if !trusted_url(url) {
        return Err("Untrusted download URL".into());
    }
    let response = client
        .get(url)
        .timeout(Duration::from_secs(30))
        .send()
        .map_err(err)?
        .error_for_status()
        .map_err(err)?;
    let mut out = vec![];
    response
        .take(limit + 1)
        .read_to_end(&mut out)
        .map_err(err)?;
    if out.len() as u64 > limit {
        return Err("Metadata exceeds the size limit".into());
    }
    Ok(out)
}
pub fn hash_ok(bytes: &[u8], expected: &str) -> bool {
    match expected.len() {
        40 => format!("{:x}", sha1::Sha1::digest(bytes)) == expected,
        64 => format!("{:x}", sha2::Sha256::digest(bytes)) == expected,
        _ => false,
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct File {
    pub path: String,
    pub url: String,
    pub hash: String,
    pub size: u64,
}
impl File {
    pub fn validate(&self) -> Result<()> {
        if !trusted_url(&self.url)
            || !matches!(self.hash.len(), 40 | 64)
            || !self
                .hash
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            || self.size == 0
            || self.size > 4 * 1024 * 1024 * 1024
        {
            return Err(format!("Invalid download metadata: {}", self.path));
        }
        Ok(())
    }
}
pub fn valid_file(path: &Path, spec: &File, cancel: &AtomicBool) -> Result<bool> {
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(err(e)),
    };
    if file.metadata().map_err(err)?.len() != spec.size {
        return Ok(false);
    }
    let mut h1 = sha1::Sha1::new();
    let mut h2 = sha2::Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        check_cancel(cancel)?;
        let n = file.read(&mut buf).map_err(err)?;
        if n == 0 {
            break;
        }
        if spec.hash.len() == 40 {
            h1.update(&buf[..n]);
        } else {
            h2.update(&buf[..n]);
        }
    }
    Ok(if spec.hash.len() == 40 {
        format!("{:x}", h1.finalize())
    } else {
        format!("{:x}", h2.finalize())
    } == spec.hash)
}
fn transfer(
    client: &Client,
    root: &Path,
    spec: &File,
    cancel: &AtomicBool,
    report: &mut dyn FnMut(u64),
) -> Result<()> {
    spec.validate()?;
    check_cancel(cancel)?;
    let path = safe_path(root, &spec.path)?;
    if valid_file(&path, spec, cancel)? {
        report(spec.size);
        return Ok(());
    }
    let parent = path.parent().ok_or("Invalid download path")?;
    fs::create_dir_all(parent).map_err(err)?;
    let mut failure = String::new();
    for attempt in 0..3 {
        check_cancel(cancel)?;
        report(0);
        let result = (|| {
            let started = Instant::now();
            let mut response = client
                .get(&spec.url)
                .send()
                .map_err(err)?
                .error_for_status()
                .map_err(err)?;
            let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(err)?;
            let mut h1 = sha1::Sha1::new();
            let mut h2 = sha2::Sha256::new();
            let mut received = 0u64;
            let mut buf = [0u8; 65536];
            loop {
                check_cancel(cancel)?;
                if started.elapsed() > Duration::from_secs(300) {
                    return Err("Download request timed out; retry".into());
                }
                let n = response.read(&mut buf).map_err(err)?;
                if n == 0 {
                    break;
                }
                received += n as u64;
                if received > spec.size {
                    return Err("Downloaded file size mismatch".into());
                }
                temp.write_all(&buf[..n]).map_err(err)?;
                if spec.hash.len() == 40 {
                    h1.update(&buf[..n]);
                } else {
                    h2.update(&buf[..n]);
                }
                report(received);
            }
            let hash = if spec.hash.len() == 40 {
                format!("{:x}", h1.finalize())
            } else {
                format!("{:x}", h2.finalize())
            };
            if received != spec.size || hash != spec.hash {
                return Err("Downloaded file verification failed".into());
            }
            check_cancel(cancel)?;
            safe_path(root, &spec.path)?;
            temp.as_file().sync_all().map_err(err)?;
            temp.persist(&path).map_err(err)?;
            Ok(())
        })();
        match result {
            Ok(()) => return Ok(()),
            Err(e) => failure = e,
        }
        if attempt < 2 {
            for _ in 0..5 {
                check_cancel(cancel)?;
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
    Err(format!("{}：{failure}", spec.path))
}
/// Four workers, one serialized progress callback. Bytes include verified cached files.
pub fn download_all(
    client: &Client,
    root: &Path,
    files: &[File],
    cancel: &AtomicBool,
    report: &Report<'_>,
    label: &str,
) -> Result<()> {
    let total = files.iter().map(|f| f.size).sum();
    let state = Mutex::new((
        vec![0u64; files.len()],
        0u64,
        Instant::now() - Duration::from_secs(1),
    ));
    let index = AtomicUsize::new(0);
    let failure = Mutex::new(None);
    std::thread::scope(|scope| {
        for _ in 0..4.min(files.len()) {
            scope.spawn(|| loop {
                if failure.lock().unwrap().is_some() {
                    break;
                }
                let i = index.fetch_add(1, Ordering::Relaxed);
                if i >= files.len() {
                    break;
                }
                let result = transfer(client, root, &files[i], cancel, &mut |n| {
                    let mut s = state.lock().unwrap();
                    s.1 = s.1 - s.0[i] + n;
                    s.0[i] = n;
                    if s.2.elapsed() >= Duration::from_millis(100) {
                        report(Progress {
                            phase: label.into(),
                            completed: s.1,
                            total,
                        });
                        s.2 = Instant::now();
                    }
                });
                if let Err(e) = result {
                    let mut error = failure.lock().unwrap();
                    if error.is_none() {
                        *error = Some(e);
                    }
                    break;
                }
            });
        }
    });
    if let Some(e) = failure.into_inner().unwrap() {
        return Err(e);
    }
    check_cancel(cancel)?;
    report(Progress {
        phase: label.into(),
        completed: total,
        total,
    });
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_and_hash_validation() {
        assert!(trusted_url("https://libraries.minecraft.net/test"));
        for u in [
            "http://libraries.minecraft.net/x",
            "https://libraries.minecraft.net.evil/x",
            "file:///tmp/x",
            "https://user@github.com/x",
        ] {
            assert!(!trusted_url(u));
        }
        assert!(hash_ok(b"abc", "a9993e364706816aba3e25717850c26c9cd0d89d"));
        assert!(!hash_ok(b"ab", "a9993e364706816aba3e25717850c26c9cd0d89d"));
    }
    #[test]
    fn corrupted_cache_is_not_accepted_and_cancel_stops_checks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file");
        fs::write(&path, b"abc").unwrap();
        let spec = File {
            path: "file".into(),
            url: "https://libraries.minecraft.net/test".into(),
            size: 3,
            hash: "a9993e364706816aba3e25717850c26c9cd0d89d".into(),
        };
        let cancel = AtomicBool::new(false);
        assert!(valid_file(&path, &spec, &cancel).unwrap());
        fs::write(&path, b"abd").unwrap();
        assert!(!valid_file(&path, &spec, &cancel).unwrap());
        cancel.store(true, Ordering::Relaxed);
        assert!(valid_file(&path, &spec, &cancel).is_err());
    }
    #[test]
    fn blocking_response_can_be_read_on_plain_worker_thread() {
        // Match the desktop installer's ordinary std::thread, with no Tokio context.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = [0u8; 4096];
            stream.read(&mut request).unwrap();
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
                .unwrap();
        });
        let worker = std::thread::spawn(move || {
            let mut response = client()
                .unwrap()
                .get(format!("http://{address}/test"))
                .send()
                .unwrap();
            let mut body = String::new();
            response.read_to_string(&mut body).unwrap();
            assert_eq!(body, "ok");
        });
        server.join().unwrap();
        worker.join().unwrap();
    }

    #[test]
    fn stalled_body_returns_timeout_without_panicking() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = [0u8; 4096];
            stream.read(&mut request).unwrap();
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n")
                .unwrap();
            std::thread::sleep(Duration::from_millis(600));
        });
        let worker = std::thread::spawn(move || {
            let mut response = client_with_timeout(Duration::from_millis(200))
                .unwrap()
                .get(format!("http://{address}/test"))
                .send()
                .unwrap();
            let mut body = String::new();
            assert!(response.read_to_string(&mut body).is_err());
        });
        worker.join().unwrap();
        server.join().unwrap();
    }
}
