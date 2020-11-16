use crate::{
    types::Result,
    traits::DatabaseInterface,
    constants::SAFE_EOS_ADDRESS,
    chains::btc::btc_state::BtcState,
    btc_on_eos::btc::minting_params::{
        BtcOnEosMintingParams,
        BtcOnEosMintingParamStruct
    },
};

fn filter_too_short_account_names(minting_params: &[BtcOnEosMintingParamStruct]) -> Result<BtcOnEosMintingParams> {
    Ok(BtcOnEosMintingParams::new( // FIXME Implement these filters on the type!
        minting_params
            .iter()
            .map(|params| {
                match params.to.is_empty() {
                    false => params.clone(),
                    true => {
                        info!("✘ Redirecting to safe address {:?} ∵ name too short:", params);
                        BtcOnEosMintingParamStruct {
                            amount: params.amount.clone(),
                            to: SAFE_EOS_ADDRESS.to_string(),
                            originating_tx_hash: params.originating_tx_hash.clone(),
                            originating_tx_address: params.originating_tx_address.clone(),
                        }
                    }
                }
            })
            .collect::<Vec<BtcOnEosMintingParamStruct>>()
    ))
}

pub fn maybe_filter_name_too_short_params_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Filtering out any minting params w/ too short account names...");
    filter_too_short_account_names(&state.btc_on_eos_minting_params)
        .and_then(|params| state.replace_btc_on_eos_minting_params(params))
}
