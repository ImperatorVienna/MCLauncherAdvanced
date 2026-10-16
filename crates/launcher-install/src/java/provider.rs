//! Java provider protocol is shared; host parameters come from the platform adapter.
use crate::{download::{self,File},err,Result};
pub fn package(client:&reqwest::blocking::Client,major:u32)->Result<File> {
    let os=launcher_platform::OS_NAME;
    let data=download::bytes(client,&format!("https://api.adoptium.net/v3/assets/latest/{major}/hotspot?architecture=x64&heap_size=normal&image_type=jdk&jvm_impl=hotspot&os={os}&vendor=eclipse"),4*1024*1024)
        .map_err(|e|format!("No Java {major} package is available; install a system runtime and retry: {e}"))?;
    let entries:serde_json::Value=serde_json::from_slice(&data).map_err(err)?;
    let item=entries.as_array().and_then(|items|items.first()).ok_or("Java download catalog is empty; install a system runtime and retry")?;
    let package=&item["binary"]["package"];
    let checksum=package["checksum"].as_str().ok_or("Missing Java checksum")?;
    if checksum.len()!=64 || !checksum.bytes().all(|b|b.is_ascii_hexdigit()){return Err("Invalid Java SHA-256 checksum".into())}
    Ok(File{path:format!("downloads/{major}-{checksum}.{}",launcher_platform::JAVA_ARCHIVE_EXTENSION),url:package["link"].as_str().ok_or("Missing Java download URL")?.into(),hash:checksum.into(),size:package["size"].as_u64().ok_or("Missing Java file size")?})
}
