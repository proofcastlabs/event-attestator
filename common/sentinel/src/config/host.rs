use common_eth::convert_hex_to_eth_address;
use derive_getters::Getters;
use ethereum_types::Address as EthAddress;
use serde::Deserialize;

use super::SentinelConfigError;
use crate::{config::ConfigT, constants::MILLISECONDS_MULTIPLIER, Endpoints, NetworkId, SentinelError};

#[derive(Debug, Clone, Deserialize)]
pub struct HostToml {
    validate: bool,
    batch_size: u64,
    gas_limit: usize,
    network_id: String,
    batch_duration: u64,
    sleep_duration: u64,
    pnetwork_hub: String,
    endpoints: Vec<String>,
    gas_price: Option<u64>,
    pre_filter_receipts: bool,
}

#[derive(Debug, Clone, Default, Getters)]
pub struct HostConfig {
    validate: bool,
    batch_size: u64,
    gas_limit: usize,
    batch_duration: u64,
    sleep_duration: u64,
    #[getter(skip)]
    endpoints: Endpoints,
    network_id: NetworkId,
    gas_price: Option<u64>,
    pnetwork_hub: EthAddress,
    pre_filter_receipts: bool,
}

impl HostConfig {
    pub fn from_toml(toml: &HostToml) -> Result<Self, SentinelError> {
        let network_id = NetworkId::try_from(&toml.network_id)?;
        let sleep_duration = toml.sleep_duration * MILLISECONDS_MULTIPLIER; // FIXME make this seconds
        let endpoints = Endpoints::new(sleep_duration, network_id, toml.endpoints.clone());
        Ok(Self {
            endpoints,
            network_id,
            sleep_duration,
            validate: toml.validate,
            gas_price: toml.gas_price,
            gas_limit: toml.gas_limit,
            pre_filter_receipts: toml.pre_filter_receipts,
            batch_size: Self::sanity_check_batch_size(toml.batch_size)?,
            pnetwork_hub: convert_hex_to_eth_address(&toml.pnetwork_hub)?,
            batch_duration: Self::sanity_check_batch_duration(toml.batch_duration)?,
        })
    }

    pub fn endpoints(&self) -> Endpoints {
        self.endpoints.clone()
    }

    pub fn get_sleep_duration(&self) -> u64 {
        self.sleep_duration
    }

    fn sanity_check_batch_size(batch_size: u64) -> Result<u64, SentinelError> {
        info!("Sanity checking batch size...");
        const MIN: u64 = 0;
        const MAX: u64 = 1000;
        if batch_size > MIN && batch_size <= MAX {
            Ok(batch_size)
        } else {
            Err(SentinelError::SentinelConfig(SentinelConfigError::BatchSize {
                size: batch_size,
                min: MIN,
                max: MAX,
            }))
        }
    }

    fn sanity_check_batch_duration(batch_duration: u64) -> Result<u64, SentinelError> {
        info!("Sanity checking batch duration...");
        // NOTE: A batch duration of 0 means we submit material one at a time...
        const MAX: u64 = 60 * 10; // NOTE: Ten mins
        if batch_duration <= MAX {
            Ok(batch_duration)
        } else {
            Err(SentinelError::SentinelConfig(SentinelConfigError::BatchDuration {
                max: MAX,
                size: batch_duration,
            }))
        }
    }
}

impl ConfigT for HostConfig {
    fn is_validating(&self) -> bool {
        self.validate
    }

    fn gas_price(&self) -> Option<u64> {
        self.gas_price
    }

    fn gas_limit(&self) -> usize {
        self.gas_limit
    }

    fn pnetwork_hub(&self) -> EthAddress {
        self.pnetwork_hub
    }

    fn pre_filter_receipts(&self) -> bool {
        *self.pre_filter_receipts()
    }
}
