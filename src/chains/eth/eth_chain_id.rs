use std::fmt;
use crate::{
    types::{
        Byte,
        Result,
    },
    errors::AppError,
};

#[derive(Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_mainnet_str_to_ethereum_chain_id_correctly() {
        let network_str = "Mainnet";
        let expected_result = EthereumChainId::Mainnet;
        let result = EthereumChainId::from_str(network_str).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_kovan_str_to_ethereum_chain_id_correctly() {
        let network_str = "kOvAN";
        let expected_result = EthereumChainId::Kovan;
        let result = EthereumChainId::from_str(network_str).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_ropsten_str_to_ethereum_chain_id_correctly() {
        let network_str = "ROPSTEN";
        let expected_result = EthereumChainId::Ropsten;
        let result = EthereumChainId::from_str(network_str).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_goerli_str_to_ethereum_chain_id_correctly() {
        let network_str = "goerli";
        let expected_result = EthereumChainId::Goerli;
        let result = EthereumChainId::from_str(network_str).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_rinkeby_str_to_ethereum_chain_id_correctly() {
        let network_str = "rinkeby";
        let expected_result = EthereumChainId::Rinkeby;
        let result = EthereumChainId::from_str(network_str).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_convert_unknown_network_correctly() {
        let network_str = "some other network";
        let expected_err = format!("✘ Unrecognized ethereum network: '{}'!", network_str);
        match EthereumChainId::from_str(network_str) {
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Err(err) => panic!("Wrong error received: {}", err),
            Ok(_) => panic!("Should not have succeeded!"),
        }
    }

    #[test]
    fn should_convert_mainnet_to_correct_chain_id() {
        let eth_network = EthereumChainId::Mainnet;
        let expected_result = 1;
        let result = eth_network.to_chain_id();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_rinekby_to_correct_chain_id() {
        let eth_network = EthereumChainId::Rinkeby;
        let expected_result = 4;
        let result = eth_network.to_chain_id();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_ropsten_to_correct_chain_id() {
        let eth_network = EthereumChainId::Ropsten;
        let expected_result = 3;
        let result = eth_network.to_chain_id();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_goerli_to_correct_chain_id() {
        let eth_network = EthereumChainId::Goerli;
        let expected_result = 5;
        let result = eth_network.to_chain_id();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_kovan_to_correct_chain_id() {
        let eth_network = EthereumChainId::Kovan;
        let expected_result = 42;
        let result = eth_network.to_chain_id();
        assert_eq!(result, expected_result);
    }
}
