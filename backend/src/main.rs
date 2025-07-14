use actix_cors::Cors;
use actix_web::{App, HttpServer, web::Data};
use backend::{
    handlers::{create_journal, delete_by_id, export, get_all, get_by_id, system_info},
    types::Records,
    utils::get_config,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let records = Data::new(Records::new());

    let config = match get_config() {
        Ok(c) => Data::new(c),
        Err(e) => {
            eprintln!("Config file not found. Exiting server with error {:?}", e);
            return Err(e);
        }
    };

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(records.clone())
            .app_data(config.clone())
            .service(create_journal)
            .service(get_all)
            .service(get_by_id)
            .service(delete_by_id)
            .service(export)
            .service(system_info)
    })
    .bind(("127.0.0.1", 7000))?
    .run()
    .await
}
