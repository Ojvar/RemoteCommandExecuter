use actix_web::{HttpResponse, Responder};

use crate::yaml_helper::read_yaml;

// Handler for /get-services (GET)
pub async fn get_services() -> impl Responder {
    match read_yaml().await {
        Ok(settings) => {
            // Return only the services list as JSON
            HttpResponse::Ok().json(settings.services)
        }
        Err(err) => {
            HttpResponse::InternalServerError().body(format!("Error reading YAML: {}", err))
        }
    }
}

// Catch-all route handler for unmatched paths
pub async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("Sorry, this path does not exist!")
}
