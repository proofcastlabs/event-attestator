use crate::{
    chains::eth::eth_state::EthState,
    dictionaries::eos_eth::EosEthTokenDictionary,
    eos_on_eth::fees_calculator::FeesCalculator,
    fees::fee_constants::DISABLE_FEES,
    traits::DatabaseInterface,
    types::Result,
};

pub fn update_accrued_fees_in_dictionary_and_return_eth_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `EosOnEthEthTxInfos`!");
        Ok(state)
    } else if state.eos_on_eth_eth_tx_infos.is_empty() {
        info!("✔ No `EosOnEthEthTxInfos` in state during ETH block submission ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accruing fees during ETH block submission...");
        EosEthTokenDictionary::get_from_db(&state.db)
            .and_then(|dictionary| {
                dictionary.increment_accrued_fees_and_save_in_db(
                    &state.db,
                    state.eos_on_eth_eth_tx_infos.get_fees(&dictionary)?,
                )
            })
            .and(Ok(state))
    }
}

pub fn account_for_fees_in_peg_in_infos_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `EosOnEthEthTxInfos`!");
        Ok(state)
    } else if state.eos_on_eth_eth_tx_infos.is_empty() {
        info!("✔ No `EosOnEthEthTxInfos` in state during ETH block submission ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accounting for fees in `EosOnEthEthTxInfos` during ETH block submission...");
        EosEthTokenDictionary::get_from_db(&state.db).and_then(|ref dictionary| {
            let tx_infos = state.eos_on_eth_eth_tx_infos.clone();
            state.replace_eos_on_eth_eth_tx_infos(tx_infos.subtract_fees(dictionary)?)
        })
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Accounting for fees in `EosOnEthEthTxInfos` during ETH block submission...");
    update_accrued_fees_in_dictionary_and_return_eth_state(state).and_then(account_for_fees_in_peg_in_infos_in_state)
}
