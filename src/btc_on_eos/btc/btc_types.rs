use std::str::FromStr;
use crate::{
    btc_on_eos::btc::minting_params::BtcOnEosMintingParams,
    types::{
        Bytes,
        Result,
    },
    constants::SAFE_BTC_ADDRESS,
    chains::btc::deposit_address_info::{
        DepositInfoList,
        DepositAddressInfoJson,
        DepositAddressInfoJsonList,
    },
};
use bitcoin::{
    hashes::sha256d,
    util::address::Address as BtcAddress,
    blockdata::block::Block as BtcBlock,
};

pub use bitcoin::blockdata::transaction::Transaction as BtcTransaction;

pub type BtcTransactions = Vec<BtcTransaction>;
pub type BtcRecipientsAndAmounts = Vec<BtcRecipientAndAmount>;

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
                        info!("✔ Error parsing BTC address for recipient: {}", error);
                        info!("✔ Defaulting to SAFE BTC address: {}", SAFE_BTC_ADDRESS);
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
    pub minting_params: BtcOnEosMintingParams,
}

impl BtcBlockInDbFormat {
    pub fn new(
        height: u64,
        id: sha256d::Hash,
        minting_params: BtcOnEosMintingParams,
        block: BtcBlock,
        extra_data: Bytes,
    ) -> Result<Self> {
        Ok(BtcBlockInDbFormat{ id, block, height, minting_params, extra_data })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SubmissionMaterialJson {
    pub ref_block_num: u16,
    pub block: BtcBlockJson,
    pub ref_block_prefix: u32,
    pub transactions: Vec<String>,
    pub deposit_address_list: DepositAddressInfoJsonList,
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
pub struct BtcBlockAndId {
    pub height: u64,
    pub block: BtcBlock,
    pub id: sha256d::Hash,
    pub deposit_address_list: DepositInfoList,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BtcUtxoAndValue {
    pub value: u64,
    pub serialized_utxo: Bytes,
    pub maybe_extra_data: Option<Bytes>,
    pub maybe_pointer: Option<sha256d::Hash>,
    pub maybe_deposit_info_json: Option<DepositAddressInfoJson>,
}
