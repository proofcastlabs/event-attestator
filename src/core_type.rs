#![allow(dead_code)]
use std::fmt;

use serde::Serialize;
use strum_macros::EnumIter;

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
    fn get_host_symbol(&self) -> String {
        self.to_string().split('_').collect::<Vec<_>>()[2].into()
    }

    fn get_native_symbol(&self) -> String {
        self.to_string().split('_').collect::<Vec<_>>()[0].into()
    }

    pub fn as_db_key_prefix(&self) -> String {
        self.to_string().to_lowercase().replace('_', "-")
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
