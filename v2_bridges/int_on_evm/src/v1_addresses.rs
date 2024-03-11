use std::fmt;

use common::AppError;
use common_eth::convert_hex_to_eth_address;
use common_metadata::MetadataChainId;
use ethereum_types::Address as EthAddress;

// NOTE: Some v1 token contracts on ETH are NOT upgradeable and thus require special handling.
lazy_static! {
    pub static ref PTLOS_ADDRESS: EthAddress =
        convert_hex_to_eth_address("0x7825e833d495f3d1c28872415a4aee339d26ac88").expect("this not to fail");
    pub static ref PLTC_ADDRESS: EthAddress =
        convert_hex_to_eth_address("0x5979f50f1d4c08f9a53863c2f39a7b0492c38d0f").expect("this not to fail");
    pub static ref PIQ_ADDRESS: EthAddress =
        convert_hex_to_eth_address("0xa23d33d5e0a61ba81919bfd727c671bb03ab0fea").expect("this not to fail");
}

#[allow(clippy::upper_case_acronyms)]
pub enum V1Addresses {
    PIQ,
    PLTC,
    PTLOS,
}

impl V1Addresses {
    pub fn to_origin_chain_id(&self) -> MetadataChainId {
        MetadataChainId::EthereumMainnet
    }

    pub fn to_destination_chain_id(&self) -> MetadataChainId {
        // NOTE: Since the tokens herein are not upgradeable, there is no destination chain ID in
        // the redeem events they fire, and so we have to have them hardcoded here instead. This
        // also means that such tokens cannot engage in host to host transfers from ETH mainnet.
        match self {
            Self::PIQ => MetadataChainId::EosMainnet,
            Self::PTLOS => MetadataChainId::TelosMainnet,
            Self::PLTC => MetadataChainId::LitecoinMainnet,
        }
    }
}

impl TryFrom<&EthAddress> for V1Addresses {
    type Error = AppError;

    fn try_from(e: &EthAddress) -> Result<Self, Self::Error> {
        if e == &*PTLOS_ADDRESS {
            Ok(Self::PTLOS)
        } else if e == &*PLTC_ADDRESS {
            Ok(Self::PLTC)
        } else if e == &*PIQ_ADDRESS {
            Ok(Self::PIQ)
        } else {
            Err("could not convert address {e} to v1 token address".into())
        }
    }
}

impl fmt::Display for V1Addresses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::PIQ => "pIQ",
            Self::PLTC => "pLTC",
            Self::PTLOS => "pTLOS",
        };
        write!(f, "{s}")
    }
}
