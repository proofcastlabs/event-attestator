use std::{
    str::FromStr,
    collections::HashMap,
};
use crate::{
    types::Result,
    utils::strip_hex_prefix,
};
use bitcoin::{
    util::address::Address as BtcAddress,
    hashes::{
        Hash,
        sha256d,
    },
};

pub type DepositInfoList = Vec<DepositAddressInfo>;
pub type DepositAddressJsonList = Vec<DepositAddressInfoJson>;
pub type DepositInfoHashMap =  HashMap<BtcAddress, DepositAddressInfo>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositAddressInfoJson {
    pub nonce: u64,
    pub address: String,
    pub btc_deposit_address: String,
    pub address_and_nonce_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepositAddressInfo {
    pub nonce: u64,
    pub address: String,
    pub commitment_hash: sha256d::Hash,
    pub btc_deposit_address: BtcAddress,
}

impl DepositAddressInfo {
    pub fn new( // Make this new_eos_info or something
        nonce: u64,
        address: &String,
        btc_deposit_address: &String,
        commitment_hash: &String,
    ) -> Result<Self> {
        Ok(
            DepositAddressInfo {
                nonce,
                address: address.to_string(),
                btc_deposit_address:
                    BtcAddress::from_str(&btc_deposit_address)?,
                commitment_hash: sha256d::Hash::from_slice(
                    &hex::decode(strip_hex_prefix(commitment_hash)?)?
                )?,
            }
        )
    }
}


