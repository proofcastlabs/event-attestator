use common::{
    chains::eth::EthState,
    dictionaries::eth_evm::EthEvmTokenDictionary,
    fees::fee_constants::DISABLE_FEES,
    traits::DatabaseInterface,
    types::Result,
};

use crate::{evm::Erc20OnEvmEthTxInfos, fees_calculator::FeesCalculator};

pub fn update_accrued_fees_in_dictionary_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `Erc20OnEvmEthTxInfos`!");
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!("✔ Not `Erc20OnEvmEthTxInfos` in state ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accruing fees during EVM block submission...");
        let dictionary = EthEvmTokenDictionary::get_from_db(state.db)?;
        let fees = Erc20OnEvmEthTxInfos::from_bytes(&state.tx_infos)?.get_fees(&dictionary)?;
        dictionary
            .increment_accrued_fees_and_save_in_db(state.db, fees)
            .and(Ok(state))
    }
}

pub fn account_for_fees_in_eth_tx_infos_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `Erc20OnEvmEthTxInfos`!");
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!("✔ Not `Erc20OnEvmEthTxInfos` in state ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accounting for fees in `Erc20OnEvmEthTxInfos` during EVM block submission...");
        let dictionary = EthEvmTokenDictionary::get_from_db(state.db)?;
        Erc20OnEvmEthTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| infos.subtract_fees(&dictionary))
            .and_then(|infos| infos.to_bytes())
            .map(|bytes| state.add_tx_infos(bytes))
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Accounting for fees in `Erc20OnEvmEthTxInfos` during EVM block submission...");
    update_accrued_fees_in_dictionary_and_return_state(state).and_then(account_for_fees_in_eth_tx_infos_in_state)
}
