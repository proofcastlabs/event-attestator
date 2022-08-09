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
}
