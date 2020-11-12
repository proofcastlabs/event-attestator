use crate::{
    btc_on_eth::btc::minting_params::BtcOnEthMintingParams,
    types::{
        Bytes,
        Result,
    },
};
use bitcoin::{
    blockdata::block::Block as BtcBlock,
    hashes::sha256d,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcBlockInDbFormat {
    pub height: u64,
    pub block: BtcBlock,
    pub id: sha256d::Hash,
    pub extra_data: Bytes,
    pub minting_params: BtcOnEthMintingParams,
}

impl BtcBlockInDbFormat {
    pub fn new(
        height: u64,
        id: sha256d::Hash,
        minting_params: BtcOnEthMintingParams,
        block: BtcBlock,
        extra_data: Bytes,
    ) -> Result<Self> {
        Ok(BtcBlockInDbFormat {
            id,
            block,
            height,
            minting_params,
            extra_data,
        })
    }
}
