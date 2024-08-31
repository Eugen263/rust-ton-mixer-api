//! # TON Mixer Operations
//!
//! This module provides functionality for interacting with a TON (The Open Network) mixer contract,
//! including initializing a TON client, creating a wallet, and performing various contract operations.


use std::{str::FromStr, thread, time::{Duration, SystemTime}};

use tonlib::{address::TonAddress, cell::{ArcCell, Cell, CellBuilder}, client::{TonClient, TonClientBuilder, TonClientInterface, TonConnectionParams}, contract::{TonContract, TonContractFactory, TonWalletContract}, mnemonic::{KeyPair, Mnemonic}, wallet::{TonWallet, WalletVersion}
};

use crate::types::{create_external_singed_message, CollectMessage, CollectMessageData, ForkMessage, SpreadMessage, SpreadWallet, TXHash};
use base64::{Engine as _, engine::general_purpose};
use hex;

/// Initializes and returns a TON client.
///
/// # Panics
///
/// Panics if the TON client initialization fails.
async fn ton_client() -> TonClient {
    let testnet_config = include_str!("../config/testnet-global.config.json").to_string();
    let client_builder = TonClientBuilder::new()
        .with_connection_params(&TonConnectionParams{
            config: testnet_config,
            blockchain_name: None,
            use_callbacks_for_network: false,
            ignore_cache: false,
            keystore_dir: None,
            notification_queue_length: 100,
            concurrency_limit: 5,
        })
        .with_pool_size(10)
        .with_logging_callback()
        .build()
        .await;

    match client_builder {
        Ok(client) => return client,
        Err(err) => {
            panic!("[ FATAL ] Ton Client Initialization Error: Can not establish connection \n {:?}", err);
        }
    }
}

/// Creates and returns a TON wallet.
///
/// # Panics
///
/// Panics if the wallet mnemonic environment variable is not set or invalid.
fn ton_wallet() -> TonWallet {
    let mnemonic_str: String = std::env::var("WALLET_MNEMONIC").unwrap();
    let mnemonic: Mnemonic = Mnemonic::from_str(&mnemonic_str, &None).unwrap();
    let keys: KeyPair = mnemonic.to_key_pair().unwrap();

    let wallet = TonWallet::derive_default(WalletVersion::V4R2, &keys).unwrap();
    return wallet;
}

/// Returns the current Unix timestamp.
fn time_now() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}

/// Sends a raw message with retries.
///
/// This function is currently unused (dead code).
#[warn(dead_code)]
async fn send_with_retrys(client: &TonClient, tx: &Vec<u8>) -> Vec<u8> {
    let max = 2;
    let mut attempts = 0;

    loop {
        match client.send_raw_message_return_hash(tx).await {
            Ok(ans) => return ans,
            Err(e) if attempts < max => {
                println!("Attempt {} failed: {:?}. Retrying...", attempts + 1, e);
                thread::sleep(Duration::from_secs(2u64.pow(attempts)));
                attempts += 1;
            },
            Err(e) => {
                println!("client error {:?}", e);
                return Vec::<u8>::new();
            },
        }
    }
}

/// Invokes the fork operation on the mixer contract.
///
/// # Returns
///
/// A string containing the transaction hash in hex and base64 formats.
pub async fn contract_invoke_fork() -> String {
    let client: TonClient = ton_client().await;
    let user_wallet: TonWallet = ton_wallet();
    let contract_str: String = std::env::var("MIXER_CONTRACT").unwrap();

    let contract_factory: TonContractFactory = TonContractFactory::builder(&client).build().await.unwrap();
    let contract_address: TonAddress = TonAddress::from_str(&contract_str).unwrap();
    let wallet_contract: TonContract = contract_factory.get_contract(&user_wallet.address);

    let seqno: u32 = wallet_contract.seqno().await.unwrap();

    let body_payload: Cell = ForkMessage::new(time_now()).build();

    let tx: Vec<u8> = create_external_singed_message(
        user_wallet,
        seqno,
        contract_address,
        5000000u64,
        time_now(),
        body_payload
    );
    
    let hash: Vec<u8> = client.send_raw_message_return_hash(tx.as_slice()).await.unwrap();

    let hex_tx: String = hex::encode(&hash);
    let base64_tx: String = general_purpose::STANDARD.encode(&hash);

    return TXHash::new(hex_tx, base64_tx).to_string();
}

/// Invokes the spread operation on the mixer contract.
///
/// # Arguments
///
/// * `total_amount` - The total amount to spread.
/// * `spread_payload` - A vector of `SpreadWallet` structs containing the spread information.
///
/// # Returns
///
/// A string containing the transaction hash in hex and base64 formats.
pub async fn contract_invoke_spread(total_amount: u64, spread_payload: Vec<SpreadWallet>) -> String {
    let client: TonClient = ton_client().await;
    let user_wallet: TonWallet = ton_wallet();
    let contract_str: String = std::env::var("MIXER_CONTRACT").unwrap();

    let contract_factory: TonContractFactory = TonContractFactory::builder(&client).build().await.unwrap();
    let contract_address: TonAddress = TonAddress::from_str(&contract_str).unwrap();
    let wallet_contract: TonContract = contract_factory.get_contract(&user_wallet.address);

    let seqno: u32 = wallet_contract.seqno().await.unwrap();

    let mut payload = CellBuilder::new().build().unwrap();
    for entry in spread_payload {
        let previous_cell = payload;

        let mut builder = CellBuilder::new();
        builder.store_reference(&ArcCell::new(previous_cell)).unwrap();

        builder.store_address(&entry.account).unwrap();
        builder.store_coins(&entry.amount).unwrap();

        payload = builder.build().unwrap();
    }

    let body_payload: Cell = SpreadMessage::new(0, time_now(), total_amount, payload).build(); 

    let tx: Vec<u8> = create_external_singed_message(
        user_wallet,
        seqno,
        contract_address,
        total_amount+5000000u64, //send total amount to spread + fee
        time_now(),
        body_payload
    );
    
    let hash: Vec<u8> = client.send_raw_message_return_hash(tx.as_slice()).await.unwrap();
    
    let hex_tx = hex::encode(&hash);
    let base64_tx = general_purpose::STANDARD.encode(&hash);

    return TXHash::new(hex_tx, base64_tx).to_string();
}

/// Invokes the collect operation on the mixer contract.
///
/// # Arguments
///
/// * `message_data` - A `CollectMessageData` struct containing the collect operation details.
///
/// # Returns
///
/// A string containing the transaction hash in hex and base64 formats.
pub async fn contract_invoke_collect(message_data: CollectMessageData) -> String {
    let client: TonClient = ton_client().await;
    let user_wallet: TonWallet = ton_wallet();
    let contract_str: String = std::env::var("MIXER_CONTRACT").unwrap();

    let contract_factory: TonContractFactory = TonContractFactory::builder(&client).build().await.unwrap();
    let contract_address: TonAddress = TonAddress::from_str(&contract_str).unwrap();
    let wallet_contract: TonContract = contract_factory.get_contract(&user_wallet.address);

    let seqno: u32 = wallet_contract.seqno().await.unwrap();

    let body_payload: Cell = CollectMessage::new(
        message_data.mode, 
        time_now(),
        message_data.jetton_wallet,
        message_data.amount
    ).build().unwrap();

    let tx: Vec<u8> = create_external_singed_message(
        user_wallet,
        seqno,
        contract_address,
        50000000u64,
        time_now(),
        body_payload
    );
    
    let hash: Vec<u8> = client.send_raw_message_return_hash(tx.as_slice()).await.unwrap();

    let hex_tx = hex::encode(&hash);
    let base64_tx = general_purpose::STANDARD.encode(&hash);

    return TXHash::new(hex_tx, base64_tx).to_string();
}