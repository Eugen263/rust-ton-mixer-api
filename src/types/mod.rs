//! # TON Mixer Types and Functions
//!
//! This module defines types and functions for a TON (The Open Network) mixer,
//! including response types, wallet operations, and message building.

use std::sync::Arc;

use crc32fast::Hasher;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tonlib::{address::TonAddress, cell::{ArcCell, BagOfCells, Cell, CellBuilder}, message::TransferMessage, wallet::TonWallet};

use num_bigint::BigUint;

/// Represents the status of a response.
#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseStatus {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "success")]
    Success
}

/// Represents a response with a status and a message.
#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub status: ResponseStatus,
    pub message: Value
}

impl Response {
    /// Creates a new error response.
    pub fn error(message: Value) -> Response {
        Response{
            status: ResponseStatus::Error,
            message
        }
    }

    /// Creates a new info response.
    pub fn info(message: Value) -> Response {
        Response{
            status: ResponseStatus::Info,
            message
        }
    }

    /// Creates a new success response.
    pub fn success(message: Value) -> Response {
        Response{
            status: ResponseStatus::Success,
            message
        }
    }

    /// Converts the response to a JSON string.
    pub fn to_string(&self) -> String {
        serde_json::to_string::<Response>(self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TXHash {
    pub hex: String,
    pub base64: String
}

impl TXHash {
    pub fn new(hex: String, base64: String) -> Self {
        TXHash{
            hex,
            base64
        }
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string::<TXHash>(self).unwrap()
    }
}

/// Represents the payload for a spread wallet operation.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpreadWalletPayload {
    pub account: String,
    pub amount: f64
}

/// Represents a spread wallet with a TON address and amount.
pub struct SpreadWallet {
    pub account: TonAddress,
    pub amount: BigUint
}

/// Represents the payload for a collect operation.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectPayload {
    pub mode: u8,
    pub jetton_wallet: Option<String>,
    pub amount: Option<f64>
}

/// Represents the data for a collect message.
pub struct CollectMessageData {
    pub mode: u8,
    pub jetton_wallet: Option<TonAddress>,
    pub amount: Option<BigUint>
}

/// Represents the opcodes for mixer operations.
#[derive(Serialize, Deserialize)]
pub struct MixerOpcodes {
    pub spread: u32,
    pub collect: u32,
    pub fork: u32
}

/// Generates an opcode for a given method name.
fn generate_opcode(method_name: &str) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(method_name.as_bytes());
    hasher.finalize()
}

impl MixerOpcodes {
    /// Creates a new MixerOpcodes instance with generated opcodes.
    pub fn new() -> MixerOpcodes {
        MixerOpcodes {
            spread: generate_opcode("op::spread"),
            collect: generate_opcode("op::collect"),
            fork: generate_opcode("op::fork")
        }
    }
}

/// Represents the collection modes for the mixer.
#[derive(Serialize, Deserialize)]
pub struct MixerCollectionModes {
    pub current_message_ton_balance: u8,
    pub all_ton_balance: u8,
    pub available_ton_balance: u8,
    pub given_jetton_balance: u8
}

impl MixerCollectionModes {
    /// Creates a new MixerCollectionModes instance with predefined modes.
    pub fn new() -> MixerCollectionModes {
        MixerCollectionModes {
            current_message_ton_balance: 0,
            all_ton_balance: 1,
            available_ton_balance: 2,
            given_jetton_balance: 3
        }
    }
}

/// Represents a fork message.
#[derive(Clone)]
pub struct ForkMessage {
    pub timestamp: u64,
}

impl ForkMessage {
    /// Creates a new ForkMessage instance.
    pub fn new(timestamp: u64) -> Self {
        ForkMessage {
            timestamp,
        }
    }

    /// Builds the fork message cell.
    pub fn build(&self) -> Cell {
        let mut mess_builder: CellBuilder = CellBuilder::new();
        mess_builder.store_u32(32, MixerOpcodes::new().fork).unwrap(); //operation
        mess_builder.store_u64(64, self.timestamp).unwrap(); //query_id

        return mess_builder.build().unwrap(); 
    }
}

/// Represents a spread message.
#[derive(Clone)]
pub struct SpreadMessage {
    pub mode: u8,
    pub timestamp: u64,
    pub amount: u64,
    pub data: Cell
}

impl SpreadMessage {
    /// Creates a new SpreadMessage instance.
    pub fn new(mode: u8, timestamp: u64, amount: u64,  data: Cell) -> Self {
        SpreadMessage {
            mode,
            timestamp,
            amount,
            data
        }
    }

    /// Builds the spread message cell.
    pub fn build(&self) -> Cell {
        let mut mess_builder: CellBuilder = CellBuilder::new();
        mess_builder.store_u32(32, MixerOpcodes::new().spread).unwrap(); //operation
        mess_builder.store_u64(64, self.timestamp).unwrap(); //query_id
        mess_builder.store_u64(64, self.amount).unwrap(); //total amount of coins
        mess_builder.store_u8(8, self.mode).unwrap(); //spread mode

        mess_builder.store_bit(true).unwrap();
        //apply body to message
        mess_builder.store_reference(&ArcCell::new(self.data.clone())).unwrap();

        return mess_builder.build().unwrap(); 
    }
}

/// Represents a collect message.
#[derive(Clone)]
pub struct CollectMessage {
    pub mode: u8,
    pub timestamp: u64,
    pub jetton_wallet: Option<TonAddress>,
    pub amount: Option<BigUint>
}

impl CollectMessage {
    /// Creates a new CollectMessage instance.
    pub fn new(mode: u8, timestamp: u64, jetton_wallet: Option<TonAddress>, amount: Option<BigUint>) -> Self {
        CollectMessage {
            mode,
            timestamp,
            jetton_wallet,
            amount
        }
    }

    /// Builds the collect message cell.
    pub fn build(&self) -> Result<Cell, String> {
        let mut mess_builder: CellBuilder = CellBuilder::new();
        mess_builder.store_u32(32, MixerOpcodes::new().collect).unwrap(); //operation
        mess_builder.store_u64(64, self.timestamp).unwrap(); //query_id
        mess_builder.store_u8(8, self.mode).unwrap(); //spread mode

        match self.mode {
            0 | 1 | 2 => {
                println!("Funds will be sent to the predefined target address stored in the contract state.");
            },
            3 => {
                if let (Some(wallet), Some(amt)) = (self.jetton_wallet.as_ref(), self.amount.as_ref()) {
                    mess_builder.store_address(wallet).unwrap();
                    mess_builder.store_coins(amt).unwrap();
                } else {
                    return Err("Jetton wallet and amount are required for mode 3".into());
                }
            },
            _ => return Err("Invalid collect mode".into()),
        }

        return Ok(mess_builder.build().unwrap()); 
    }
}

/// Creates an external signed message for a TON wallet.
pub fn create_external_singed_message(user_wallet: TonWallet, seqno: u32, destination_address: TonAddress, amount: u64, now: u64, body_payload: Cell) -> Vec<u8> {
    //create external message
    let transfer = TransferMessage::new(
        &destination_address, 
        &BigUint::from(amount)
    ).with_data(body_payload)
        .build()
        .unwrap();

    let msg_arc: Vec<Arc<Cell>> = vec![transfer].into_iter().map(Arc::new).collect();
    let body: Cell = user_wallet.create_external_body(now as u32 + 60, seqno, msg_arc).unwrap();
    let signed: Cell = user_wallet.sign_external_body(&body).unwrap();
    let wrapped: Cell = user_wallet.wrap_signed_body(signed, true).unwrap();
    let boc: BagOfCells = BagOfCells::from_root(wrapped);

    boc.serialize(true).unwrap()
}