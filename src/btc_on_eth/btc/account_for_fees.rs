use crate::{
    btc_on_eth::btc::minting_params::BtcOnEthMintingParams,
    chains::btc::btc_state::BtcState,
    fees::{
        fee_constants::DISABLE_FEES,
        fee_database_utils::{get_btc_on_eth_peg_in_basis_points_from_db, increment_btc_on_eth_accrued_fees},
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn subtract_fees_from_minting_params(
    minting_params: &BtcOnEthMintingParams,
    fee_basis_points: u64,
) -> BtcOnEthMintingParams {
    let (fees, _) = minting_params.calculate_fees(fee_basis_points);
    info!("BTC `MintingParam` fees: {:?}", fees);
    BtcOnEthMintingParams::new(
        fees.iter()
            .zip(minting_params.iter())
            .map(|(fee, minting_params)| minting_params.subtract_satoshi_amount(*fee))
            .collect(),
    )
}

fn accrue_fees_from_minting_params<D: DatabaseInterface>(
    db: &D,
    minting_params: &BtcOnEthMintingParams,
    fee_basis_points: u64,
) -> Result<()> {
    let (_, total_fee) = minting_params.calculate_fees(fee_basis_points);
    info!("BTC `MintingParams` total fee: {}", total_fee);
    increment_btc_on_eth_accrued_fees(db, total_fee)
}

fn account_for_fees_in_minting_params<D: DatabaseInterface>(
    db: &D,
    minting_params: &BtcOnEthMintingParams,
    fee_basis_points: u64,
) -> Result<BtcOnEthMintingParams> {
    if fee_basis_points == 0 {
        info!("✔ `BTC-on-ETH` peg-in fees are set to zero ∴ not taking any fees!");
        Ok(minting_params.clone())
    } else {
        info!("✔ Accounting for fees @ {} basis points...", fee_basis_points);
        accrue_fees_from_minting_params(db, minting_params, fee_basis_points)
            .map(|_| subtract_fees_from_minting_params(minting_params, fee_basis_points))
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Maybe accounting for fees...");
    if DISABLE_FEES {
        info!("✔ Taking fees is disabled ∴ not taking any fees!");
        Ok(state)
    } else if state.btc_on_eth_minting_params.is_empty() {
        info!("✔ Not minting-params in state ∴ not taking any fees!");
        Ok(state)
    } else {
        account_for_fees_in_minting_params(
            &state.db,
            &state.btc_on_eth_minting_params,
            get_btc_on_eth_peg_in_basis_points_from_db(&state.db)?,
        )
        .and_then(|updated_minting_params| state.replace_btc_on_eth_minting_params(updated_minting_params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        btc_on_eth::utils::convert_satoshis_to_wei,
        chains::btc::btc_test_utils::get_sample_minting_params,
        fees::fee_database_utils::get_btc_on_eth_accrued_fees_from_db,
        test_utils::get_test_database,
    };

    #[test]
    fn should_account_for_fees_in_btc_on_eth_minting_params() {
        let fee_basis_points = 25;
        let db = get_test_database();
        let accrued_fees_before = get_btc_on_eth_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_before, 0);
        let minting_params = get_sample_minting_params();
        let (_, total_fee) = minting_params.calculate_fees(fee_basis_points);
        let expected_total_fee = 36;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = minting_params.sum();
        let resulting_params = account_for_fees_in_minting_params(&db, &minting_params, fee_basis_points).unwrap();
        let total_value_after = resulting_params.sum();
        let accrued_fees_after = get_btc_on_eth_accrued_fees_from_db(&db).unwrap();
        let expected_amount_after_1 = convert_satoshis_to_wei(4988);
        let expected_amount_after_2 = convert_satoshis_to_wei(4989);
        let expected_amount_after_3 = convert_satoshis_to_wei(4987);
        assert_eq!(
            total_value_after + convert_satoshis_to_wei(total_fee),
            total_value_before
        );
        assert_eq!(accrued_fees_after, total_fee);
        assert_eq!(resulting_params[0].amount, expected_amount_after_1);
        assert_eq!(resulting_params[1].amount, expected_amount_after_2);
        assert_eq!(resulting_params[2].amount, expected_amount_after_3);
    }

    #[test]
    fn should_not_account_for_fees_if_basis_points_are_zero() {
        let fee_basis_points = 0;
        assert_eq!(fee_basis_points, 0);
        let db = get_test_database();
        let accrued_fees_before = get_btc_on_eth_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_before, 0);
        let minting_params = get_sample_minting_params();
        let (_, total_fee) = minting_params.calculate_fees(fee_basis_points);
        let expected_total_fee = 0;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = minting_params.sum();
        let resulting_params = account_for_fees_in_minting_params(&db, &minting_params, fee_basis_points).unwrap();
        let total_value_after = resulting_params.sum();
        assert_eq!(total_value_before, total_value_after);
        let accrued_fees_after = get_btc_on_eth_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_after, 0);
    }

    #[test]
    fn should_account_for_fees_correctly_if_minting_params_are_emtpy() {
        let fee_basis_points = 25;
        assert!(fee_basis_points > 0);
        let db = get_test_database();
        let accrued_fees_before = get_btc_on_eth_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_before, 0);
        let minting_params = BtcOnEthMintingParams::new(vec![]);
        let (_, total_fee) = minting_params.calculate_fees(fee_basis_points);
        let expected_total_fee = 0;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = minting_params.sum();
        let resulting_params = account_for_fees_in_minting_params(&db, &minting_params, fee_basis_points).unwrap();
        let total_value_after = resulting_params.sum();
        assert_eq!(total_value_before, total_value_after);
        let accrued_fees_after = get_btc_on_eth_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_after, 0);
    }
}
