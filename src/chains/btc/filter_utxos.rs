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
