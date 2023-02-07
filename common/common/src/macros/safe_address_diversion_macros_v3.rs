// NOTE: These assume that tx infos stored in state are done as as _bytes_ rather than their
// actual type! This is to make the state generic over the various tx infos it has to hold.

#[macro_export]
macro_rules! impl_safe_address_diversion_fxn_v3 {
    (
        $thing_to_check:expr,
        $state_type:ty,
        $tx_info_name:ident
    ) => {
        paste! {
            pub fn [< divert_tx_infos_to_safe_address_if_destination_is_ $thing_to_check:snake:lower _address>]<D>(
                state: $state_type
            ) -> $crate::types::Result<$state_type>
                where D: $crate::traits::DatabaseInterface
            {
                info!(
                    "✔ Diverting tx infos if destination address is the {} address...",
                    $thing_to_check,
                );
                let tx_infos = [< $tx_info_name:camel s>]::from_bytes(&state.tx_infos)?;
                let filtered = [< $tx_info_name:camel s>]::new(
                    tx_infos
                        .iter()
                        .cloned()
                        .map(|tx_info| tx_info.[< divert_to_safe_address_if_destination_is_ $thing_to_check:snake:lower _address>]())
                        .collect::<Vec<[< $tx_info_name:camel >]>>()

                );
                Ok(state.add_tx_infos(filtered.to_bytes()?))
            }
        }
    }
}
