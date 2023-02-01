use crate::{
    dictionaries::eos_eth::EosEthTokenDictionary,
    eos_on_eth::fees_calculator::FeesCalculator,
    fees::fee_constants::DISABLE_FEES,
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};

pub fn update_accrued_fees_in_dictionary_and_return_eos_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `EosOnEthEosTxInfos`!");
        Ok(state)
    } else if state.eos_on_eth_eos_tx_infos.is_empty() {
        info!("✔ No `EosOnEthEosTxInfos` in state during EOS block submission ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accruing fees during EOS block submission...");
        EosEthTokenDictionary::get_from_db(state.db)
            .and_then(|dictionary| {
                dictionary.increment_accrued_fees_and_save_in_db(
                    state.db,
                    &state.eos_on_eth_eos_tx_infos.get_fees(&dictionary)?,
                )
            })
            .and(Ok(state))
    }
}

pub fn account_for_fees_in_eos_tx_infos_in_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `EosOnEthEosTxInfos`!");
        Ok(state)
    } else if state.eos_on_eth_eos_tx_infos.is_empty() {
        info!("✔ No `EosOnEthEosTxInfos` in state during EOS block submission ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accounting for fees in `EosOnEthEosTxInfos` during EOS block submission...");
        EosEthTokenDictionary::get_from_db(state.db).and_then(|ref dictionary| {
            let redeem_infos = state.eos_on_eth_eos_tx_infos.clone();
            state.replace_eos_on_eth_eos_tx_infos(redeem_infos.subtract_fees(dictionary)?)
        })
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Accounting for fees in `EosOnEthEosTxInfos` during EOS block submission...");
    update_accrued_fees_in_dictionary_and_return_eos_state(state).and_then(account_for_fees_in_eos_tx_infos_in_state)
}
