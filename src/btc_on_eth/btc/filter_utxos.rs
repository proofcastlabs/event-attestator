use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eth::btc::btc_state::BtcState,
    chains::btc::{
        btc_constants::MINIMUM_REQUIRED_SATOSHIS,
        filter_utxos::filter_out_utxos_whose_value_is_too_low,
        utxo_manager::utxo_types::{
            BtcUtxoAndValue,
            BtcUtxosAndValues,
        },
    },
};

pub fn filter_out_value_too_low_utxos_from_state<D>(
    state: BtcState<D>
) -> Result<BtcState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe filtering out any UTXOs below minimum # of Satoshis...");
    filter_out_utxos_whose_value_is_too_low(&state.utxos_and_values)
        .and_then(|utxos| state.replace_utxos_and_values(utxos))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::btc::btc_test_utils::get_sample_utxo_and_values;

    #[test]
    fn should_filter_utxos() {
        let expected_num_after_filtering = 3;
        let utxos = get_sample_utxo_and_values();
        let utxos_length_before = utxos.len();
        let result = filter_out_utxos_whose_value_is_too_low(&utxos).unwrap();
        let utxos_length_after = result.len();
        assert!(utxos_length_after < utxos_length_before);
        assert_ne!(utxos_length_before, utxos_length_after);
        assert_eq!(utxos_length_after, expected_num_after_filtering);
    }
}
