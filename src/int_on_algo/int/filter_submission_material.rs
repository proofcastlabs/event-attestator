use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::{
        eth_contracts::erc20_vault::ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2,
        eth_database_utils::EthDbUtilsExt,
        eth_receipt::{EthReceipt, EthReceipts},
        eth_state::EthState,
        eth_submission_material::EthSubmissionMaterial,
    },
    int_on_algo::int::algo_tx_info::IntOnAlgoAlgoTxInfos,
    traits::DatabaseInterface,
    types::Result,
};

impl IntOnAlgoAlgoTxInfos {
    fn receipt_containts_supported_peg_in(receipt: &EthReceipt, vault_address: &EthAddress) -> bool {
        Self::get_supported_logs(receipt, vault_address).len() > 0
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
                .filter(|receipt| IntOnAlgoAlgoTxInfos::receipt_containts_supported_peg_in(receipt, vault_address))
                .cloned()
                .collect(),
        );
        info!("✔ Num receipts after filtering: {}", filtered_receipts.len());
        let mut mutable_submission_material = submission_material.clone();
        mutable_submission_material.receipts = filtered_receipts;
        Ok(mutable_submission_material)
    }
}

pub fn filter_submission_material_for_peg_in_events_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing `int-on-algo` peg in events...");
    let vault_address = state.eth_db_utils.get_int_on_algo_smart_contract_address()?;
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_address_and_with_topics(&vault_address, &[*ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2])
        .and_then(|filtered_submission_material| {
            IntOnAlgoAlgoTxInfos::filter_eth_submission_material_for_supported_peg_ins(
                &filtered_submission_material,
                &vault_address,
            )
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}

// TODO Test!
