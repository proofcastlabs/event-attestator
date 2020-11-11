use std::str::FromStr;
use crate::{
    constants::SAFE_BTC_ADDRESS,
    btc_on_eth::btc::minting_params::BtcOnEthMintingParams,
    types::{
        Bytes,
        Result,
    },
};
use bitcoin::{
    blockdata::block::Block as BtcBlock,
    hashes::sha256d,
    util::address::Address as BtcAddress,
};

pub use bitcoin::blockdata::transaction::Transaction as BtcTransaction;

pub type BtcTransactions = Vec<BtcTransaction>;
pub type BtcRecipientsAndAmounts = Vec<BtcRecipientAndAmount>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcRecipientAndAmount {
    pub amount: u64,
    pub recipient: BtcAddress,
}

impl BtcRecipientAndAmount {
    pub fn new(recipient: &str, amount: u64) -> Result<Self> {
        Ok(BtcRecipientAndAmount {
            amount,
            recipient: match BtcAddress::from_str(recipient) {
                Ok(address) => address,
                Err(error) => {
                    info!("✔ Error parsing BTC address for recipient: {}", error);
                    info!("✔ Defaulting to SAFE BTC address: {}", SAFE_BTC_ADDRESS,);
                    BtcAddress::from_str(SAFE_BTC_ADDRESS)?
                }
            },
        })
    }
}

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
