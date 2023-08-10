#[macro_export]
macro_rules! make_topics {
    ($($name:ident => $hex:expr),* $(,)?) => {
        lazy_static! {
            $(
                pub static ref $name: ethereum_types::H256 = {
                    ethereum_types::H256::from_slice(&hex::decode($hex)
                        .expect(&format!("invalid hex in `{}`!", stringify!($name))),
                    )
                };
            )*
        }
    };
}

#[macro_export]
macro_rules! impl_to_erc20_token_event {
    ($path:path, $value:ident, $to:ident, $from:ident, $token_address:ident) => {
        use $crate::{Erc20TokenTransferEvent, ToErc20TokenTransferEvent};

        impl ToErc20TokenTransferEvent for $path {
            fn to_erc20_token_transfer_event(&self) -> Erc20TokenTransferEvent {
                Erc20TokenTransferEvent::new(self.$value, self.$to, self.$from, self.$token_address)
            }
        }
    };
}

#[macro_export]
macro_rules! make_erc20_token_event_filterer {
    ($state:ty, $db_utils:ident, $tx_infos_field:ident) => {
        use paste;

        paste! {
            use $crate::{
                EthState,
                Erc20TokenTransferEvents,
                EthDbUtilsExt,
            };
            use common::{
                traits::DatabaseInterface,
                types::Result,
            };

            pub fn filter_tx_info_with_no_erc20_transfer_event<D: DatabaseInterface>(
                state: $state
            ) -> Result<$state> {
                info!("✔ Filtering out tx infos which don't have corresponding ERC20 transfer events ...");
                if state.tx_infos.is_empty() {
                    warn!("✔ NOT filtering because no tx infos exist!");
                    Ok(state)
                } else {
                    let tx_infos = [< $tx_infos_field:camel >]::from_bytes(&state.tx_infos)?;
                    state
                        .$db_utils
                        .get_eth_canon_block_from_db()
                        .map(|submission_material| {
                            Erc20TokenTransferEvents::filter_if_no_transfer_event_in_submission_material(
                                &submission_material,
                                &tx_infos,
                            )
                        })
                        .map([< $tx_infos_field:camel >]::new)
                        .and_then(|filtered| filtered.to_bytes())
                        .map(|bytes| state.add_tx_infos(bytes))
                }
            }

            pub fn debug_filter_tx_info_with_no_erc20_transfer_event<D: DatabaseInterface>(
                state: $state
            ) -> Result<$state> {
                info!("✔ Debug filtering out tx infos which don't have corresponding ERC20 transfer events ...");
                // NOTE: These filterers are to be used in debug block reprocessors  A reprocess
                // is like a submission with 0 confs, ∴ we need to check the _current_ submission material,
                // not the canon block material!
                let tx_infos = [< $tx_infos_field:camel >]::from_bytes(&state.tx_infos)?;
                state
                    .get_eth_submission_material()
                    .map(|submission_material| {
                        Erc20TokenTransferEvents::filter_if_no_transfer_event_in_submission_material(
                            submission_material,
                            &tx_infos,
                        )
                    })
                    .map([< $tx_infos_field:camel >]::new)
                    .and_then(|filtered| filtered.to_bytes())
                    .map(|bytes| state.add_tx_infos(bytes))
            }
        }
    };
}
