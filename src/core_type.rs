#![allow(dead_code)]
use std::fmt;

pub enum CoreType {
    BtcOnEth,
    BtcOnEos,
    EosOnEth,
    Erc20OnEos,
    Erc20OnEvm,
}

impl fmt::Display for CoreType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::BtcOnEth => "BTC_ON_ETH",
            Self::BtcOnEos => "BTC_ON_EOS",
            Self::EosOnEth => "EOS_ON_ETH",
            Self::Erc20OnEos => "ERC20_ON_EOS",
            Self::Erc20OnEvm => "ERC20_ON_EVM",
        };
        write!(f, "{}", s)
    }
}
