use std::str::FromStr;
use derive_more::Constructor;
use crate::{
    constants::SAFE_BTC_ADDRESS,
    btc_on_eth::btc::minting_params::BtcOnEthMintingParams,
    btc_on_eos::btc::minting_params::BtcOnEosMintingParams,
    types::{
        Bytes,
        Result,
        NoneError,
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
    consensus::encode::deserialize as btc_deserialize,
    blockdata::{
        block::Block as BtcBlock,
        block::BlockHeader as BtcBlockHeader,
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

impl BtcBlockAndId {
    pub fn from_json(json: &BtcSubmissionMaterialJson) -> Result<Self> {
        info!("✔ Parsing `BtcBlockAndId` from json...");
        Ok(Self {
            height: json.block.height,
            block: json.to_btc_block()?,
            id: sha256d::Hash::from_str(&json.block.id)?,
            deposit_address_list: DepositInfoList::from_json(&json.deposit_address_list)?,
        })
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

impl BtcBlockJson {
    pub fn to_block_header(&self) -> Result<BtcBlockHeader> {
        info!("✔ Parsing `BtcBlockJson` to `BtcBlockHeader`...");
        Ok(BtcBlockHeader::new(
            self.timestamp,
            self.bits,
            self.nonce,
            self.version,
            sha256d::Hash::from_str(&self.merkle_root)?,
            sha256d::Hash::from_str(&self.previousblockhash)?,
        ))
    }
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

impl BtcSubmissionMaterialJson {
    fn convert_hex_txs_to_btc_transactions(hex_txs: Vec<String>) -> Result<Vec<BtcTransaction>> { // TODO FIXME Make this a tuple struct & impl on there!
        hex_txs.into_iter().map(Self::convert_hex_tx_to_btc_transaction).collect::<Result<Vec<BtcTransaction>>>()
    }

    fn convert_hex_tx_to_btc_transaction(hex: String) -> Result<BtcTransaction> {
        Ok(btc_deserialize::<BtcTransaction>(&hex::decode(hex)?)?)
    }

    pub fn to_btc_block(&self) -> Result<BtcBlock> {
        info!("✔ Parsing `BtcSubmissionMaterialJson` to `BtcBlock`...");
        Ok(BtcBlock::new(
            self.block.to_block_header()?,
            Self::convert_hex_txs_to_btc_transactions(self.transactions.clone())?,
        ))
    }

    pub fn from_str(string: &str) -> Result<Self> {
        info!("✔ Parsing `BtcSubmissionMaterialJson` from string...");
        match serde_json::from_str(string) {
            Ok(json) => Ok(json),
            Err(err) => Err(err.into())
        }
    }
}

// FIXME This could cause issues now if these aren't here in case of btc-on-eth!! TODO change to options!
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcSubmissionMaterial {
    pub ref_block_num: u16,
    pub ref_block_prefix: u32,
    pub block_and_id: BtcBlockAndId,
}

impl BtcSubmissionMaterial {
    pub fn from_json(json: &BtcSubmissionMaterialJson) -> Result<Self> {
        Ok(Self {
            block_and_id: BtcBlockAndId::from_json(json)?,
            ref_block_num: json.ref_block_num.ok_or(NoneError("No `ref_block_num` in submission material!"))?,
            ref_block_prefix: json.ref_block_prefix.ok_or(NoneError("No `ref_block_prefix` in submission material!"))?,
        })
    }

    pub fn from_str(string: &str) -> Result<Self> {
        BtcSubmissionMaterialJson::from_str(string).and_then(|json| Self::from_json(&json))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::{
        consensus::encode::deserialize as btc_deserialize,
        blockdata::transaction::Transaction as BtcTransaction,
    };
    use crate::chains::btc::btc_test_utils::get_sample_btc_submission_material_json;

    #[test]
    fn should_parse_block_and_tx_json_to_struct() {
        let json = get_sample_btc_submission_material_json().unwrap();
        let result = BtcBlockAndId::from_json(&json);
        assert!(result.is_ok());
    }

    #[test]
    fn should_deserialize_tx() {
        let tx_bytes = hex::decode("0200000000010117c33a062c8d0c2ce104c9988599f6ba382ff9f786ad48519425e39af23da9880000000000feffffff022c920b00000000001976a914be8a09363cd4719b1c05b2703797ca890b718b5088acf980d30d000000001600147448bbdfe47ec14f27c68393e766567ac7c9c77102473044022073fc2b43d5c5f56d7bc92b47a28db989e04988411721db96fb0eea6689fb83ab022034b7ce2729e867962891fec894210d0faf538b971d3ae9059ebb34358209ec9e012102a51b8eb0eb8ef6b2a421fb1aae3d7308e6cdae165b90f78074c2493af98e3612c43b0900").unwrap();
        let result = btc_deserialize::<BtcTransaction>(&tx_bytes);
        assert!(result.is_ok());
    }
}
