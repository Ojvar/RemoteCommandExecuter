mod web_helper;
mod yaml_helper;

use crate::{
    web_helper::{get_services, not_found},
    yaml_helper::read_yaml,
};
use actix_web::{App, HttpServer, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = read_yaml().await.unwrap();
    HttpServer::new(move || {
        App::new()
            .route("/get-services", web::get().to(get_services))
            .default_service(web::get().to(not_found))
    })
    .bind(settings.host)?
    .run()
    .await
}
