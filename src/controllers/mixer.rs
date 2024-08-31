//! # Mixer Controllers
//!
//! This module defines the controller functions for the mixer service in the TON (The Open Network) application.
//! It handles incoming HTTP requests, performs input validation, and calls the appropriate service functions.

use std::str::FromStr;

use actix_web::{error::ErrorBadRequest, get, post, web::Json, Error, HttpResponse};
use tonlib::address::{TonAddress, TonAddressParseError};

use crate::{services::mixer, types::{CollectPayload, Response, SpreadWalletPayload}};

/// Handles the spread operation.
///
/// # Arguments
///
/// * `body_payload` - A JSON payload containing a vector of `SpreadWalletPayload`.
///
/// # Returns
///
/// Returns an HTTP response or an error.
#[post("/spread")]
pub async fn spread(body_payload: Json<Vec<SpreadWalletPayload>>) -> Result<HttpResponse, Error> {
    return mixer::spread(&body_payload.0).await;
}

/// Handles the collect operation.
///
/// This function performs input validation for collection mode 3,
/// checking for the presence and validity of `jetton_wallet` and `amount` fields.
///
/// # Arguments
///
/// * `body_payload` - A JSON payload containing `CollectPayload`.
///
/// # Returns
///
/// Returns an HTTP response or an error.
#[post("/collect")]
pub async fn collect(body_payload: Json<CollectPayload>) -> Result<HttpResponse, Error> {
    let payload: CollectPayload = body_payload.0;

    if payload.mode == 3 {
        if payload.jetton_wallet.is_none() {
            return Err(ErrorBadRequest(
                Response::error(
                    serde_json::Value::String(String::from("in collection mode 3 field `jetton_wallet` is required"))
                ).to_string()
            ));
        } else {
            let check: Result<TonAddress, TonAddressParseError> = TonAddress::from_str(&payload.jetton_wallet.clone().unwrap());

            match check {
                Ok(_) => {},
                Err(err) => {
                    return Err(ErrorBadRequest(
                        Response::error(
                            serde_json::Value::String(err.to_string())
                        ).to_string()
                    ));
                }
            }
        }

        if payload.amount.is_none() {
            return Err(ErrorBadRequest(
                Response::error(
                    serde_json::Value::String(String::from("in collection mode 3 field `amount` is required"))
                ).to_string()
            ));
        }
    }

    return mixer::collect(payload).await;
}

/// Retrieves the collection modes.
///
/// # Returns
///
/// Returns an HTTP response containing the collection modes or an error.
#[get("/collect/modes")]
pub async fn get_collect_modes() -> Result<HttpResponse, Error> {
    return mixer::get_collect_modes().await;
}

/// Handles the fork operation.
///
/// # Returns
///
/// Returns an HTTP response or an error.
#[post("/fork")]
pub async fn fork() -> Result<HttpResponse, Error> {
    return mixer::fork().await;
}

/// Retrieves the operation codes.
///
/// # Returns
///
/// Returns an HTTP response containing the operation codes or an error.
#[get("/op_codes")]
pub async fn opcodes() -> Result<HttpResponse, Error> {
    return mixer::get_opcodes().await;
}