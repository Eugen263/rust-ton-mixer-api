//! # HTTP Server
//! 
//! This module implements an HTTP server using Actix Web framework.
//! It sets up CORS, compression, and routes for the application.

use std::{io::Result, env};
use actix_cors::Cors;
use actix_web::{middleware::Compress, App, HttpServer};
use dotenv::dotenv;

pub mod routes;
pub mod controllers;
pub mod services;
pub mod types;
pub mod ton;

/// The main function that starts the HTTP server.
///
/// # Errors
///
/// This function will return an error if the server fails to start or
/// if there's an issue during server execution.
#[actix_web::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();
    // Parse the PORT environment variable
    let port: u16 = env::var("PORT").unwrap().parse::<u16>().unwrap();

    println!("[ INFO ] Http server is starting on port {:?}", port);

    // Create and run the HTTP server
    HttpServer::new(|| {
        App::new()
            .wrap(
                // Configure CORS
                Cors::default()
                .allowed_origin("http://localhost:5173")
                .allowed_origin("http://localhost:3001")
                .allowed_methods(vec![
                    "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS",
                ])
                .allowed_headers(vec![
                    actix_web::http::header::CONTENT_TYPE,
                ])
            )
            .wrap(Compress::default()) // Enable compression
            .service(routes::new()) // Add routes
    })
    .workers(num_cpus::get() * 2) // Set number of workers to twice the number of CPU cores
    .bind(("0.0.0.0", port)) // Bind to all interfaces on the specified port
    .unwrap()
    .run()
    .await
}
