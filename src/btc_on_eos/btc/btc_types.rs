use crate::{
    btc_on_eos::btc::minting_params::BtcOnEosMintingParams,
    types::{
        Bytes,
        Result,
    },
};
use bitcoin::{
    hashes::sha256d,
    blockdata::block::Block as BtcBlock,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcBlockInDbFormat {
    pub height: u64,
    pub block: BtcBlock,
    pub id: sha256d::Hash,
    pub extra_data: Bytes,
    pub eos_minting_params: Option<BtcOnEosMintingParams>,
}

impl BtcBlockInDbFormat {
    pub fn new(
        height: u64,
        id: sha256d::Hash,
        eos_minting_params: BtcOnEosMintingParams,
        block: BtcBlock,
        extra_data: Bytes,
    ) -> Result<Self> {
        Ok(BtcBlockInDbFormat{ id, block, height, extra_data, eos_minting_params: Some(eos_minting_params) })
    }

    pub fn get_eos_minting_params(&self) -> BtcOnEosMintingParams {
        self.eos_minting_params.clone().unwrap_or(BtcOnEosMintingParams::new(vec![]))
    }
}
