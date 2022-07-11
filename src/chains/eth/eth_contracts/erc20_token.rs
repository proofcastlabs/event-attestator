use derive_more::{Constructor, Deref};
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    chains::eth::{eth_log::EthLogExt, eth_receipt::EthReceipt, eth_submission_material::EthSubmissionMaterial},
    types::Result,
};

const ERC20_TOKEN_TRANSFER_EVENT_TOPIC_HEX: &str = "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

lazy_static! {
    // TODO macro for these!! (Since they're in other contract mods too!)
    static ref ERC20_TOKEN_TRANSFER_EVENT_TOPIC: EthHash = {
        EthHash::from_slice(
            &hex::decode(ERC20_TOKEN_TRANSFER_EVENT_TOPIC_HEX)
                .expect("✘ Invalid hex in `ERC20_TOKEN_TRANSFER_EVENT_TOPIC_HEX`!"),
        )
    };
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Deref, Constructor)]
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

    fn from_eth_receipts(receipts: &[EthReceipt]) -> Result<Self> {
        Ok(Self::new(
            receipts.iter().flat_map(Self::from_eth_receipt).collect::<Vec<_>>(),
        ))
    }

    pub fn from_eth_submission_material(submission_material: &EthSubmissionMaterial) -> Result<Self> {
        Self::from_eth_receipts(&submission_material.receipts)
    }

    pub fn erc20_transfer_exists(
        &self,
        token_address: &EthAddress,
        from: &EthAddress,
        to: &EthAddress,
        value: &U256,
    ) -> bool {
        info!("✔ Checking erc20 token transfer exists...");
        self.iter()
            .filter(|event| {
                let is_correct_to_address = event.to == *to;
                let is_correct_value = event.value == *value;
                let is_correct_from_address = event.from == *from;
                let is_correct_token_address = event.token_address == *token_address;
                debug!("Correct value:         {}", is_correct_value);
                debug!("Correct to address:    {}", is_correct_to_address);
                debug!("Correct token address: {}", is_correct_token_address);
                debug!("Correct from address:  {}", is_correct_from_address);
                is_correct_token_address && is_correct_to_address && is_correct_from_address && is_correct_value
            })
            .count()
            >= 1
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Erc20TokenTransferEvent {
    pub value: U256,
    pub to: EthAddress,
    pub from: EthAddress,
    pub topics: Vec<EthHash>,
    pub token_address: EthAddress, // NOTE: Whence the event was emitted.
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
                    topics: log.get_topics(),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::eth::{
        eth_test_utils::get_sample_submission_material_with_erc20_peg_in_event,
        eth_utils::convert_hex_to_eth_address,
    };

    #[test]
    fn should_get_erc20_token_params_from_submission_material() {
        let submission_material = get_sample_submission_material_with_erc20_peg_in_event().unwrap();
        let result = Erc20TokenTransferEvents::from_eth_submission_material(&submission_material).unwrap();
        let expected_num_results = 16;
        assert_eq!(result.len(), expected_num_results);
    }

    #[test]
    fn erc20_transfer_should_exist() {
        let events = Erc20TokenTransferEvents::from_eth_submission_material(
            &get_sample_submission_material_with_erc20_peg_in_event().unwrap(),
        )
        .unwrap();
        let from = convert_hex_to_eth_address("0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap();
        let to = convert_hex_to_eth_address("0xd0a3d2d3d19a6ac58e60254fd606ec766638c3ba").unwrap();
        let token_address = convert_hex_to_eth_address("0x9f57cb2a4f462a5258a49e88b4331068a391de66").unwrap();
        let value = U256::from(1337);
        let result = events.erc20_transfer_exists(&token_address, &from, &to, &value);
        assert!(result);
    }
}
