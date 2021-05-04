use crate::{
    chains::eth::eth_state::EthState,
    dictionaries::eth_evm::EthEvmTokenDictionary,
    erc20_on_evm::traits::FeesCalculator,
    fees::fee_constants::DISABLE_FEES,
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Maybe accounting for fees during ETH block submission...");
    if DISABLE_FEES {
        info!("✔ Taking fees is disabled ∴ not taking any fees!");
        Ok(state)
    } else if state.erc20_on_evm_evm_tx_infos.is_empty() {
        info!("✔ Not tx info in state ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accounting for fees in `EthOnEvmEvmTxInfos` during ETH block submission...");
        let tx_infos = state.erc20_on_evm_evm_tx_infos.clone();
        let dictionary = EthEvmTokenDictionary::get_from_db(&state.db)?;
        dictionary.increment_accrued_fees_and_save_in_db(&state.db, tx_infos.get_fees(&dictionary)?)?;
        state.replace_erc20_on_evm_evm_tx_infos(tx_infos.subtract_fees(&dictionary)?)
    }
}
