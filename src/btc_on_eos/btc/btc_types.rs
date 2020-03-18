use eos_primitives::{
    Checksum256,
    AccountName as EosAccountName,
};
use std::{
    str::FromStr,
    collections::HashMap,
};
use crate::btc_on_eos::{
    constants::{
        SAFE_BTC_ADDRESS,
        EOS_TOKEN_TICKER,
    },
    utils::{
        strip_hex_prefix,
        convert_hex_to_address,
    },
    types::{
        Bytes,
        Result,
    },
    btc::{
        btc_utils::{
            serialize_btc_utxo,
            deserialize_btc_utxo,
        },
    },
};
use bitcoin::{
    util::address::Address as BtcAddress,
    hashes::{
        Hash,
        sha256d,
    },
    blockdata::{
        block::Block as BtcBlock,
        transaction::{
            TxIn as BtcUtxo,
            Transaction as BtcTransaction,
        },
    },
};

pub type BtcUtxos = Vec<BtcUtxo>;
pub type BtcSignature = [u8; 65];
pub type BtcTransactions = Vec<BtcTransaction>;
pub type MintingParams = Vec<MintingParamStruct>;
pub type BtcUtxosAndValues = Vec<BtcUtxoAndValue>;
pub type DepositInfoList = Vec<DepositAddressInfo>;
pub type BtcRecipientsAndAmounts = Vec<BtcRecipientAndAmount>;
pub type DepositAddressJsonList = Vec<DepositAddressInfoJson>;
pub type DepositInfoHashMap =  HashMap<BtcAddress, DepositAddressInfo>;

#[derive(Debug, Serialize, Deserialize)]
pub struct BtcTxInfo {
    pub btc_tx_hex: String,
    pub btc_tx_amount: u64,
    pub btc_tx_hash: String,
    pub signature_timestamp: u64,
    pub btc_account_nonce: u64,
    pub btc_tx_recipient: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcRecipientAndAmount {
    pub amount: u64,
    pub recipient: BtcAddress,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmissionMaterial {
    pub ref_block_num: u16,
    pub ref_block_prefix: u32,
    pub block_and_id: BtcBlockAndId,
}

impl BtcRecipientAndAmount {
    pub fn new(recipient: &str, amount: u64) -> Result<Self> {
        Ok(
            BtcRecipientAndAmount {
                amount,
                recipient: match BtcAddress::from_str(recipient) {
                    Ok(address) => address,
                    Err(error) => {
                        info!(
                            "✔ Error parsing BTC address for recipient: {}",
                            error
                        );
                        info!(
                            "✔ Defaulting to SAFE BTC address: {}",
                            SAFE_BTC_ADDRESS,
                        );
                        BtcAddress::from_str(SAFE_BTC_ADDRESS)?
                    }
                }
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcBlockInDbFormat {
    pub height: u64,
    pub block: BtcBlock,
    pub id: sha256d::Hash,
    pub extra_data: Bytes,
    pub minting_params: MintingParams,
}

impl BtcBlockInDbFormat {
    pub fn new(
        height: u64,
        id: sha256d::Hash,
        minting_params: MintingParams,
        block: BtcBlock,
        extra_data: Bytes,
    ) -> Result<Self> {
        Ok(BtcBlockInDbFormat{ id, block, height, minting_params, extra_data })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MintingParamStruct {
    pub amount: String,
    pub to: String,
    pub originating_tx_hash: String,
    pub originating_tx_address: String,
}

impl MintingParamStruct {
    pub fn new(
        amount: u64,
        to: String,
        originating_tx_hash: sha256d::Hash,
        originating_tx_address: BtcAddress,
    ) -> MintingParamStruct {
        MintingParamStruct {
            to,
            amount: format!("{} {}", amount, EOS_TOKEN_TICKER),
            originating_tx_hash: originating_tx_hash.to_string(),
            originating_tx_address: originating_tx_address.to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SubmissionMaterialJson {
    pub ref_block_num: u16,
    pub block: BtcBlockJson,
    pub ref_block_prefix: u32,
    pub transactions: Vec<String>,
    pub deposit_address_list: DepositAddressJsonList,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BtcBlockJson {
    pub bits: u32,
    pub id: String,
    pub nonce: u32,
    pub version: u32,
    pub height: u64,
    pub timestamp: u32,
    pub merkle_root: String,
    pub previousblockhash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepositAddressInfo {
    pub nonce: u64,
    pub eos_address: String,
    pub commitment_hash: sha256d::Hash,
    pub btc_deposit_address: BtcAddress,
}

impl DepositAddressInfo {
    pub fn new(
        nonce: &u64,
        eos_address: &String,
        btc_deposit_address: &String,
        commitment_hash: &String,
    ) -> Result<Self> {
        Ok(
            DepositAddressInfo {
                nonce: *nonce,
                eos_address: eos_address.to_string(),
                btc_deposit_address:
                    BtcAddress::from_str(&btc_deposit_address)?,
                commitment_hash: sha256d::Hash::from_slice(
                    &hex::decode(strip_hex_prefix(commitment_hash)?)?
                )?,
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositAddressInfoJson {
    pub nonce: u64,
    pub address: String,
    pub btc_deposit_address: String,
    pub address_and_nonce_hash: String,
}

impl DepositAddressInfoJson {
    pub fn new(
        nonce: u64,
        address: String,
        btc_deposit_address: String,
        address_and_nonce_hash: String,
    ) -> Self {
        DepositAddressInfoJson {
            nonce,
            address,
            btc_deposit_address,
            address_and_nonce_hash,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcBlockAndId {
    pub height: u64,
    pub block: BtcBlock,
    pub id: sha256d::Hash,
    pub deposit_address_list: DepositInfoList,
}

impl BtcBlockAndId {
    pub fn new(
        height: u64,
        block: BtcBlock,
        id: sha256d::Hash,
        deposit_address_list: DepositInfoList
    ) -> Self {
        BtcBlockAndId { id, block, height, deposit_address_list }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BtcUtxoAndValue {
    pub value: u64,
    pub serialized_utxo: Bytes,
    pub maybe_extra_data: Option<Bytes>,
    pub maybe_pointer: Option<sha256d::Hash>,
    pub maybe_deposit_info_json: Option<DepositAddressInfoJson>,
}

impl BtcUtxoAndValue {
    pub fn new(
        value: u64,
        utxo: &BtcUtxo,
        maybe_deposit_info_json: Option<DepositAddressInfoJson>,
        maybe_extra_data: Option<Bytes>,
    ) -> Self {
        BtcUtxoAndValue {
            value,
            maybe_extra_data,
            maybe_pointer: None,
            maybe_deposit_info_json,
            serialized_utxo: serialize_btc_utxo(utxo),
        }
    }

    pub fn get_utxo(&self) -> Result<BtcUtxo> {
        deserialize_btc_utxo(&self.serialized_utxo)
    }

    pub fn update_pointer(mut self, hash: sha256d::Hash) -> Self {
        self.maybe_pointer = Some(hash);
        self
    }
}
