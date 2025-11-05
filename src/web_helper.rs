use crate::yaml_helper::read_yaml;
use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use tokio::process::Command;

#[derive(Deserialize)]
pub struct ServiceInfo {
    pub service_name: String,
}

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

pub async fn update_services(query: web::Query<ServiceInfo>) -> impl Responder {
    // Extract service_name from query string
    let service_name = &query.service_name;

    // Read settings to find the service
    match read_yaml().await {
        Ok(settings) => {
            // Find service with matching service_name
            if let Some(service) = settings
                .services
                .iter()
                .find(|s| s.service_name == *service_name)
            {
                // Execute the path value in shell
                println!("{}", &service.cmd);
                match Command::new("sh")
                    .arg("-c")
                    .arg(&service.cmd)
                    .output()
                    .await
                {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let status = if output.status.success() {
                            "success"
                        } else {
                            "failed"
                        };
                        HttpResponse::Ok().body(format!(
                            "Service '{}' executed with status: {}\nStdout: {}\nStderr: {}",
                            service_name, status, stdout, stderr
                        ))
                    }
                    Err(err) => HttpResponse::InternalServerError().body(format!(
                        "Error executing path for service '{}': {}",
                        service_name, err
                    )),
                }
            } else {
                HttpResponse::NotFound().body(format!(
                    "Service with name '{}' not found in settings",
                    service_name
                ))
            }
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
