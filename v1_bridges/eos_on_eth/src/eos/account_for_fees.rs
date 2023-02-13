use common::{
    dictionaries::eos_eth::EosEthTokenDictionary,
    fees::fee_constants::DISABLE_FEES,
    traits::DatabaseInterface,
    types::Result,
};
use common_eos::EosState;

use crate::{eos::EosOnEthEthTxInfos, fees_calculator::FeesCalculator};

pub fn update_accrued_fees_in_dictionary_and_return_eos_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `EosOnEthEthTxInfos`!");
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!("✔ No `EosOnEthEthTxInfos` in state during EOS block submission ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accruing fees during EOS block submission...");
        let dictionary = EosEthTokenDictionary::get_from_db(state.db)?;
        let infos = EosOnEthEthTxInfos::from_bytes(&state.tx_infos)?;
        dictionary
            .increment_accrued_fees_and_save_in_db(state.db, &infos.get_fees(&dictionary)?)
            .and(Ok(state))
    }
}

pub fn account_for_fees_in_eos_tx_infos_in_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `EosOnEthEthTxInfos`!");
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!("✔ No `EosOnEthEthTxInfos` in state during EOS block submission ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accounting for fees in `EosOnEthEthTxInfos` during EOS block submission...");
        let dictionary = EosEthTokenDictionary::get_from_db(state.db)?;
        let infos = EosOnEthEthTxInfos::from_bytes(&state.tx_infos)?;
        infos
            .subtract_fees(&dictionary)
            .and_then(|infos| infos.to_bytes())
            .map(|bytes| state.add_tx_infos(bytes))
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Accounting for fees in `EosOnEthEthTxInfos` during EOS block submission...");
    update_accrued_fees_in_dictionary_and_return_eos_state(state).and_then(account_for_fees_in_eos_tx_infos_in_state)
}
