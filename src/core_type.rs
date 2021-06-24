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
        match self {
            Self::BtcOnEth => write!(f, "BTC-on-ETH"),
            Self::BtcOnEos => write!(f, "BTC-on-EOS"),
            Self::EosOnEth => write!(f, "EOS-on-ETH"),
            Self::Erc20OnEos => write!(f, "ERC20-on-EOS"),
            Self::Erc20OnEvm => write!(f, "ERC20-on-EVM"),
        }
    }
}
