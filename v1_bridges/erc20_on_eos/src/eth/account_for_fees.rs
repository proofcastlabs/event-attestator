use common::{
    dictionaries::eos_eth::EosEthTokenDictionary,
    fees::fee_constants::DISABLE_FEES,
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

use crate::{eth::Erc20OnEosEosTxInfos, fees_calculator::FeesCalculator};

pub fn update_accrued_fees_in_dictionary_and_return_eth_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `Erc20OnEosEosTxInfos`!");
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!("✔ No `Erc20OnEosEosTxInfos` in state during ETH block submission ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accruing fees during ETH block submission...");
        let dictionary = EosEthTokenDictionary::get_from_db(state.db)?;
        let fees = Erc20OnEosEosTxInfos::from_bytes(&state.tx_infos)?.get_fees(&dictionary)?;
        dictionary
            .increment_accrued_fees_and_save_in_db(state.db, &fees)
            .and(Ok(state))
    }
}

pub fn account_for_fees_in_eos_tx_infos_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `Erc20OnEosEosTxInfo`!");
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!("✔ No `Erc20OnEosEosTxInfos` in state during ETH block submission ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accounting for fees in `Erc20OnEosEosTxInfos` during ETH block submission...");
        EosEthTokenDictionary::get_from_db(state.db).and_then(|ref dictionary| {
            let tx_infos = Erc20OnEosEosTxInfos::from_bytes(&state.tx_infos)?;
            Ok(state.add_tx_infos(tx_infos.subtract_fees(dictionary)?.to_bytes()?))
        })
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Accounting for fees in `Erc20OnEosEosTxInfos` during ETH block submission...");
    update_accrued_fees_in_dictionary_and_return_eth_state(state).and_then(account_for_fees_in_eos_tx_infos_in_state)
}
