use common::{
    chains::eth::{
        eth_contracts::erc20_vault::ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2,
        eth_database_utils::EthDbUtilsExt,
        eth_receipt::{EthReceipt, EthReceipts},
        eth_submission_material::EthSubmissionMaterial,
    },
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};
use ethereum_types::Address as EthAddress;

use crate::eth::int_tx_info::Erc20OnIntIntTxInfos;

impl Erc20OnIntIntTxInfos {
    fn receipt_contains_supported_erc20_on_evm_peg_in(receipt: &EthReceipt, vault_address: &EthAddress) -> bool {
        Self::get_supported_erc20_on_evm_logs_from_receipt(receipt, vault_address).len() > 0
    }

    fn filter_eth_submission_material_for_supported_peg_ins(
        submission_material: &EthSubmissionMaterial,
        vault_address: &EthAddress,
    ) -> Result<EthSubmissionMaterial> {
        info!("✔ Filtering submission material receipts for those pertaining to `erc20-on-int` peg-ins...");
        info!(
            "✔ Num receipts before filtering: {}",
            submission_material.receipts.len()
        );
        let filtered_receipts = EthReceipts::new(
            submission_material
                .receipts
                .iter()
                .filter(|receipt| {
                    Erc20OnIntIntTxInfos::receipt_contains_supported_erc20_on_evm_peg_in(receipt, vault_address)
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

pub fn filter_submission_material_for_peg_in_events_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing `erc20-on-int` peg in events...");
    let vault_address = state.eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?;
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_address_and_with_topics(&vault_address, &[*ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2])
        .and_then(|filtered_submission_material| {
            Erc20OnIntIntTxInfos::filter_eth_submission_material_for_supported_peg_ins(
                &filtered_submission_material,
                &vault_address,
            )
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{get_sample_peg_in_1_submission_material, get_sample_vault_address};

    #[test]
    fn should_filter_submission_info_for_supported_redeems() {
        let material = get_sample_peg_in_1_submission_material();
        let vault_address = get_sample_vault_address();
        let result =
            Erc20OnIntIntTxInfos::filter_eth_submission_material_for_supported_peg_ins(&material, &vault_address)
                .unwrap();
        let expected_num_receipts = 1;
        assert_eq!(result.receipts.len(), expected_num_receipts);
    }
}
