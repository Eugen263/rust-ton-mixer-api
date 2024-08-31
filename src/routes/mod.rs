//! # Mixer Routes
//!
//! This module defines the routes for the mixer service in the TON (The Open Network) application.
//! It uses the Actix web framework to set up the routing.

use actix_web::{web, Scope};

use crate::controllers::mixer;

/// Creates and returns a new `Scope` for the mixer routes.
///
/// This function sets up the following routes under the "/mixer" path:
/// - POST /fork
/// - POST /spread
/// - POST /collect
/// - GET /collect_modes
/// - GET /opcodes
///
/// # Returns
///
/// Returns a `Scope` object configured with the mixer routes.
pub fn new() -> Scope {
    web::scope("/mixer")
        // Route for retrieving archived emails
        .service(mixer::fork)
        .service(mixer::spread)
        .service(mixer::collect)
        .service(mixer::get_collect_modes)
        .service(mixer::opcodes)
}
