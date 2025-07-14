use reqwest::{Error, Response};

use crate::types::CreateJournalRequest;

pub async fn create_journal(body: String, title: String, tags: Vec<String>) {
    let client = reqwest::Client::new();
    let payload = CreateJournalRequest { body, title, tags };

    match client
        .post("http://127.0.0.1:7000/entry")
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            println!("Response: {:?}", resp);
        }
        Err(err) => eprintln!("Error: {:?}", err),
    }
}

pub async fn delete_journal(id: String) {
    let client = reqwest::Client::new();

    match client
        .delete(&format!("http://127.0.0.1:7000/entry/{}", id))
        .send()
        .await
    {
        Ok(resp) => {
            println!("Response: {:?}", resp);
        }
        Err(err) => eprintln!("Error: {:?}", err),
    }
}

pub async fn export() -> Result<Response, Error> {
    let client = reqwest::Client::new();

    match client.post("http://127.0.0.1:7000/export").send().await {
        Ok(resp) => Ok(resp),
        Err(err) => Err(err),
    }
}

pub fn remove_from_vec(tags: Vec<String>, remove_tag: String) -> Vec<String> {
    tags.iter()
        .filter(|tag| **tag != remove_tag)
        .cloned()
        .collect::<Vec<String>>()
}
