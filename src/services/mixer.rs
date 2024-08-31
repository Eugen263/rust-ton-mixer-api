//! # TON Mixer Services
//!
//! This module provides service functions for a TON (The Open Network) mixer application,
//! including spreading funds, collecting funds, forking, and retrieving opcodes and collection modes.

use std::str::FromStr;

use actix_web::{Error, HttpResponse};
use num_bigint::BigUint;
use tonlib::address::TonAddress;

use crate::{ton::{self, contract_invoke_fork}, types::{CollectMessageData, CollectPayload, MixerCollectionModes, MixerOpcodes, SpreadWallet, SpreadWalletPayload}};

/// Spreads funds across multiple wallets.
///
/// # Arguments
///
/// * `wallets` - A vector of `SpreadWalletPayload` structs containing wallet addresses and amounts.
///
/// # Returns
///
/// Returns an HTTP response containing the transaction details.
pub async fn spread(wallets: &Vec<SpreadWalletPayload>) -> Result<HttpResponse, Error> {
    let mut total_coins_amout: u64 = 0;
    let serialized_closer_to_ton: Vec<SpreadWallet> = wallets.iter().map(| v | {
        let nano = (v.amount * 1_000_000_000.0).round() as u64;
        total_coins_amout += nano;

        SpreadWallet {
            account: TonAddress::from_str(&v.account).unwrap(),
            amount: BigUint::from_str(&nano.to_string()).unwrap()
        }
    }).collect();

    let tx: String = ton::contract_invoke_spread(
        total_coins_amout,
        serialized_closer_to_ton
    ).await;

    Ok(HttpResponse::Ok().body(tx))
}

/// Collects funds from the mixer.
///
/// # Arguments
///
/// * `payload` - A `CollectPayload` struct containing collection details.
///
/// # Returns
///
/// Returns an HTTP response containing the transaction details.
pub async fn collect(payload: CollectPayload) -> Result<HttpResponse, Error> {
    let mut collect_message_data: CollectMessageData = CollectMessageData {
        mode: payload.mode,
        jetton_wallet: None,
        amount: None
    };

    if let Some(w) = payload.jetton_wallet {
        collect_message_data.jetton_wallet = Some(TonAddress::from_str(&w).unwrap());
    }

    if let Some(a) = payload.amount {
        let nano: u64 = (a * 1_000_000_000.0).round() as u64;
        collect_message_data.amount = Some(BigUint::from(nano))
    }

    let tx = ton::contract_invoke_collect(collect_message_data).await;
    Ok(HttpResponse::Ok().body(tx))
}

/// Invokes the fork operation on the mixer contract.
///
/// # Returns
///
/// Returns an HTTP response containing the transaction details.
pub async fn fork() -> Result<HttpResponse, Error> {
    let a = contract_invoke_fork().await;
    Ok(HttpResponse::Ok().body(a))
}

/// Retrieves the opcodes for mixer operations.
///
/// # Returns
///
/// Returns an HTTP response containing the opcodes in JSON format.
pub async fn get_opcodes() -> Result<HttpResponse, Error> {
    let op = MixerOpcodes::new();

    Ok(HttpResponse::Ok().json(op))
}

/// Retrieves the collection modes for the mixer.
///
/// # Returns
///
/// Returns an HTTP response containing the collection modes in JSON format.
pub async fn get_collect_modes() -> Result<HttpResponse, Error> {
    let op = MixerCollectionModes::new();

    Ok(HttpResponse::Ok().json(op))
}