use std::fmt;

use common::types::Result;
use derive_more::{Constructor, Deref, DerefMut};
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, U256};

use crate::{
    eth_log::EthLogExt,
    eth_receipt::EthReceipt,
    eth_submission_material::EthSubmissionMaterial,
    eth_utils::{convert_eth_address_to_string, convert_hex_to_eth_address},
};

crate::make_topics!(ERC20_TOKEN_TRANSFER_EVENT_TOPIC => "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

lazy_static! {
    static ref PNT_TOKEN_ADDRESS_ON_ETH: EthAddress =
        convert_hex_to_eth_address("0x89Ab32156e46F46D02ade3FEcbe5Fc4243B9AAeD")
            .expect("Invalid ETH address hex for `PNT_TOKEN_ADDRESS_ON_ETH`!");
    static ref ETHPNT_TOKEN_ADDRESS_ON_ETH: EthAddress =
        convert_hex_to_eth_address("0xf4ea6b892853413bd9d9f1a5d3a620a0ba39c5b2")
            .expect("Invalid ETH address hex for `ETHPNT_TOKEN_ADDRESS_ON_ETH`!");
}

pub trait ToErc20TokenTransferEvent {
    fn to_erc20_token_transfer_event(&self) -> Erc20TokenTransferEvent;
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Deref, DerefMut, Constructor)]
pub struct Erc20TokenTransferEvents(Vec<Erc20TokenTransferEvent>);

impl Erc20TokenTransferEvents {
    fn from_eth_receipt(eth_receipt: &EthReceipt) -> Vec<Erc20TokenTransferEvent> {
        eth_receipt
            .logs
            .iter()
            .filter(|log| log.contains_topic(&ERC20_TOKEN_TRANSFER_EVENT_TOPIC))
            .filter_map(|log| {
                if let Ok(event) = Erc20TokenTransferEvent::from_eth_log(log) {
                    Some(event)
                } else {
                    // NOTE: The logs are already filtered for those that contain the correct topic,
                    // and so it should always parse to the expected event. Thus the `result` here
                    // should never error, hence why we can filter out any without concern.
                    None
                }
            })
            .collect()
    }

    fn from_eth_receipts(receipts: &[EthReceipt]) -> Self {
        Self::new(receipts.iter().flat_map(Self::from_eth_receipt).collect::<Vec<_>>())
    }

    fn from_eth_submission_material(submission_material: &EthSubmissionMaterial) -> Self {
        Self::from_eth_receipts(&submission_material.receipts)
    }

    fn remove(&mut self, event_to_remove: &Erc20TokenTransferEvent) {
        if let Some(index) = self.iter().position(|event| event == event_to_remove) {
            // NOTE: We don't care about ordering here!
            self.swap_remove(index);
        };
    }

    fn filter_if_no_transfer_event<T>(&self, ts: &[T]) -> Vec<T>
    where
        T: ToErc20TokenTransferEvent + std::fmt::Display + std::clone::Clone,
    {
        info!("✔ Number of things before filtering: {}", ts.len());
        let mut mutable_self = self.clone();
        let filtered = ts
            .iter()
            .filter(|t| {
                let event = t.to_erc20_token_transfer_event();
                info!("✔ Looking for this standard ERC20 transfer event: {}", event);
                if mutable_self.contains(&event) {
                    // NOTE: If the event does exist in `mutable_self`, we MUST remove it before we
                    // check the next event's existence!  This way, multiple of the exact same
                    // peg-ins/outs in a single submission will correctly require the same number of
                    // corresponding token transfers events to exist.
                    mutable_self.remove(&event);
                    info!("✔ Standard ERC20 transfer event found in submission material!");
                    return true;
                }
                info!("✘ No standard ERC20 transfer event found. Checking if the event is for ETHPNT...");
                let eth_pnt_event = event.update_emittance_address(&ETHPNT_TOKEN_ADDRESS_ON_ETH);
                if event.token_address == *PNT_TOKEN_ADDRESS_ON_ETH && mutable_self.contains(&eth_pnt_event) {
                    // NOTE: So a vault change will mean that a ETHPNT peg in will fire a peg-in event to make a
                    // PNT peg in happen. This means the PNT peg-in event will NOT have a corresponding
                    // ERC20 transfer event, but there will exist instead an ETHPNT erc20 transfer
                    // event, which we will look for to pass this validation step instead.

                    // NOTE: See above for why we remove the event.
                    mutable_self.remove(&eth_pnt_event);
                    info!("✔ ETHPNT transfer event found in submission material!");
                    return true;
                }
                info!("✘ No ETHPNT event found. Checking if the event is for a minting transfer event...");
                let minting_transfer_event = event.update_the_from_address(&EthAddress::zero());
                if mutable_self.contains(&minting_transfer_event) {
                    // NOTE: So in the case of pegging in the wrapped version of a native token
                    // (eg wETH) via the vault's `pegInEth(...)` function, the corresponding
                    // transfer event will be a _minting_ event of that wETH token. This means that
                    // the `from` address in the event will be the ETH zero address.

                    // NOTE: See above for why we remove the event.
                    mutable_self.remove(&minting_transfer_event);
                    info!("✔ Minting transfer event found in submission material!");
                    return true;
                }
                warn!(
                    "✘ Filtering this out because it has no corresponding ERC20 transfer event: {}",
                    t,
                );
                false
            })
            .cloned()
            .collect::<Vec<T>>();
        info!("✔ Number of things after filtering: {}", filtered.len());
        filtered
    }

    pub fn filter_if_no_transfer_event_in_submission_material<T>(
        submission_material: &EthSubmissionMaterial,
        ts: &[T],
    ) -> Vec<T>
    where
        T: ToErc20TokenTransferEvent + std::fmt::Display + std::clone::Clone,
    {
        Self::from_eth_submission_material(submission_material).filter_if_no_transfer_event(ts)
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Constructor)]
pub struct Erc20TokenTransferEvent {
    pub value: U256,
    pub to: EthAddress,
    pub from: EthAddress,
    pub token_address: EthAddress, // NOTE: Whence the event was emitted.
}

impl fmt::Display for Erc20TokenTransferEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "
Erc20TokenTransferEvent: {{
    to: {},
    from: {},
    value: {},
    token_address: {},
}}
",
            convert_eth_address_to_string(&self.to),
            convert_eth_address_to_string(&self.from),
            self.value,
            convert_eth_address_to_string(&self.token_address),
        )
    }
}

impl Erc20TokenTransferEvent {
    fn get_err_msg(field: &str) -> String {
        format!("Error getting `{}` for `Erc20TokenTransferEvent`!", field)
    }

    fn from_eth_log<L: EthLogExt>(log: &L) -> Result<Self> {
        info!("Decoding ERC20 token transfer event from log...");
        log
            // NOTE: ERC20 events with the correct topic hash will ALWAYS have three topics total.
            // The first topic is the hash of the event signature, the second is first indexed argument
            // (the `from` address) and the third is the second indexed argument (the `to` address).
            .check_has_x_topics(3)
            .and_then(|_| {
                let tokens = eth_abi_decode(&[EthAbiParamType::Uint(256)], &log.get_data())?;
                Ok(Self {
                    token_address: log.get_address(),
                    // NOTE: The 20 byte ETH addresses are stored in 32 byte words, right aligned.
                    from: EthAddress::from_slice(&log.get_topics()[1][12..]),
                    to: EthAddress::from_slice(&log.get_topics()[2][12..]),
                    value: match tokens[0] {
                        EthAbiToken::Uint(value) => Ok(value),
                        _ => Err(Self::get_err_msg("from")),
                    }?,
                })

            })
    }

    pub fn update_emittance_address(&self, address: &EthAddress) -> Self {
        let mut mutable_self = self.clone();
        mutable_self.token_address = *address;
        mutable_self
    }

    pub fn update_the_from_address(&self, address: &EthAddress) -> Self {
        let mut mutable_self = self.clone();
        mutable_self.from = *address;
        mutable_self
    }
}

#[cfg(test)]
use crate::test_utils::{get_random_eth_address, get_random_u256};

#[cfg(test)]
impl Erc20TokenTransferEvent {
    fn random() -> Self {
        Self::new(
            get_random_u256(),
            get_random_eth_address(),
            get_random_eth_address(),
            get_random_eth_address(),
        )
    }
}

#[cfg(test)]
impl Erc20TokenTransferEvents {
    fn get_n_random_events(n: usize) -> Self {
        Self::new(
            vec![0; n]
                .iter()
                .map(|_| Erc20TokenTransferEvent::random())
                .collect::<Vec<_>>(),
        )
    }
}

// NOTE: So that we can use a list of `Erc20TokenTransferEvent`s when tesing the filterer
// in `Erc20TokenTransferEvents`.
#[cfg(test)]
impl ToErc20TokenTransferEvent for Erc20TokenTransferEvent {
    fn to_erc20_token_transfer_event(&self) -> Erc20TokenTransferEvent {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_sample_submission_material_with_erc20_peg_in_event;

    #[test]
    fn should_get_erc20_token_params_from_submission_material() {
        let submission_material = get_sample_submission_material_with_erc20_peg_in_event().unwrap();
        let result = Erc20TokenTransferEvents::from_eth_submission_material(&submission_material);
        let expected_num_results = 16;
        assert_eq!(result.len(), expected_num_results);
    }

    #[test]
    fn should_remove_erc20_token_transfer_event_from_events() {
        let num_events = 10;
        let mut events = Erc20TokenTransferEvents::get_n_random_events(num_events);
        let event = events[5].clone();
        let event_exist_before = events.contains(&event);
        let num_results_before = events.len();
        assert_eq!(num_results_before, num_events);
        assert!(event_exist_before);
        events.remove(&event);
        let num_results_after = events.len();
        assert_eq!(num_results_after, num_events - 1);
        let event_exists_after = events.contains(&event);
        assert!(!event_exists_after);
    }

    #[test]
    fn should_not_filter_things_if_all_have_corresponding_erc20_token_transfer_events() {
        let events = Erc20TokenTransferEvents::get_n_random_events(10);
        let things_to_filter = Erc20TokenTransferEvents::new(vec![
            events[0].clone(),
            events[1].clone(),
            events[3].clone(),
            events[4].clone(),
            events[5].clone(),
        ]);
        let result = events.filter_if_no_transfer_event(&things_to_filter);
        assert_eq!(result.len(), things_to_filter.len());
        things_to_filter.iter().for_each(|thing| {
            assert!(result.contains(thing));
        });
    }

    #[test]
    fn should_filter_things_if_they_do_not_have_corresponding_erc20_token_transfer_events() {
        let events = Erc20TokenTransferEvents::get_n_random_events(10);
        // NOTE: This test also shows that the ordering of the things does not matter!
        let things_that_will_not_be_filtered_out = vec![
            events[7].clone(),
            events[5].clone(),
            events[3].clone(),
            events[0].clone(),
        ];
        let things_that_will_be_filtered_out = vec![
            Erc20TokenTransferEvent::random(),
            Erc20TokenTransferEvent::random(),
            Erc20TokenTransferEvent::random(),
            Erc20TokenTransferEvent::random(),
        ];
        // NOTE: We're just zipping them together into a single vec because the filterer
        // does not & should not care about ordering.
        let things_to_filter = things_that_will_not_be_filtered_out
            .iter()
            .zip(things_that_will_be_filtered_out.iter())
            .fold(Vec::new(), |mut acc, (a, b)| {
                acc.push(a.clone());
                acc.push(b.clone());
                acc
            });
        let result = events.filter_if_no_transfer_event(&things_to_filter);
        assert_eq!(result.len(), things_that_will_not_be_filtered_out.len());
        result.iter().for_each(|thing| {
            assert!(things_that_will_not_be_filtered_out.contains(thing));
            assert!(!things_that_will_be_filtered_out.contains(thing));
        })
    }

    #[test]
    fn filter_should_require_transfer_event_for_every_thing_even_when_thing_is_duplicated() {
        let mut events = Erc20TokenTransferEvents::get_n_random_events(10);
        let repeated_thing = events[5].clone();
        events.push(repeated_thing.clone()); // NOTE: Thus there may be two repeated things in the final result...
        let things_that_will_not_be_filtered_out = vec![
            events[7].clone(),
            repeated_thing.clone(),
            events[3].clone(),
            repeated_thing.clone(),
            events[0].clone(),
        ];
        let things_that_will_be_filtered_out = vec![
            Erc20TokenTransferEvent::random(),
            repeated_thing.clone(), // NOTE: But this one...
            Erc20TokenTransferEvent::random(),
            Erc20TokenTransferEvent::random(),
            repeated_thing.clone(), // NOTE: And this one should both get filtered out.
        ];
        // NOTE: We're just zipping them together into a single vec because the filterer
        // does not & should not care about ordering.
        let things_to_filter = things_that_will_not_be_filtered_out
            .iter()
            .zip(things_that_will_be_filtered_out.iter())
            .fold(Vec::new(), |mut acc, (a, b)| {
                acc.push(a.clone());
                acc.push(b.clone());
                acc
            });
        let results = events.filter_if_no_transfer_event(&things_to_filter);
        assert_eq!(results.len(), things_that_will_not_be_filtered_out.len());
        results.iter().for_each(|thing| {
            assert!(things_that_will_not_be_filtered_out.contains(thing));
        });
        // NOTE: Now we need to assert that ONLY two of the repeated things made it through the filter...
        assert_eq!(results.iter().filter(|thing| *thing == &repeated_thing).count(), 2);
        // NOTE: We can assert the things to be filtered out were indeed filtered out, but first we
        // need to remove the repeated element since it'll give false asserion failures.
        things_that_will_be_filtered_out
            .iter()
            .filter(|thing| *thing != &repeated_thing)
            .for_each(|thing| assert!(!results.contains(thing)));
    }

    #[test]
    fn pnt_event_should_not_get_filtered_out_if_ethpnt_transfer_event_exists() {
        let mut events_to_filter = Erc20TokenTransferEvents::get_n_random_events(10);

        // NOTE: Make the PNT event which won't have a corresponding event...
        let pnt_event = Erc20TokenTransferEvent::random().update_emittance_address(&PNT_TOKEN_ADDRESS_ON_ETH);

        // NOTE: But will have a corresponding EthPNT event
        let ethpnt_event = pnt_event.update_emittance_address(&ETHPNT_TOKEN_ADDRESS_ON_ETH);
        events_to_filter.push(ethpnt_event);
        assert!(!events_to_filter.contains(&pnt_event));
        let things_that_will_not_be_filtered_out = vec![pnt_event];
        let result = events_to_filter.filter_if_no_transfer_event(&things_that_will_not_be_filtered_out);
        assert_eq!(result.len(), 1);
        assert_eq!(result, things_that_will_not_be_filtered_out);
    }

    #[test]
    fn event_should_not_be_filtered_out_if_it_has_a_corresponding_mint_transfer_event() {
        let mut events_to_filter = Erc20TokenTransferEvents::get_n_random_events(10);

        // NOTE: Make an event which won't have a corresponding transfer event...
        let event = Erc20TokenTransferEvent::random();

        // NOTE: But will have a corresponding minting transfer_event...
        let minting_transfer_event = event.update_the_from_address(&EthAddress::zero());

        events_to_filter.push(minting_transfer_event);
        assert!(!events_to_filter.contains(&event));

        let things_that_will_not_be_filtered_out = vec![event];
        let result = events_to_filter.filter_if_no_transfer_event(&things_that_will_not_be_filtered_out);
        assert_eq!(result.len(), 1);
        assert_eq!(result, things_that_will_not_be_filtered_out);
    }
}
