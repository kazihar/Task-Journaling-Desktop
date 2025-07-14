use std::{
    fs::{self, File},
    io::{self, Error, ErrorKind, Read, Write},
    path::Path,
};

use aes_gcm::{
    AeadCore, Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use base64::{Engine, engine::general_purpose};
use sysinfo::{Disks, System};

use crate::types::{Config, DiskInfo, EncryptedFile, Journal, ProcessInfo};

pub fn get_config() -> Result<Config, Error> {
    let config = fs::read_to_string("conf.toml").map_err(|e| {
        Error::new(
            ErrorKind::NotFound,
            format!("The system cannot find the path conf.toml, {}", e),
        )
    })?;

    config.parse::<Config>()
}

pub fn write_to_md_file(records: Vec<Journal>, filename: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;

    for journal in records {
        let mut title = "Untitled".to_string();
        let mut body = "".to_string();
        if journal.title.is_some() {
            title = journal.title.unwrap();
        }
        if journal.body.is_some() {
            body = journal.body.unwrap();
        }
        writeln!(file, "# {}\n\n{}\n", title, body)?;
    }

    Ok(())
}

pub fn get_disk_details() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    disks
        .iter()
        .map(|disk| {
            let total_space = disk.total_space();
            let available_space = disk.available_space();
            let used_percentage = if total_space > 0 {
                ((total_space - available_space) as f64 / total_space as f64) * 100.0
            } else {
                0.0
            };
            DiskInfo {
                mount_point: disk.mount_point().to_string_lossy().into_owned(),
                total_space,
                available_space,
                used_percentage,
            }
        })
        .collect::<Vec<DiskInfo>>()
}

pub fn get_top_5_process_info() -> Vec<ProcessInfo> {
    let mut system = System::new_all();
    system.refresh_all();
    let mut processes = vec![];
    for (pid, process) in system.processes() {
        processes.push((pid.as_u32(), process));
    }

    processes.sort_by(|a, b| {
        b.1.cpu_usage()
            .partial_cmp(&a.1.cpu_usage())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    processes
        .iter()
        .take(5)
        .map(|(pid, proc)| ProcessInfo {
            pid: pid.to_owned(),
            name: proc.name().display().to_string(),
            cpu_usage: proc.cpu_usage(),
        })
        .collect::<Vec<ProcessInfo>>()
}

pub fn encrypt_data(plaintext: &str, key: &[u8; 32]) -> Result<(String, String), aes_gcm::Error> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bit nonce
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes())?;

    // Encode ciphertext and nonce as base64 for storage
    let ciphertext_b64 = general_purpose::STANDARD.encode(&ciphertext);
    let nonce_b64 = general_purpose::STANDARD.encode(&nonce);

    Ok((ciphertext_b64, nonce_b64))
}

pub fn decrypt_data(
    ciphertext_b64: &str,
    nonce_b64: &str,
    key: &[u8; 32],
) -> Result<String, aes_gcm::Error> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();

    // Decode from base64
    let ciphertext = general_purpose::STANDARD.decode(ciphertext_b64).unwrap();
    let nonce = general_purpose::STANDARD.decode(nonce_b64).unwrap();

    let nonce = Nonce::from_slice(&nonce);
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())?;

    Ok(String::from_utf8(plaintext).unwrap())
}

pub fn get_key(secret: String) -> [u8; 32] {
    secret.as_bytes().try_into().expect("Key must be 32 bytes")
}

pub fn read_file(file_path: &str) -> Result<EncryptedFile, std::io::Error> {
    let mut file = File::open(Path::new(file_path))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let parsed_file_content = serde_json::from_str::<EncryptedFile>(&contents)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

    Ok(parsed_file_content)
}

pub fn write_to_file(
    path: String,
    id: String,
    text: String,
    nonce: String,
) -> Result<(), std::io::Error> {
    let content = EncryptedFile {
        content: text,
        nonce,
    };

    let json_string = serde_json::to_string(&content)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

    let mut file = File::create(Path::new(&format!("{}/{}.json", path, id)))?;
    file.write_all(json_string.as_bytes())?;
    Ok(())
}

pub fn list_files_in_a_dir(dir_path: &str, key: [u8; 32]) -> Result<Vec<Journal>, io::Error> {
    let path = Path::new(dir_path);
    let metadata = path.metadata()?;
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            format!("{} is not a directory", dir_path),
        ));
    }

    let permissions = metadata.permissions();
    if permissions.readonly() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("Directory {} is not readable", dir_path),
        ));
    }

    let mut results = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();
        if file_path.is_file() && file_path.extension().and_then(|s| s.to_str()) == Some("json") {
            let path_str = file_path.to_string_lossy().to_string();
            let encrypted = read_file(&path_str)?;
            match decrypt_data(&encrypted.content, &encrypted.nonce, &key) {
                Ok(stringified) => {
                    let parsed = serde_json::from_str::<Journal>(&stringified).map_err(|err| {
                        io::Error::new(io::ErrorKind::InvalidData, err.to_string())
                    })?;
                    results.push(parsed);
                }
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
            }
        }
    }
    Ok(results)
}
