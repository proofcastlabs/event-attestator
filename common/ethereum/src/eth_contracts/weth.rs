use std::fmt;

use common::AppError;
use common_chain_ids::EthChainId;
use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, U256};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    convert_h256_to_eth_address,
    convert_hex_to_eth_address,
    Erc20TokenTransferEvent,
    EthLog,
    EthReceipt,
    EthSubmissionMaterial,
};

pub trait ToWethDepositEvent {
    fn to_weth_deposit_event(&self) -> WethDepositEvent;
}

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

impl From<&Erc20TokenTransferEvent> for WethDepositEvent {
    fn from(e: &Erc20TokenTransferEvent) -> Self {
        Self::new(e.value, e.to)
    }
}

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct WethDepositEvents(Vec<WethDepositEvent>);

crate::make_topics!(WETH_DEPOSIT_TOPIC => "e1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c");

#[derive(Clone, Default, Debug, PartialEq, Eq, Constructor)]
pub struct WethDepositEvent {
    wad: U256,
    to: EthAddress,
}

impl fmt::Display for WethDepositEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WethDepositEvent {{ to: {}, wad: {}}}", self.to, self.wad)
    }
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

impl WethDepositEvents {
    fn from_receipt(receipt: &EthReceipt, cid: &EthChainId) -> Result<Self, AppError> {
        let weth_address = WethAddresses::try_from(cid)?.into();
        Ok(Self(
            receipt
            .logs
            .iter()
            .filter(|log| log.is_from_address_and_contains_topic(&weth_address, &WETH_DEPOSIT_TOPIC))
            .map(WethDepositEvent::try_from)
            .filter_map(Result::ok) // NOTE: Because we've filtered above, we can safely ignore errors here
            .collect(),
        ))
    }

    pub fn from_submission_material(m: &EthSubmissionMaterial, cid: &EthChainId) -> Self {
        Self(
            m.receipts
                .iter()
                // NOTE we don't care about errors here, they must just mean it's not a WETH deposit
                .filter_map(|r| WethDepositEvents::from_receipt(r, cid).ok())
                .map(|x| x.0)
                .collect::<Vec<_>>()
                .concat(),
        )
    }

    fn remove(&mut self, event_to_remove: &WethDepositEvent) {
        if let Some(index) = self.iter().position(|event| event == event_to_remove) {
            // NOTE: We don't care about ordering here!
            self.swap_remove(index);
        };
    }

    pub fn filter_if_no_deposit_event<T>(&self, ts: &[T]) -> (Vec<T>, Vec<T>)
    where
        T: ToWethDepositEvent + std::fmt::Display + std::clone::Clone,
    {
        info!("number of things before filtering: {}", ts.len());
        let mut mutable_self = self.clone();
        let mut has_deposit_event = vec![];
        let mut does_not_have_deposit_event = vec![];
        for t in ts.iter() {
            let event = t.to_weth_deposit_event();
            info!("looking for this standard ERC20 transfer event: {}", event);
            if mutable_self.contains(&event) {
                // NOTE: If the event does exist in `mutable_self`, we MUST remove it before we
                // check the next event's existence!  This way, multiple of the exact same ETH
                // peg-ins/outs in a single submission will correctly require the same number of
                // corresponding weth deposit events to exist.
                mutable_self.remove(&event);
                info!("standard weth deposit event found in submission material!");
                has_deposit_event.push(t.clone());
                continue;
            }
            warn!("filtering this because it has no corresponding ERC20 transfer event: {t}");
            does_not_have_deposit_event.push(t.clone());
        }

        info!("number of things after filtering: {}", has_deposit_event.len());
        (has_deposit_event, does_not_have_deposit_event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eth_contracts::test_utils::get_sample_submission_material_with_weth_deposit;

    #[test]
    fn should_get_weth_deposit_events_from_eth_submission_material() {
        let m = get_sample_submission_material_with_weth_deposit();
        let cid = EthChainId::Mainnet;
        let r = WethDepositEvents::from_submission_material(&m, &cid);
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
