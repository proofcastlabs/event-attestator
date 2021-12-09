use crate::{
    chains::eth::eth_state::EthState,
    dictionaries::eth_evm::EthEvmTokenDictionary,
    erc20_on_int::fees_calculator::FeesCalculator,
    fees::fee_constants::DISABLE_FEES,
    traits::DatabaseInterface,
    types::Result,
};

pub fn update_accrued_fees_in_dictionary_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `EthOnIntEthTxInfos`!");
        Ok(state)
    } else if state.erc20_on_evm_eth_tx_infos.is_empty() {
        info!("✔ Not `EthOnIntEthTxInfos` in state ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accruing fees during EVM block submission...");
        EthEvmTokenDictionary::get_from_db(state.db)
            .and_then(|ref dictionary| {
                dictionary.increment_accrued_fees_and_save_in_db(
                    state.db,
                    state.erc20_on_int_eth_tx_infos.get_fees(dictionary)?,
                )
            })
            .and(Ok(state))
    }
}

pub fn account_for_fees_in_eth_tx_infos_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `EthOnEvmIntTxInfos`!");
        Ok(state)
    } else if state.erc20_on_evm_eth_tx_infos.is_empty() {
        info!("✔ Not `EthOnEvmIntTxInfos` in state ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accounting for fees in `EthOnIntEthTxInfos` during EVM block submission...");
        EthEvmTokenDictionary::get_from_db(state.db).and_then(|ref dictionary| {
            let tx_infos = state.erc20_on_int_eth_tx_infos.clone();
            state.replace_erc20_on_int_eth_tx_infos(tx_infos.subtract_fees(dictionary)?)
        })
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Accounting for fees in `EthOnEvmIntTxInfos` during EVM block submission...");
    update_accrued_fees_in_dictionary_and_return_state(state).and_then(account_for_fees_in_eth_tx_infos_in_state)
}
