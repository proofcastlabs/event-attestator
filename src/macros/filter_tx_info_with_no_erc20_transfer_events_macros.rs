macro_rules! impl_to_erc20_token_event {
    ($path:path, $value:ident, $to:ident, $from:ident, $token_address:ident) => {
        use $crate::chains::eth::eth_contracts::erc20_token::{Erc20TokenTransferEvent, ToErc20TokenTransferEvent};

        impl ToErc20TokenTransferEvent for $path {
            fn to_erc20_token_transfer_event(&self) -> Erc20TokenTransferEvent {
                Erc20TokenTransferEvent::new(self.$value, self.$to, self.$from, self.$token_address)
            }
        }
    };
}

macro_rules! make_erc20_token_event_filterer {
    ($state:ty, $db_utils:ident, $tx_infos_field:ident) => {
        use paste;

        paste! {
            use $crate::{
                chains::eth::{
                    eth_state::EthState,
                    eth_contracts::erc20_token::Erc20TokenTransferEvents,
                    eth_database_utils::EthDbUtilsExt,
                },
                traits::DatabaseInterface,
                types::Result,
            };

            pub fn filter_tx_info_with_no_erc20_transfer_event<D: DatabaseInterface>(
                state: $state
            ) -> Result<$state> {
                info!("✔ Filtering out tx infos which don't have corresponding ERC20 transfer events ...");
                state
                    .$db_utils
                    .get_eth_canon_block_from_db()
                    .map(|submission_material| {
                        Erc20TokenTransferEvents::filter_if_no_transfer_event_in_submission_material(
                            &submission_material,
                            &state.[< $tx_infos_field:snake >],
                        )
                    })
                    .map([< $tx_infos_field:camel >]::new)
                    .and_then(|filtered| state.[< replace_ $tx_infos_field:snake >](filtered))
            }

            pub fn debug_filter_tx_info_with_no_erc20_transfer_event<D: DatabaseInterface>(
                state: $state
            ) -> Result<$state> {
                info!("✔ Debug filtering out tx infos which don't have corresponding ERC20 transfer events ...");
                // NOTE: These filterers are to be used in debug block reprocessors  A reprocess
                // is like a submission with 0 confs, ∴ we need to check the _current_ submission material,
                // not the canon block material!
                state
                    .get_eth_submission_material()
                    .map(|submission_material| {
                        Erc20TokenTransferEvents::filter_if_no_transfer_event_in_submission_material(
                            submission_material,
                            &state.[< $tx_infos_field:snake >],
                        )
                    })
                    .map([< $tx_infos_field:camel >]::new)
                    .and_then(|filtered| state.[< replace_ $tx_infos_field:snake >](filtered))
            }
        }
    };
}
