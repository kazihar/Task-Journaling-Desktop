use std::path::Path;

use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, Responder, delete, get, http::header, post, web};
use serde::Deserialize;

use crate::{
    types::{Config, CreateUpdateRequest, Records, SystemInfo},
    utils::{get_disk_details, get_top_5_process_info},
};

#[post("/entry")]
async fn create_journal(
    payload: web::Json<CreateUpdateRequest>,
    state: web::Data<Records>,
    config: web::Data<Config>,
) -> impl Responder {
    match state
        .insert(
            payload.title.clone(),
            payload.body.clone(),
            payload.tags.clone(),
            config.into_inner(),
        )
        .await
    {
        Ok(id) => HttpResponse::Created().json(id),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/entry/{id}")]
async fn get_by_id(
    id: web::Path<String>,
    state: web::Data<Records>,
    config: web::Data<Config>,
) -> impl Responder {
    match state.find_by_id(&id, config.into_inner()).await {
        Ok(journal) => HttpResponse::Ok().json(journal),
        Err(err) => HttpResponse::NotFound().body(err.to_string()).into(),
    }
}

#[derive(Deserialize)]
struct QueryTag {
    tag: Option<String>,
}

#[get("/entries")]
async fn get_all(
    tag: web::Query<QueryTag>,
    state: web::Data<Records>,
    config: web::Data<Config>,
) -> impl Responder {
    let query = tag.into_inner();
    match state.find_by_tag(query.tag, config.into_inner()).await {
        Ok(journals) => HttpResponse::Ok().json(journals),
        Err(err) => HttpResponse::from_error(err),
    }
}

#[delete("/entry/{id}")]
async fn delete_by_id(
    id: web::Path<String>,
    state: web::Data<Records>,
    config: web::Data<Config>,
) -> impl Responder {
    match state.delete_by_id(&id, config.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(id.to_string()),
        Err(err) => HttpResponse::from_error(err),
    }
}

#[post("/export")]
async fn export(
    req: HttpRequest,
    state: web::Data<Records>,
    config: web::Data<Config>,
) -> impl Responder {
    let file_name = "journal.md";

    if let Err(e) = state.export(file_name, config.into_inner()).await {
        return HttpResponse::InternalServerError().body(format!("Failed to export file: {}", e));
    }

    if Path::new(file_name).exists() {
        match NamedFile::open_async(file_name).await {
            Ok(mut file) => {
                file = file.set_content_disposition(header::ContentDisposition {
                    disposition: header::DispositionType::Attachment,
                    parameters: vec![header::DispositionParam::Filename(file_name.to_string())],
                });
                return file.into_response(&req);
            }
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(format!("Failed to open file: {}", e));
            }
        }
    } else {
        return HttpResponse::NotFound().body("File not found");
    }
}

#[get("/sysinfo")]
async fn system_info() -> impl Responder {
    let disk_usage = get_disk_details();
    let current_user = whoami::username();
    let top_cpu_processes = get_top_5_process_info();

    // Return JSON response
    HttpResponse::Ok().json(SystemInfo {
        disk_usage,
        current_user,
        top_cpu_processes,
    })
}
