
use common::{
    dictionaries::eos_eth::EosEthTokenDictionary,
    fees::fee_constants::DISABLE_FEES,
    traits::DatabaseInterface,
    types::Result,
};
use common_eth::EthState;

use crate::{eth::EosOnEthEosTxInfos, fees_calculator::FeesCalculator};

pub fn update_accrued_fees_in_dictionary_and_return_eth_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `EosOnEthEosTxInfos`!");
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!("✔ No `EosOnEthEosTxInfos` in state during ETH block submission ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accruing fees during ETH block submission...");
        let infos = EosOnEthEosTxInfos::from_bytes(&state.tx_infos)?;
        EosEthTokenDictionary::get_from_db(state.db)
            .and_then(|dictionary| {
                dictionary.increment_accrued_fees_and_save_in_db(state.db, &infos.get_fees(&dictionary)?)
            })
            .and(Ok(state))
    }
}

pub fn account_for_fees_in_eth_tx_infos_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `EosOnEthEosTxInfos`!");
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!("✔ No `EosOnEthEosTxInfos` in state during ETH block submission ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accounting for fees in `EosOnEthEosTxInfos` during ETH block submission...");
        EosEthTokenDictionary::get_from_db(state.db).and_then(|ref dictionary| {
            let tx_infos = EosOnEthEosTxInfos::from_bytes(&state.tx_infos)?;
            let updated_infos = tx_infos.subtract_fees(dictionary)?;
            let bytes = updated_infos.to_bytes()?;
            Ok(state.add_tx_infos(bytes))
        })
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Accounting for fees in `EosOnEthEosTxInfos` during ETH block submission...");
    update_accrued_fees_in_dictionary_and_return_eth_state(state).and_then(account_for_fees_in_eth_tx_infos_in_state)
}
