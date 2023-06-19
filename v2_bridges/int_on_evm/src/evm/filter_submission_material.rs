use common::{dictionaries::eth_evm::EthEvmTokenDictionary, traits::DatabaseInterface, types::Result};
use common_eth::{
    Erc777BurnEvent,
    Erc777RedeemEvent,
    EthLog,
    EthLogExt,
    EthLogs,
    EthReceipt,
    EthReceipts,
    EthState,
    EthSubmissionMaterial,
    ERC777_REDEEM_EVENT_TOPIC_V2,
    ERC_777_BURN_EVENT_TOPIC,
    ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA,
    ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA,
};
use ethereum_types::U256;

use crate::{
    constants::{PLTC_ADDRESS, PTLOS_ADDRESS},
    evm::int_tx_info::IntOnEvmIntTxInfos,
};

#[derive(Debug, Default)]
pub struct RelevantLogs {
    burn_logs: EthLogs,
    redeem_logs: EthLogs,
}

impl RelevantLogs {
    pub fn redeem_logs(&self) -> EthLogs {
        self.redeem_logs.clone()
    }

    pub fn to_burn_events(&self) -> Result<Vec<Erc777BurnEvent>> {
        self.burn_logs
            .iter()
            .map(Erc777BurnEvent::try_from)
            .collect::<Result<Vec<_>>>()
    }

    fn new(all_logs: &EthLogs) -> Result<Self> {
        let (burn_logs, redeem_logs) = all_logs.iter().fold((vec![], vec![]), |mut tuple, log| {
            if log.topics[0] == *ERC_777_BURN_EVENT_TOPIC {
                tuple.0.push(log.clone())
            } else {
                tuple.1.push(log.clone())
            };
            tuple
        });

        // NOTE: Since burns can exist without a corresponding redeem event, we filter any such
        // "orphans" out.
        let filtered_burn_logs = burn_logs
            .into_iter()
            .enumerate()
            .filter(|(i, log)| {
                match (
                    Self::get_value_from_burn_log(log),
                    Self::get_value_from_redeem_log(&redeem_logs[*i]),
                ) {
                    (Ok(a), Ok(b)) => a == b,
                    _ => false,
                }
            })
            .map(|(_, log)| log)
            .collect::<Vec<_>>();

        let n_redeem_logs = redeem_logs.len();
        let n_burn_logs = filtered_burn_logs.len();

        // NOTE: And after the above filtering, we should have an equal amount of burn & redeem logs
        if n_redeem_logs != n_burn_logs {
            return Err(
                format!("relevant logs ({n_redeem_logs}) does not match number of burn logs {n_burn_logs}").into(),
            );
        }

        Ok(Self {
            redeem_logs: EthLogs::new(redeem_logs),
            burn_logs: EthLogs::new(filtered_burn_logs),
        })
    }

    fn len(&self) -> usize {
        self.redeem_logs.len()
    }

    fn get_value_from_burn_log(l: &EthLog) -> Result<U256> {
        Erc777BurnEvent::try_from(l).map(|event| event.amount)
    }

    fn get_value_from_redeem_log(l: &EthLog) -> Result<U256> {
        Erc777RedeemEvent::from_eth_log(l).map(|event| event.value)
    }
}

impl IntOnEvmIntTxInfos {
    fn log_contains_topic(log: &EthLog) -> bool {
        debug!("checking log contains relevant topic...");
        if log.topics[0] == *ERC_777_BURN_EVENT_TOPIC {
            true
        } else if log.address == *PTLOS_ADDRESS {
            warn!("pTLOS redeem detected - checking for v1 event topics");
            log.contains_topic(&ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA)
                || log.contains_topic(&ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA)
        } else if log.address == *PLTC_ADDRESS {
            warn!("pLTC redeem detected - checking for v1 event topics");
            log.contains_topic(&ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA)
                || log.contains_topic(&ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA)
        } else {
            log.contains_topic(&ERC777_REDEEM_EVENT_TOPIC_V2)
        }
    }

    fn is_log_int_on_evm_redeem(log: &EthLog, dictionary: &EthEvmTokenDictionary) -> Result<bool> {
        let token_is_supported = dictionary.is_evm_token_supported(&log.address);
        let log_contains_topic = Self::log_contains_topic(log);
        debug!("✔ Log is supported: {}", token_is_supported);
        debug!("✔ Log has correct topic: {}", log_contains_topic);
        Ok(token_is_supported && log_contains_topic)
    }

    fn is_log_relevant(log: &EthLog, dictionary: &EthEvmTokenDictionary) -> Result<bool> {
        if Self::is_log_int_on_evm_redeem(log, dictionary)? {
            Ok(dictionary.is_evm_token_supported(&log.address))
        } else {
            Ok(false)
        }
    }

    pub fn get_relevant_logs_from_receipt(
        receipt: &EthReceipt,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<RelevantLogs> {
        RelevantLogs::new(&EthLogs::new(
            receipt
                .logs
                .iter()
                .filter(|log| matches!(Self::is_log_relevant(log, dictionary), Ok(true)))
                .cloned()
                .collect(),
        ))
    }

    fn receipt_contains_supported_erc20_on_evm_redeem(
        receipt: &EthReceipt,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<bool> {
        if Self::get_relevant_logs_from_receipt(receipt, dictionary)?.len() > 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn filter_eth_submission_material_for_supported_redeems(
        submission_material: &EthSubmissionMaterial,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<EthSubmissionMaterial> {
        info!("✔ Filtering submission material receipts for those pertaining to `ERC20-on-EVM` redeems...");
        info!(
            "✔ Num receipts before filtering: {}",
            submission_material.receipts.len()
        );
        let filtered_receipts = EthReceipts::new(
            submission_material
                .receipts
                .iter()
                .filter(|receipt| {
                    matches!(
                        IntOnEvmIntTxInfos::receipt_contains_supported_erc20_on_evm_redeem(receipt, dictionary),
                        Ok(true)
                    )
                })
                .cloned()
                .collect(),
        );
        info!("✔ Num receipts after filtering: {}", filtered_receipts.len());
        Ok(EthSubmissionMaterial::new(
            submission_material.get_block()?,
            filtered_receipts,
            None,
            None,
        ))
    }
}

pub fn filter_submission_material_for_redeem_events_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing `ERC20-on-EVM` redeem events...");
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_addresses_and_with_topics(
            &state.get_eth_evm_token_dictionary()?.to_evm_addresses(),
            &[
                *ERC777_REDEEM_EVENT_TOPIC_V2,
                // NOTE: The following v1 events are for allowing the migration of v1 bridge whose
                // contracts are not upgradeable to v2 bridges.
                *ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA,
                *ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA,
            ],
        )
        .and_then(|filtered_submission_material| {
            IntOnEvmIntTxInfos::filter_eth_submission_material_for_supported_redeems(
                &filtered_submission_material,
                state.get_eth_evm_token_dictionary()?,
            )
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{get_sample_peg_out_submission_material, get_sample_token_dictionary};

    #[test]
    fn should_filter_submission_info_for_supported_redeems() {
        let dictionary = get_sample_token_dictionary();
        let material = get_sample_peg_out_submission_material();
        let result =
            IntOnEvmIntTxInfos::filter_eth_submission_material_for_supported_redeems(&material, &dictionary).unwrap();
        let expected_num_receipts = 1;
        assert_eq!(result.receipts.len(), expected_num_receipts);
    }
}
