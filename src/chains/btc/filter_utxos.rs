use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::btc::{
        btc_constants::MINIMUM_REQUIRED_SATOSHIS,
        utxo_manager::{
            utxo_utils::utxos_exist_in_db,
            utxo_types::{
                BtcUtxoAndValue,
                BtcUtxosAndValues,
            },
        },
    },
};

pub fn filter_out_utxos_extant_in_db<D>(
    db: &D,
    utxos: &[BtcUtxoAndValue]
) -> Result<BtcUtxosAndValues>
    where D: DatabaseInterface
{
    utxos_exist_in_db(db, utxos)
        .map(|bool_arr| {
            utxos
                .iter()
                .enumerate()
                .filter(|(i, _)| bool_arr[*i])
                .map(|(_, utxo)| utxo)
                .cloned()
                .collect::<BtcUtxosAndValues>()
        })
}

pub fn filter_out_utxos_whose_value_is_too_low(utxos: &[BtcUtxoAndValue]) -> Result<BtcUtxosAndValues> {
    Ok(
        utxos
            .iter()
            .filter(|utxo| {
                match utxo.value >= MINIMUM_REQUIRED_SATOSHIS {
                    true => true,
                    false => {
                        info!("✘ Filtering UTXO ∵ value too low: {:?}", utxo);
                        false
                    }
                }
            })
            .cloned()
            .collect::<BtcUtxosAndValues>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::btc::btc_test_utils::get_sample_utxo_and_values;
    // TODO Start using chains::btc::test_utils!

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
