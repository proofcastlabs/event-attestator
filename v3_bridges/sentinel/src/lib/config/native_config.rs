use std::{result::Result, str::FromStr};

use common::BridgeSide;
use common_chain_ids::EthChainId;
use ethereum_types::Address as EthAddress;
use serde::Deserialize;

use crate::{
    config::{ConfigT, ContractInfoToml, ContractInfos},
    constants::MILLISECONDS_MULTIPLIER,
    Endpoints,
    SentinelError,
};

#[derive(Debug, Clone, Deserialize)]
pub struct NativeToml {
    validate: bool,
    sleep_duration: u64,
    eth_chain_id: String,
    endpoints: Vec<String>,
    contract_info: Vec<ContractInfoToml>,
}

#[derive(Debug, Clone, Default)]
pub struct NativeConfig {
    validate: bool,
    sleep_duration: u64,
    endpoints: Endpoints,
    eth_chain_id: EthChainId,
    contract_infos: ContractInfos,
}

impl NativeConfig {
    pub fn from_toml(toml: &NativeToml) -> Result<Self, SentinelError> {
        let sleep_duration = toml.sleep_duration * MILLISECONDS_MULTIPLIER;
        Ok(Self {
            sleep_duration,
            validate: toml.validate,
            endpoints: Endpoints::new(false, sleep_duration, toml.endpoints.clone()),
            contract_infos: ContractInfos::from_tomls(&toml.contract_info)?,
            eth_chain_id: match EthChainId::from_str(&toml.eth_chain_id) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Could not parse `eth_chain_id` from native config, defaulting to ETH mainnet!");
                    warn!("{e}");
                    EthChainId::Mainnet
                },
            },
        })
    }

    pub fn get_first_endpoint(&self) -> Result<String, SentinelError> {
        info!("Getting first native endpoint");
        self.endpoints.get_first_endpoint()
    }

    pub fn get_endpoints(&self) -> Endpoints {
        info!("Getting native endpoints!");
        self.endpoints.clone()
    }

    pub fn get_sleep_duration(&self) -> u64 {
        self.sleep_duration
    }

    pub fn get_eth_chain_id(&self) -> EthChainId {
        self.eth_chain_id.clone()
    }
}

impl ConfigT for NativeConfig {
    fn get_contract_addresses(&self) -> Vec<EthAddress> {
        self.contract_infos.get_addresses()
    }

    fn side(&self) -> BridgeSide {
        BridgeSide::Native
    }

    fn is_validating(&self) -> bool {
        self.validate
    }
}

#[cfg(test)]
impl NativeConfig {
    pub fn set_contract_infos(&mut self, infos: ContractInfos) {
        self.contract_infos = infos;
    }
}
