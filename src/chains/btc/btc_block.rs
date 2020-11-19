use std::str::FromStr;
use derive_more::Constructor;
use crate::{
    traits::DatabaseInterface,
    btc_on_eth::btc::minting_params::BtcOnEthMintingParams,
    btc_on_eos::btc::minting_params::BtcOnEosMintingParams,
    types::{
        Byte,
        Bytes,
        Result,
    },
    utils::{
        convert_u64_to_bytes,
        convert_bytes_to_u64,
    },
    chains::btc::{
        btc_state::BtcState,
        deposit_address_info::DepositInfoList,
        btc_submission_material::BtcSubmissionMaterialJson,
    },
};
pub use bitcoin::{
    util::address::Address as BtcAddress,
    hashes::{
        Hash,
        sha256d,
    },
    consensus::encode::{
        serialize as btc_serialize,
        deserialize as btc_deserialize,
    },
    blockdata::{
        block::Block as BtcBlock,
        block::BlockHeader as BtcBlockHeader,
        transaction::Transaction as BtcTransaction,
    },
};

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
        Ok(BtcBlockInDbFormat{ id, block, height, extra_data, eth_minting_params, eos_minting_params })
    }

    pub fn get_eos_minting_params(&self) -> BtcOnEosMintingParams {
        self.eos_minting_params.clone().unwrap_or_else(|| BtcOnEosMintingParams::new(vec![]))
    }

    pub fn get_eth_minting_params(&self) -> BtcOnEthMintingParams {
        self.eth_minting_params.clone().unwrap_or_else(|| BtcOnEthMintingParams::new(vec![]))
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

    pub fn to_bytes(&self) -> Result<(Bytes, Bytes)> { // FIXME Rm the tuple!
        let serialized_id = self.id.to_vec();
        Ok(
            (
                serialized_id.clone(),
                serde_json::to_vec(
                    &SerializedBlockInDbFormat::new(
                        serialized_id,
                        btc_serialize(&self.block),
                        convert_u64_to_bytes(self.height),
                        self.extra_data.clone(),
                        self.get_eth_minting_param_bytes()?,
                        self.get_eos_minting_param_bytes()?,
                    )
                )?
            )
        )
    }

    pub fn from_bytes(serialized_block_in_db_format: &[Byte]) -> Result<BtcBlockInDbFormat> {
        let serialized_struct: SerializedBlockInDbFormat = serde_json::from_slice(&serialized_block_in_db_format)?;
        BtcBlockInDbFormat::new(
            convert_bytes_to_u64(&serialized_struct.height)?,
            sha256d::Hash::from_slice(&serialized_struct.id)?,
            btc_deserialize(&serialized_struct.block)?,
            serialized_struct.extra_data.clone(),
            serialized_struct.get_btc_on_eos_minting_params()?,
            serialized_struct.get_btc_on_eth_minting_params()?,
        )
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
        let bytes = self.eos_minting_params.clone().unwrap_or_default();
        if bytes.is_empty() { Ok(None) } else { Ok(Some(BtcOnEosMintingParams::from_bytes(&bytes)?)) }
    }

    pub fn get_btc_on_eth_minting_params(&self) -> Result<Option<BtcOnEthMintingParams>> {
        let params = BtcOnEthMintingParams::from_bytes(&self.minting_params)?;
        if params.is_empty() { Ok(None) } else { Ok(Some(params)) }
    }
}

pub fn parse_btc_block_and_id_and_put_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    BtcBlockAndId::from_json(state.get_btc_submission_json()?).and_then(|block| state.add_btc_block_and_id(block))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::{
        consensus::encode::deserialize as btc_deserialize,
        blockdata::transaction::Transaction as BtcTransaction,
    };
    use crate::chains::btc::btc_test_utils::{
        get_sample_btc_block_in_db_format,
        get_sample_btc_submission_material_json,
    };

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

    #[test]
    fn should_serde_btc_block_in_db_format() {
        let block = get_sample_btc_block_in_db_format().unwrap();
        let (_db_key, serialized_block) = block.to_bytes().unwrap();
        let result = BtcBlockInDbFormat::from_bytes(&serialized_block).unwrap();
        assert_eq!(result, block);
    }
}
