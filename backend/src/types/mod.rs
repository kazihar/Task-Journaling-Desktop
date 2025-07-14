use std::{
    collections::HashMap,
    fs,
    io::{self, Error},
    path::PathBuf,
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::utils::{
    decrypt_data, encrypt_data, get_key, list_files_in_a_dir, read_file, write_to_file,
    write_to_md_file,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    storage: PathBuf,
    secret: String,
}

impl std::str::FromStr for Config {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

#[derive(Serialize)]
pub struct SystemInfo {
    pub disk_usage: Vec<DiskInfo>,
    pub current_user: String,
    pub top_cpu_processes: Vec<ProcessInfo>,
}

#[derive(Serialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_percentage: f64,
}

#[derive(Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CreateUpdateRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EncryptedFile {
    pub content: String,
    pub nonce: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Journal {
    pub id: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub tags: Vec<String>,
}

impl Journal {
    fn new(id: String, title: Option<String>, body: Option<String>, tags: Vec<String>) -> Self {
        Journal {
            id,
            title,
            body,
            tags: tags,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Records {
    pub records: Arc<Mutex<HashMap<String, Journal>>>,
}

impl Records {
    pub fn new() -> Self {
        Records {
            records: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn insert(
        &self,
        title: Option<String>,
        body: Option<String>,
        tags: Vec<String>,
        config: Arc<Config>,
    ) -> Result<String, Error> {
        let id = Uuid::new_v4().to_string();
        let journal = Journal::new(id.clone(), title, body, tags);
        if let Ok(stringified) = serde_json::to_string(&journal) {
            let key = get_key(config.secret.clone());
            match encrypt_data(&stringified, &key) {
                Ok((text, nonce)) => {
                    write_to_file(
                        config.storage.to_string_lossy().to_string(),
                        id.clone(),
                        text,
                        nonce,
                    )?;
                    Ok(id)
                }
                Err(err) => Err(Error::new(io::ErrorKind::Other, err.to_string())),
            }
        } else {
            Err(Error::new(
                io::ErrorKind::InvalidData,
                "Unable to convert data to string",
            ))
        }
    }

    pub async fn find_by_id(&self, id: &String, config: Arc<Config>) -> Result<Journal, io::Error> {
        match read_file(&format!("{}/{}.txt", config.storage.to_string_lossy(), id)) {
            Ok(encrypted) => {
                let key = get_key(config.secret.clone());
                match decrypt_data(&encrypted.content, &encrypted.nonce, &key) {
                    Ok(stringified) => {
                        let parsed =
                            serde_json::from_str::<Journal>(&stringified).map_err(|err| {
                                io::Error::new(io::ErrorKind::InvalidData, err.to_string())
                            })?;
                        Ok(parsed)
                    }
                    Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
                }
            }
            Err(err) => Err(err),
        }
    }

    pub async fn find_by_tag(
        &self,
        tag: Option<String>,
        config: Arc<Config>,
    ) -> Result<Vec<Journal>, io::Error> {
        let mut result = vec![];

        let files_list = list_files_in_a_dir(
            &config.storage.to_string_lossy().to_string(),
            get_key(config.secret.clone()),
        )?;

        if let Some(tag) = tag {
            for journal in files_list.iter() {
                if journal.tags.contains(&tag) {
                    result.push(journal.clone());
                }
            }
            return Ok(result);
        }

        Ok(files_list)
    }

    pub async fn delete_by_id(&self, id: &String, config: Arc<Config>) -> Result<(), io::Error> {
        let path = format!(
            "{}/{}.json",
            config.storage.to_string_lossy().to_string(),
            id
        );
        match fs::remove_file(path) {
            Ok(_) => Ok(()),
            Err(err) => Err(io::Error::new(io::ErrorKind::NotFound, err.to_string())),
        }
    }

    pub async fn export(&self, file_name: &str, config: Arc<Config>) -> Result<(), std::io::Error> {
        let files_list = list_files_in_a_dir(
            &config.storage.to_string_lossy().to_string(),
            get_key(config.secret.clone()),
        )?;

        let mut result = vec![];
        for journal in files_list {
            result.push(journal.clone());
        }

        write_to_md_file(result, file_name)
    }
}
