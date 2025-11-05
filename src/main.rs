use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::fs;
use std::error::Error;

#[derive(Debug, Deserialize, Serialize)]
struct Settings {
    security_token: String,
    services: Vec<Service>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Service {
    service_name: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct Root {
    settings: Settings,
}

// Function to read and parse the YAML file into the Settings struct
fn read_yaml(file_path: &str) -> Result<Settings, Box<dyn Error>> {
    let yaml_content = fs::read_to_string(file_path)?;
    let root: Root = serde_yaml::from_str(&yaml_content)?;
    Ok(root.settings)
}

// Handler for /update-service (GET)
async fn update_service() -> impl Responder {
    let file_path = "./settings.yaml";
    match read_yaml(file_path) {
        Ok(settings) => HttpResponse::Ok().json(settings), // Return full settings as JSON
        Err(err) => HttpResponse::InternalServerError().body(format!("Error reading YAML: {}", err)),
    }
}

// Handler for /get-services (GET)
async fn get_services() -> impl Responder {
    let file_path = "./settings.yaml";
    match read_yaml(file_path) {
        Ok(settings) => {
            // Return only the services list as JSON
            HttpResponse::Ok().json(settings.services)
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error reading YAML: {}", err)),
    }
}

// Handler for /get-security-token (GET)
async fn get_security_token() -> impl Responder {
    let file_path = "./settings.yaml";
    match read_yaml(file_path) {
        Ok(settings) => {
            // Return only the security token as JSON
            HttpResponse::Ok().json(settings.security_token)
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error reading YAML: {}", err)),
    }
}

// Catch-all route handler for unmatched paths
async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("Sorry, this path does not exist!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Start the Actix web server and define the routes
    HttpServer::new(|| {
        App::new()
            .route("/update-service", web::get().to(update_service))      // Handle /update-service
            .route("/get-services", web::get().to(get_services))          // Handle /get-services
            .route("/get-security-token", web::get().to(get_security_token)) // Handle /get-security-token
            .default_service(web::get().to(not_found))                    // Catch all unhandled paths
    })
    .bind("127.0.0.1:8080")? // Bind to localhost:8080
    .run()
    .await
}
