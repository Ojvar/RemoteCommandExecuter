use crate::yaml_helper::read_yaml;
use actix_web::Error;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use std::future::Future;
use std::future::{Ready, ready};
use std::pin::Pin;
use std::task::{Context, Poll};
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

// Token authentication middleware
pub struct TokenAuth {
    token: String,
}

pub fn token_middleware(token: String) -> TokenAuth {
    TokenAuth { token }
}

impl<S> Transform<S, ServiceRequest> for TokenAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = TokenAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TokenAuthMiddleware {
            service,
            token: self.token.clone(),
        }))
    }
}

pub struct TokenAuthMiddleware<S> {
    service: S,
    token: String,
}

impl<S> Service<ServiceRequest> for TokenAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let token = self.token.trim().to_string();
        let has_token = req
            .headers()
            .get("x-token")
            .and_then(|h| h.to_str().ok())
            .map(|v| v.trim() == token)
            .unwrap_or_else(|| false);
        if !has_token {
            let response = req.into_response(HttpResponse::Unauthorized().finish());
            return Box::pin(async move { Ok(response) });
        }
        let fut = self.service.call(req);
        Box::pin(async move { fut.await })
    }
}
