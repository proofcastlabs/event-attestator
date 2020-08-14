use std::fmt;
use crate::{
    types::{
        Byte,
        Result,
    },
    errors::AppError,
};

pub enum EthereumChainId {
    Kovan,
    Goerli,
    Mainnet,
    Rinkeby,
    Ropsten,
}

impl EthereumChainId {
    pub fn from_chain_id(int: &u8) -> Result<Self> {
        match int {
            1 => Ok(EthereumChainId::Mainnet),
            3 => Ok(EthereumChainId::Ropsten),
            4 => Ok(EthereumChainId::Rinkeby),
            5 => Ok(EthereumChainId::Goerli),
            42 => Ok(EthereumChainId::Kovan),
            _ => Err(AppError::Custom(format!("✘ Unrecognised chain id: '{}'!", int)))
        }
    }

    pub fn to_byte(&self) -> Byte {
        self.to_chain_id() as u8
    }

    pub fn to_chain_id(&self) -> u8 {
        match self {
            EthereumChainId::Mainnet => 1,
            EthereumChainId::Ropsten => 3,
            EthereumChainId::Rinkeby => 4,
            EthereumChainId::Goerli => 5,
            EthereumChainId::Kovan => 42,
        }
    }

    pub fn from_str(network_str: &str) -> Result<Self> {
        let lowercase_network_str: &str = &network_str.to_lowercase();
        match lowercase_network_str {
            "mainnet" => EthereumChainId::from_chain_id(&1),
            "ropsten" => EthereumChainId::from_chain_id(&3),
            "rinkeby" => EthereumChainId::from_chain_id(&4),
            "goerli"  => EthereumChainId::from_chain_id(&5),
            "kovan"   => EthereumChainId::from_chain_id(&42),
            _ => Err(AppError::Custom(format!("✘ Unrecognized ethereum network: '{}'!", network_str))),
        }
    }
}

impl fmt::Display for EthereumChainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EthereumChainId::Mainnet => write!(f, "Mainnet"),
            EthereumChainId::Kovan => write!(f, "Kovan Testnet"),
            EthereumChainId::Goerli => write!(f, "Goerli Testnet"),
            EthereumChainId::Ropsten => write!(f, "Ropsten Testnet"),
            EthereumChainId::Rinkeby => write!(f, "Rinkeby Testnet"),
        }
    }
}
