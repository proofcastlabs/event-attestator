use std::str::FromStr;
use derive_more::Constructor;
use crate::{
    constants::SAFE_BTC_ADDRESS,
    btc_on_eth::btc::minting_params::BtcOnEthMintingParams,
    btc_on_eos::btc::minting_params::BtcOnEosMintingParams,
    types::{
        Bytes,
        Result,
    },
    chains::btc::deposit_address_info::{
        DepositInfoList,
        DepositAddressInfoJson,
        DepositAddressInfoJsonList,
    },
};
pub use bitcoin::{
    hashes::sha256d,
    util::address::Address as BtcAddress,
    blockdata::{
        block::Block as BtcBlock,
        transaction::Transaction as BtcTransaction,
    },
};

pub type BtcTransactions = Vec<BtcTransaction>;

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

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct BtcSubmissionMaterialJson {
    pub block: BtcBlockJson,
    pub any_sender: Option<bool>,
    pub transactions: Vec<String>,
    pub ref_block_num: Option<u16>,
    pub ref_block_prefix: Option<u32>,
    pub deposit_address_list: DepositAddressInfoJsonList,
}

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
    pub eos_minting_params: Option<BtcOnEosMintingParams>,
    pub eth_minting_params: Option<BtcOnEthMintingParams>,
}

impl BtcBlockInDbFormat {
    pub fn new(
        height: u64,
        id: sha256d::Hash,
        block: BtcBlock,
        extra_data: Bytes,
        eos_minting_params: Option<BtcOnEosMintingParams>,
        eth_minting_params: Option<BtcOnEthMintingParams>,
    ) -> Result<Self> {
        Ok(BtcBlockInDbFormat{
            id,
            block,
            height,
            extra_data,
            eth_minting_params: eth_minting_params,
            eos_minting_params: eos_minting_params,
        })
    }

    pub fn get_eos_minting_params(&self) -> BtcOnEosMintingParams {
        self.eos_minting_params.clone().unwrap_or(BtcOnEosMintingParams::new(vec![]))
    }

    pub fn get_eth_minting_params(&self) -> BtcOnEthMintingParams {
        self.eth_minting_params.clone().unwrap_or(BtcOnEthMintingParams::new(vec![]))
    }

    pub fn get_eos_minting_param_bytes(&self) -> Result<Option<Bytes>> {
        // NOTE: This returns the option required for the serialized structure to be backwards compatible.
        if self.eos_minting_params.is_some() {
            Ok(Some(self.get_eos_minting_params().to_bytes()?))
        } else {
            Ok(None)
        }
    }

    pub fn get_eth_minting_param_bytes(&self) -> Result<Bytes> {
        self.get_eth_minting_params().to_bytes()
    }

    pub fn remove_minting_params(&self) -> Result<Self> {
        Self::new(self.height, self.id, self.block.clone(), self.extra_data.clone(), None, None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct SerializedBlockInDbFormat {
    pub id: Bytes,
    pub block: Bytes,
    pub height: Bytes,
    pub extra_data: Bytes,
    pub minting_params: Bytes,
    pub eos_minting_params: Option<Bytes>, // Option ∴ backwards compatible
}

impl SerializedBlockInDbFormat {
    pub fn get_btc_on_eos_minting_params(&self) -> Result<Option<BtcOnEosMintingParams>> {
        let bytes = self.eos_minting_params.clone().unwrap_or(vec![]);
        let empty_bytes: Vec<u8> = vec![];
        if bytes == empty_bytes { Ok(None) } else { Ok(Some(BtcOnEosMintingParams::from_bytes(&bytes)?)) }
    }

    pub fn get_btc_on_eth_minting_params(&self) -> Result<Option<BtcOnEthMintingParams>> {
        let params = BtcOnEthMintingParams::from_bytes(&self.minting_params)?;
        if params.is_empty() { Ok(None) } else { Ok(Some(params)) }
    }
}
