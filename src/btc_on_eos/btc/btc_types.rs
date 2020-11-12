use crate::{
    btc_on_eos::btc::minting_params::BtcOnEosMintingParams,
    types::{
        Bytes,
        Result,
    },
    chains::btc::{
        deposit_address_info::DepositAddressInfoJsonList,
        btc_types::{
            BtcBlockAndId,
            BtcBlockJson,
        },
    },
};
use bitcoin::{
    hashes::sha256d,
    blockdata::block::Block as BtcBlock,
};

pub use bitcoin::blockdata::transaction::Transaction as BtcTransaction;

pub type BtcTransactions = Vec<BtcTransaction>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmissionMaterial {
    pub ref_block_num: u16,
    pub ref_block_prefix: u32,
    pub block_and_id: BtcBlockAndId,
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
