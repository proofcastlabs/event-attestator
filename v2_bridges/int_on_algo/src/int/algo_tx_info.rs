use std::{fmt, str::FromStr};

use common::{
    chains::eth::eth_utils::{convert_eth_address_to_string, convert_eth_hash_to_string},
    metadata::metadata_chain_id::MetadataChainId,
    types::{Byte, Bytes, Result},
    utils::convert_bytes_to_string,
};
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use rust_algorand::{AlgorandAddress, AlgorandAppId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct IntOnAlgoAlgoTxInfo {
    pub user_data: Bytes,
    pub algo_asset_id: u64,
    pub host_token_amount: U256,
    pub token_sender: EthAddress,
    pub native_token_amount: U256,
    pub vault_address: EthAddress,
    pub router_address: EthAddress,
    pub originating_tx_hash: EthHash,
    pub int_token_address: EthAddress,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
    pub issuance_manager_app_id: AlgorandAppId,
    pub destination_app_id: Option<AlgorandAppId>,
    pub destination_address: Option<AlgorandAddress>,
}

// NOTE: These `serdable` versions are because the `AlgorandAddress` uses some custom serialization which breaks things
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntOnAlgoAlgoTxInfoSerdable {
    user_data: Bytes,
    algo_asset_id: u64,
    host_token_amount: U256,
    token_sender: EthAddress,
    native_token_amount: U256,
    vault_address: EthAddress,
    router_address: EthAddress,
    originating_tx_hash: EthHash,
    int_token_address: EthAddress,
    origin_chain_id: MetadataChainId,
    destination_chain_id: MetadataChainId,
    issuance_manager_app_id: AlgorandAppId,
    destination_app_id: Option<AlgorandAppId>,
    destination_address: Option<String>,
}

impl IntOnAlgoAlgoTxInfoSerdable {
    pub fn from_tx_info(info: &IntOnAlgoAlgoTxInfo) -> Self {
        Self {
            user_data: info.user_data.clone(),
            algo_asset_id: info.algo_asset_id,
            host_token_amount: info.host_token_amount,
            token_sender: info.token_sender,
            native_token_amount: info.native_token_amount,
            vault_address: info.vault_address,
            router_address: info.router_address,
            originating_tx_hash: info.originating_tx_hash,
            int_token_address: info.int_token_address,
            origin_chain_id: info.origin_chain_id,
            destination_chain_id: info.destination_chain_id,
            issuance_manager_app_id: info.issuance_manager_app_id.clone(),
            destination_app_id: info.destination_app_id.clone(),
            destination_address: info.destination_address.map(|x| x.to_string()),
        }
    }

    fn to_tx_info(&self) -> Result<IntOnAlgoAlgoTxInfo> {
        Ok(IntOnAlgoAlgoTxInfo {
            user_data: self.user_data.clone(),
            algo_asset_id: self.algo_asset_id,
            host_token_amount: self.host_token_amount,
            token_sender: self.token_sender,
            native_token_amount: self.native_token_amount,
            vault_address: self.vault_address,
            router_address: self.router_address,
            originating_tx_hash: self.originating_tx_hash,
            int_token_address: self.int_token_address,
            origin_chain_id: self.origin_chain_id,
            destination_chain_id: self.destination_chain_id,
            issuance_manager_app_id: self.issuance_manager_app_id.clone(),
            destination_app_id: self.destination_app_id.clone(),
            destination_address: match self.destination_address {
                Some(ref s) => Some(AlgorandAddress::from_str(s)?),
                _ => None,
            },
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Constructor, Deref, Serialize, Deserialize)]
pub struct IntOnAlgoAlgoTxInfosSerdable(pub Vec<IntOnAlgoAlgoTxInfoSerdable>);

impl IntOnAlgoAlgoTxInfosSerdable {
    fn from_tx_infos(tx_infos: &IntOnAlgoAlgoTxInfos) -> Self {
        Self::new(
            tx_infos
                .iter()
                .map(IntOnAlgoAlgoTxInfoSerdable::from_tx_info)
                .collect::<Vec<_>>(),
        )
    }

    fn to_tx_infos(&self) -> Result<IntOnAlgoAlgoTxInfos> {
        Ok(IntOnAlgoAlgoTxInfos::new(
            self.iter()
                .map(IntOnAlgoAlgoTxInfoSerdable::to_tx_info)
                .collect::<Result<Vec<_>>>()?,
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct IntOnAlgoAlgoTxInfos(pub Vec<IntOnAlgoAlgoTxInfo>);

impl IntOnAlgoAlgoTxInfos {
    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        if bytes.is_empty() {
            Ok(Self::default())
        } else {
            serde_json::from_slice::<IntOnAlgoAlgoTxInfosSerdable>(bytes)?.to_tx_infos()
        }
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&IntOnAlgoAlgoTxInfosSerdable::from_tx_infos(self))?)
    }
}

impl IntOnAlgoAlgoTxInfo {
    pub fn destination_is_app(&self) -> bool {
        self.destination_app_id.is_some()
    }

    pub fn get_destination_address(&self) -> Result<AlgorandAddress> {
        match &self.destination_address {
            Some(address) => Ok(*address),
            None => Err("No `destination_address` in `IntOnAlgoAlgoTxInfo` - is it an app peg-in?".into()),
        }
    }

    pub fn get_destination_app_id(&self) -> Result<AlgorandAppId> {
        match &self.destination_app_id {
            Some(app_id) => Ok(app_id.clone()),
            None => Err("No `destination_app_id` in `IntOnAlgoAlgoTxInfo` - is it an address peg-in?".into()),
        }
    }
}

impl fmt::Display for IntOnAlgoAlgoTxInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "
IntOnAlgoAlgoTxInfo: {{
    algo_asset_id: {},
    token_sender: {},
    host_token_amount: {},
    native_token_amount: {},
    vault_address: {},
    router_address: {},
    originating_tx_hash: {},
    int_token_address: {},
    origin_chain_id: {},
    destination_chain_id: {},
    issuance_manager_app_id: {},
    destination_app_id: {},
    destination_address: {},
    user_data: {},
}}
            ",
            self.algo_asset_id,
            convert_eth_address_to_string(&self.token_sender),
            self.host_token_amount,
            self.native_token_amount,
            convert_eth_address_to_string(&self.vault_address),
            convert_eth_address_to_string(&self.router_address),
            convert_eth_hash_to_string(&self.originating_tx_hash),
            convert_eth_address_to_string(&self.int_token_address),
            self.origin_chain_id,
            self.destination_chain_id,
            self.issuance_manager_app_id,
            match self.destination_app_id.as_ref() {
                Some(address) => address.to_string(),
                None => "Cannot unwrap `destination_app_id` option!".to_string(),
            },
            match self.destination_address.as_ref() {
                Some(address) => address.to_string(),
                None => "Cannot unwrap `destination_address` option!".to_string(),
            },
            convert_bytes_to_string(&self.user_data),
        )
    }
}

#[cfg(test)]
mod tests {
    use common::chains::eth::eth_utils::convert_hex_to_eth_address;

    use super::*;

    #[test]
    fn should_serde_tx_info_to_and_from_bytes() {
        let mut info = IntOnAlgoAlgoTxInfo::default();
        info.destination_address = Some(AlgorandAddress::create_random().unwrap());
        info.destination_app_id = Some(AlgorandAppId(1337));
        let infos = IntOnAlgoAlgoTxInfos::new(vec![IntOnAlgoAlgoTxInfo::default()]);
        let bytes = infos.clone().to_bytes().unwrap();
        let result = IntOnAlgoAlgoTxInfos::from_bytes(&bytes).unwrap();
        assert_eq!(result, infos);
    }

    #[test]
    fn should_serde_tx_infos_correctly() {
        let info = IntOnAlgoAlgoTxInfo {
            user_data: vec![222, 202, 255],
            algo_asset_id: 714666072,
            host_token_amount: U256::from(1337),
            token_sender: convert_hex_to_eth_address("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap(),
            native_token_amount: U256::from_dec_str("133700000000").unwrap(),
            vault_address: convert_hex_to_eth_address("e0806ce04978224e27c6bb10e27fd30a7785ae9d").unwrap(),
            router_address: convert_hex_to_eth_address("ec1700a39972482d5db20e73bb3ffe6829b0c102").unwrap(),
            originating_tx_hash: EthHash::from_slice(
                &hex::decode("b81f5564195f022f9812d5dfe80052afdfaf8cc86243a98b0dbdd887ef97bda7").unwrap(),
            ),
            int_token_address: convert_hex_to_eth_address("4262d1f878d191fbc66dca73bad57309916b1412").unwrap(),
            origin_chain_id: MetadataChainId::InterimChain,
            destination_chain_id: MetadataChainId::AlgorandMainnet,
            issuance_manager_app_id: AlgorandAppId(1337),
            destination_app_id: None,
            destination_address: Some(
                AlgorandAddress::from_bytes(&[
                    50, 167, 219, 223, 205, 231, 105, 93, 145, 172, 67, 129, 82, 252, 144, 134, 23, 255, 191, 157, 185,
                    79, 132, 60, 37, 2, 104, 230, 254, 33, 160, 160,
                ])
                .unwrap(),
            ),
        };
        let infos = IntOnAlgoAlgoTxInfos::new(vec![info]);
        let bytes = infos.clone().to_bytes().unwrap();
        let result = IntOnAlgoAlgoTxInfos::from_bytes(&bytes).unwrap();
        assert_eq!(result, infos);
    }
}
