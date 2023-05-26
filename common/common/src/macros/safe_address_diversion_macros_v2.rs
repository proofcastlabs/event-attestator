#[macro_export]
macro_rules! impl_safe_address_diversion_fxn_v2 {
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
                let tx_infos = state.[< $tx_info_name s>].clone();
                let filtered = [< $tx_info_name:camel s>]::new(
                    tx_infos
                        .iter()
                        .cloned()
                        .map(|tx_info| tx_info.[< divert_to_safe_address_if_destination_is_ $thing_to_check:snake:lower _address>]())
                        .collect::<Vec<[< $tx_info_name:camel >]>>()

                );
                state.[< replace_ $tx_info_name s >](filtered)
            }
        }
    }
}
