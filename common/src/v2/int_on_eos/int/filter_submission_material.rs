use crate::{
    chains::eth::{
        eth_contracts::erc20_vault::ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2,
        eth_database_utils::EthDbUtilsExt,
        eth_receipt::{EthReceipt, EthReceipts},
        eth_submission_material::EthSubmissionMaterial,
    },
    dictionaries::eos_eth::EosEthTokenDictionary,
    int_on_eos::int::eos_tx_info::IntOnEosEosTxInfos,
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

impl IntOnEosEosTxInfos {
    fn receipt_contains_relevant_logs(receipt: &EthReceipt, dictionary: &EosEthTokenDictionary) -> bool {
        warn!("here is the dict: {:?}", dictionary); // FIXME rm!
        Self::get_relevant_logs_from_receipt(receipt, dictionary).len() > 0
    }

    fn filter_eth_submission_material_for_supported_peg_ins(
        submission_material: &EthSubmissionMaterial,
        dictionary: &EosEthTokenDictionary,
    ) -> Result<EthSubmissionMaterial> {
        info!("✔ Filtering submission material receipts pertaining to `int-on-eos` peg-ins...");
        info!(
            "✔ Num receipts before filtering: {}",
            submission_material.receipts.len()
        );
        let filtered_receipts = EthReceipts::new(
            submission_material
                .receipts
                .iter()
                .filter(|receipt| Self::receipt_contains_relevant_logs(receipt, dictionary))
                .cloned()
                .collect(),
        );
        info!("✔ Num receipts after filtering: {}", filtered_receipts.len());
        Ok(EthSubmissionMaterial::new(
            submission_material.get_block()?,
            filtered_receipts,
            submission_material.eos_ref_block_num,
            submission_material.eos_ref_block_prefix,
        ))
    }
}

pub fn filter_submission_material_for_relevant_receipts_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing relevant `INT-on-EOS` events...");
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_address_and_with_topics(
            &state.eth_db_utils.get_int_on_eos_smart_contract_address_from_db()?,
            &[*ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2],
        )
        .and_then(|filtered_submission_material| {
            IntOnEosEosTxInfos::filter_eth_submission_material_for_supported_peg_ins(
                &filtered_submission_material,
                state.get_eos_eth_token_dictionary()?,
            )
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}
