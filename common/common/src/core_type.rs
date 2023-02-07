#![allow(dead_code)]
use std::fmt;

use serde::Serialize;
use strum_macros::EnumIter;

use crate::{
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    traits::DatabaseInterface,
    types::{Bytes, Result},
};

lazy_static! {
    pub static ref CORE_IS_INITIALIZED_MARKER: Bytes = vec![1u8];
    pub static ref HOST_CORE_IS_INITIALIZED_DB_KEY: [u8; 32] =
        crate::utils::get_prefixed_db_key("host_core_is_initialized_db_key");
    pub static ref NATIVE_CORE_IS_INITIALIZED_DB_KEY: [u8; 32] =
        crate::utils::get_prefixed_db_key("native_core_is_initialized_db_key");
}

#[derive(Clone, Copy, EnumIter, Serialize)]
pub enum CoreType {
    BtcOnEth,
    BtcOnInt,
    IntOnEos,
    IntOnEvm,
    EosOnInt,
    BtcOnEos,
    EosOnEth,
    IntOnAlgo,
    Erc20OnEos,
    Erc20OnInt,
    Erc20OnEvm,
}

impl CoreType {
    pub fn initialize_native_core<D: DatabaseInterface>(db: &D) -> Result<()> {
        info!("✔ Initializing NATIVE core...");
        db.put(
            NATIVE_CORE_IS_INITIALIZED_DB_KEY.to_vec(),
            CORE_IS_INITIALIZED_MARKER.clone(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn initialize_host_core<D: DatabaseInterface>(db: &D) -> Result<()> {
        info!("✔ Initializing HOST core...");
        db.put(
            HOST_CORE_IS_INITIALIZED_DB_KEY.to_vec(),
            CORE_IS_INITIALIZED_MARKER.clone(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn get_host_symbol(&self) -> String {
        self.to_string().split('_').collect::<Vec<_>>()[2].into()
    }

    fn get_native_symbol(&self) -> String {
        self.to_string().split('_').collect::<Vec<_>>()[0].into()
    }

    pub fn as_db_key_prefix(&self) -> String {
        self.to_string().to_lowercase().replace('_', "-")
    }

    fn core_is_initialized<D: DatabaseInterface>(db: &D, is_native: bool) -> bool {
        db.get(
            if is_native {
                NATIVE_CORE_IS_INITIALIZED_DB_KEY.to_vec()
            } else {
                HOST_CORE_IS_INITIALIZED_DB_KEY.to_vec()
            },
            MIN_DATA_SENSITIVITY_LEVEL,
        )
        .is_ok()
    }

    pub fn native_core_is_initialized<D: DatabaseInterface>(db: &D) -> bool {
        Self::core_is_initialized(db, true)
    }

    pub fn host_core_is_initialized<D: DatabaseInterface>(db: &D) -> bool {
        Self::core_is_initialized(db, false)
    }

    pub fn check_is_initialized<D: DatabaseInterface>(db: &D) -> Result<()> {
        if !Self::native_core_is_initialized(db) {
            Err("NATIVE side of core is not initialized!".into())
        } else if !Self::host_core_is_initialized(db) {
            Err("HOST side of core is not initialized!".into())
        } else {
            info!("✔ Core is initialized!");
            Ok(())
        }
    }
}

impl Default for CoreType {
    fn default() -> Self {
        Self::BtcOnInt
    }
}

impl fmt::Display for CoreType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::BtcOnEth => "BTC_ON_ETH",
            Self::BtcOnEos => "BTC_ON_EOS",
            Self::EosOnEth => "EOS_ON_ETH",
            Self::BtcOnInt => "BTC_ON_INT",
            Self::IntOnEos => "INT_ON_EOS",
            Self::EosOnInt => "EOS_ON_INT",
            Self::IntOnEvm => "INT_ON_EVM",
            Self::IntOnAlgo => "INT_ON_ALGO",
            Self::Erc20OnEos => "ERC20_ON_EOS",
            Self::Erc20OnEvm => "ERC20_ON_EVM",
            Self::Erc20OnInt => "ERC20_ON_INT",
        };
        write!(f, "{}", s)
    }
}

macro_rules! make_stateful_initialization_checkers {
    ($($chain:ident),*) => {
        paste! {
            $(
                use $crate::state::[< $chain:camel State>];

                impl CoreType {
                    pub fn [< check_core_is_initialized_and_return_ $chain:lower _state > ]<D: DatabaseInterface>(
                        state: [< $chain:camel State >]<D>,
                    ) -> Result<[< $chain:camel State >]<D>> {
                        Self::check_is_initialized(state.[< $chain:lower _db_utils >].get_db()).and(Ok(state))
                    }
                }
            )*
        }
    }
}

make_stateful_initialization_checkers!(Eth, Eos, Btc);

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn should_get_core_type_as_db_key_prefix() {
        let expected_results = vec![
            "btc-on-eth",
            "btc-on-int",
            "int-on-eos",
            "int-on-evm",
            "eos-on-int",
            "btc-on-eos",
            "eos-on-eth",
            "int-on-algo",
            "erc20-on-eos",
            "erc20-on-int",
            "erc20-on-evm",
        ];
        CoreType::iter()
            .zip(expected_results.iter())
            .for_each(|(core_type, expected_result)| assert_eq!(&core_type.as_db_key_prefix(), *expected_result))
    }

    #[test]
    fn should_get_native_symbol_from_core_type() {
        let expected_results = vec![
            "BTC", "BTC", "INT", "INT", "EOS", "BTC", "EOS", "INT", "ERC20", "ERC20", "ERC20",
        ];
        CoreType::iter()
            .zip(expected_results.iter())
            .for_each(|(core_type, expected_result)| assert_eq!(&core_type.get_native_symbol(), *expected_result))
    }

    #[test]
    fn should_get_host_symbol_from_core_type() {
        let expected_results = vec![
            "ETH", "INT", "EOS", "EVM", "INT", "EOS", "ETH", "ALGO", "EOS", "INT", "EVM",
        ];
        CoreType::iter()
            .zip(expected_results.iter())
            .for_each(|(core_type, expected_result)| assert_eq!(&core_type.get_host_symbol(), *expected_result))
    }
}
