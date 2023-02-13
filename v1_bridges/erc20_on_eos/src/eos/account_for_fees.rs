use common::{
    dictionaries::eos_eth::EosEthTokenDictionary,
    fees::fee_constants::DISABLE_FEES,
    traits::DatabaseInterface,
    types::Result,
};
use common_eos::EosState;

use crate::{eos::Erc20OnEosEthTxInfos, fees_calculator::FeesCalculator};

pub fn update_accrued_fees_in_dictionary_and_return_eos_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `Erc20OnEosEthTxInfos`!");
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!("✔ No `Erc20OnEosEthTxInfos` in state during EOS block submission ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accruing fees during EOS block submission...");
        let dictionary = EosEthTokenDictionary::get_from_db(state.db)?;
        let fees = Erc20OnEosEthTxInfos::from_bytes(&state.tx_infos)?.get_fees(&dictionary)?;
        dictionary
            .increment_accrued_fees_and_save_in_db(state.db, &fees)
            .and(Ok(state))
    }
}

pub fn account_for_fees_in_eth_tx_infos_in_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `Erc20OnEthRedeemInfos`!");
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!("✔ No `Erc20OnEthRedeemInfos` in state during EOS block submission ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accounting for fees in `Erc20OnEthRedeemInfos` during EOS block submission...");
        let dictionary = EosEthTokenDictionary::get_from_db(state.db)?;
        let infos = Erc20OnEosEthTxInfos::from_bytes(&state.tx_infos)?;
        infos
            .subtract_fees(&dictionary)
            .and_then(|updated_infos| updated_infos.to_bytes())
            .map(|bytes| state.add_tx_infos(bytes))
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Accounting for fees in `Erc20OnEosEthTxInfos` during EOS block submission...");
    update_accrued_fees_in_dictionary_and_return_eos_state(state).and_then(account_for_fees_in_eth_tx_infos_in_state)
}
