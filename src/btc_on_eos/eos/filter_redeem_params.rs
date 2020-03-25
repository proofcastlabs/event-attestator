use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    constants::MINIMUM_REQUIRED_SATOSHIS,
    eos::{
        eos_state::EosState,
        eos_types::RedeemParams,
    },
};

fn filter_redeem_params(
    redeem_params: &Vec<RedeemParams>,
) -> Result<Vec<RedeemParams>> {
    Ok(
        redeem_params
            .iter()
            .map(|params| params.amount)
            .collect::<Vec<u64>>()
            .into_iter()
            .zip(redeem_params.iter())
            .filter(|(amount, params)| {
                match amount >= &MINIMUM_REQUIRED_SATOSHIS {
                    true => true,
                    false => {
                        info!(
                            "✘ Filtering redeem params ∵ value too low: {:?}",
                            params,
                        );
                        false
                    }
                }
            })
            .map(|(_, params)| params)
            .cloned()
            .collect::<Vec<RedeemParams>>()
    )
}

pub fn maybe_filter_value_too_low_redeem_params_in_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out any redeem params below minimum # of Satoshis...");
    filter_redeem_params(&state.redeem_params)
        .and_then(|new_params| state.replace_redeem_params(new_params))
}

/* TODO Get test vector for EOS side and re-instate this test!
#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eos::btc::btc_test_utils::get_sample_minting_params;

    #[test]
    fn should_filter_minting_params() {
        let expected_length_before = 3;
        let expected_length_after = 2;
        let redeem_params = get_sample_minting_params();
        let length_before = redeem_params.len();
        assert_eq!(length_before, expected_length_before);
        let result = filter_redeem_params(&redeem_params)
            .unwrap();
        let length_after = result.len();
        assert_eq!(length_after, expected_length_after);
        result
            .iter()
            .map(|params|
                 assert!(
                     convert_eos_asset_to_u64(&params.amount).unwrap() >=
                     MINIMUM_REQUIRED_SATOSHIS
                 )
             )
            .for_each(drop);
    }
}
*/
