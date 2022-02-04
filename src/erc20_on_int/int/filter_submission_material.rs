use crate::{
    chains::eth::{
        eth_contracts::erc777::ERC777_REDEEM_EVENT_TOPIC_V2,
        eth_receipt::{EthReceipt, EthReceipts},
        eth_state::EthState,
        eth_submission_material::EthSubmissionMaterial,
    },
    dictionaries::eth_evm::EthEvmTokenDictionary,
    erc20_on_int::int::eth_tx_info::Erc20OnIntEthTxInfos,
    traits::DatabaseInterface,
    types::Result,
};

impl Erc20OnIntEthTxInfos {
    fn receipt_contains_redeem_event(receipt: &EthReceipt, dictionary: &EthEvmTokenDictionary) -> bool {
        Self::get_logs_with_redeem_event_from_receipt(receipt, dictionary).len() > 0
    }

    fn filter_submission_material_for_supported_redeems(
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
                .filter(|receipt| Erc20OnIntEthTxInfos::receipt_contains_redeem_event(receipt, dictionary))
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
            &[*ERC777_REDEEM_EVENT_TOPIC_V2],
        )
        .and_then(|filtered_submission_material| {
            Erc20OnIntEthTxInfos::filter_submission_material_for_supported_redeems(
                &filtered_submission_material,
                state.get_eth_evm_token_dictionary()?,
            )
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc20_on_int::test_utils::{get_sample_peg_out_submission_material, get_sample_token_dictionary};

    #[test]
    fn should_filter_submission_info_for_supported_redeems() {
        let dictionary = get_sample_token_dictionary();
        let material = get_sample_peg_out_submission_material();
        let result =
            Erc20OnIntEthTxInfos::filter_submission_material_for_supported_redeems(&material, &dictionary).unwrap();
        let expected_num_receipts = 1;
        assert_eq!(result.receipts.len(), expected_num_receipts);
    }
}
