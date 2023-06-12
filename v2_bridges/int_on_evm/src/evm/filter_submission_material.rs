use common::{dictionaries::eth_evm::EthEvmTokenDictionary, traits::DatabaseInterface, types::Result};
use common_eth::{
    EthLog,
    EthLogExt,
    EthLogs,
    EthReceipt,
    EthReceipts,
    EthState,
    EthSubmissionMaterial,
    ERC777_REDEEM_EVENT_TOPIC_V2,
    ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA,
    ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA,
};

use crate::{constants::PTELOS_ADDRESS, evm::int_tx_info::IntOnEvmIntTxInfos};

impl IntOnEvmIntTxInfos {
    fn is_log_int_on_evm_redeem(log: &EthLog, dictionary: &EthEvmTokenDictionary) -> Result<bool> {
        debug!(
            "✔ Checking log contains topic: {}",
            hex::encode(ERC777_REDEEM_EVENT_TOPIC_V2.as_bytes())
        );
        let token_is_supported = dictionary.is_evm_token_supported(&log.address);
        let log_contains_topic = if log.address == *PTELOS_ADDRESS {
            warn!("pTLOS redeem detected - checking for v1 event topics");
            log.contains_topic(&ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA)
                || log.contains_topic(&ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA)
        } else {
            log.contains_topic(&ERC777_REDEEM_EVENT_TOPIC_V2)
        };
        debug!("✔ Log is supported: {}", token_is_supported);
        debug!("✔ Log has correct topic: {}", log_contains_topic);
        Ok(token_is_supported && log_contains_topic)
    }

    fn is_log_relevant(log: &EthLog, dictionary: &EthEvmTokenDictionary) -> Result<bool> {
        match Self::is_log_int_on_evm_redeem(log, dictionary)? {
            false => Ok(false),
            true => Ok(dictionary.is_evm_token_supported(&log.address)),
        }
    }

    pub fn get_relevant_logs_from_receipt(receipt: &EthReceipt, dictionary: &EthEvmTokenDictionary) -> EthLogs {
        EthLogs::new(
            receipt
                .logs
                .iter()
                .filter(|log| matches!(Self::is_log_relevant(log, dictionary), Ok(true)))
                .cloned()
                .collect(),
        )
    }

    fn receipt_contains_supported_erc20_on_evm_redeem(
        receipt: &EthReceipt,
        dictionary: &EthEvmTokenDictionary,
    ) -> bool {
        Self::get_relevant_logs_from_receipt(receipt, dictionary).len() > 0
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
                    IntOnEvmIntTxInfos::receipt_contains_supported_erc20_on_evm_redeem(receipt, dictionary)
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
