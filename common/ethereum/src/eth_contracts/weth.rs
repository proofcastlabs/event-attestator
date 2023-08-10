use common::AppError;
use common_chain_ids::EthChainId;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, U256};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    convert_h256_to_eth_address,
    convert_hex_to_eth_address,
    EthLog,
    EthLogExt,
    EthReceipt,
    EthSubmissionMaterial,
};

#[derive(Debug, Clone, EnumIter)]
enum WethAddresses {
    EthereumMainnet,
}

impl Default for WethAddresses {
    fn default() -> Self {
        Self::EthereumMainnet
    }
}

#[allow(clippy::from_over_into)]
impl Into<Vec<EthAddress>> for WethAddresses {
    fn into(self) -> Vec<EthAddress> {
        Self::iter().map(Self::into).collect()
    }
}

#[allow(clippy::from_over_into)]
impl Into<EthAddress> for WethAddresses {
    fn into(self) -> EthAddress {
        match self {
            Self::EthereumMainnet => {
                convert_hex_to_eth_address("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").expect("this not to fail")
            },
        }
    }
}

impl TryFrom<&EthChainId> for WethAddresses {
    type Error = AppError;

    fn try_from(id: &EthChainId) -> Result<Self, Self::Error> {
        match id {
            EthChainId::Mainnet => Ok(Self::EthereumMainnet),
            _ => Err(format!("cannot get weth address from eth chain id {id}").into()),
        }
    }
}

#[derive(Clone, Debug, Default, Deref)]
struct WethDepositEvents(Vec<WethDepositEvent>);

crate::make_topics!(WETH_DEPOSIT_TOPIC => "e1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c");

#[derive(Clone, Default, Debug, PartialEq, Eq, Constructor)]
struct WethDepositEvent {
    wad: U256,
    to: EthAddress,
}

impl TryFrom<&EthLog> for WethDepositEvent {
    type Error = AppError;

    fn try_from(log: &EthLog) -> Result<Self, Self::Error> {
        if log.topics.len() != 2 {
            Err("eth log does not contain enough topics to be a wETH deposit event".into())
        } else if log.topics[0] != *WETH_DEPOSIT_TOPIC {
            Err("cannot convert non wETH deposit event into a wETH deposit event".into())
        } else {
            Ok(Self::new(
                U256::from_big_endian(&log.data),
                convert_h256_to_eth_address(&log.topics[1]),
            ))
        }
    }
}

impl From<&EthReceipt> for WethDepositEvents {
    fn from(receipt: &EthReceipt) -> Self {
        Self(
            receipt
            .logs
            .iter()
            .filter(|log| log.contains_topic(&WETH_DEPOSIT_TOPIC))
            .map(WethDepositEvent::try_from)
            .filter_map(Result::ok) // NOTE: Because we've filtered above, we can safely ignore errors here
            .collect(),
        )
    }
}

impl From<&EthSubmissionMaterial> for WethDepositEvents {
    fn from(m: &EthSubmissionMaterial) -> Self {
        Self(m.receipts.iter().map(|r| Self::from(r).0).collect::<Vec<_>>().concat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eth_contracts::test_utils::get_sample_submission_material_with_weth_deposit;

    #[test]
    fn should_get_weth_deposit_events_from_eth_submission_material() {
        let m = get_sample_submission_material_with_weth_deposit();
        let r = WethDepositEvents::from(&m);
        let n = r.len();
        let expected_n = 28;
        assert_eq!(n, expected_n);
        let wad = U256::from_dec_str("10000000000000000").unwrap();
        let address = convert_hex_to_eth_address("0xe396757EC7E6aC7C8E5ABE7285dde47b98F22db8").unwrap();
        let expected_event = WethDepositEvent::new(wad, address);
        assert!(r.contains(&expected_event));
    }

    #[test]
    fn should_convert_weth_addresses_enum_to_eth_addresses() {
        let r: Vec<EthAddress> = WethAddresses::default().into();
        let expected_address = convert_hex_to_eth_address("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap();
        assert!(r.contains(&expected_address))
    }
}
