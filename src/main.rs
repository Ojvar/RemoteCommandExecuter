mod web_helper;
mod yaml_helper;

use crate::{
    web_helper::{get_services, not_found, update_services},
    yaml_helper::read_yaml,
};
use actix_web::{App, HttpServer, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = read_yaml().await.unwrap();
    let security_token = settings.security_token.clone();
    let server = HttpServer::new(move || {
        let token = security_token.clone();
        App::new()
            .wrap(web_helper::token_middleware(token))
            .route("/get-services", web::get().to(get_services))
            .route("/update-service", web::get().to(update_services))
            .default_service(web::get().to(not_found))
    })
    .bind(&settings.host)?;
    println!(
        "Server started successfully and listening on {}",
        settings.host
    );
    server.run().await
}
