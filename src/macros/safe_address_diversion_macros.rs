/*
#[macro_export]
macro_rules! impl_safe_address_diversion_fxns_v2 {
    // TODO So this will impl the macro for a given thing that's gettable via the tx info trait
    // we should be generic of state so we'll need a marker trait for state
    // so we take state and return state.
    // we get the infos from state.
    // we update them if so
    // we replace them in state.
    //
}
*/

#[macro_export]
macro_rules! create_safe_address_diversion_fxns {
    ($struct_name:expr => $state:ty => $symbol:expr => $safe_address:expr => $address_type:ty => $($contract_name:expr),*) => {
        paste! {
            impl [< $struct_name s>] {
                $(
                    pub fn [<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>](&self) -> Self {
                        Self::new(
                            self.iter()
                                .map(|info| {
                                    info.[<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>]()
                                })
                                .collect::<Vec<[< $struct_name >]>>(),
                        )
                    }
                )*
            }

            impl [< $struct_name >] {
                fn update_destination_address(&self, new_address: $address_type) -> Self {
                    let mut new_self = self.clone();
                    new_self.destination_address = new_address;
                    new_self
                }

                fn divert_to_safe_address_if_destination_matches_address(&self, address: $address_type) -> Self {
                    if self.destination_address == address {
                        self.update_destination_address($safe_address)
                    } else {
                        self.clone()
                    }
                }

                $(
                    pub fn [<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>](&self) -> Self {
                        info!("✔ Checking if the destination address matches the {} contract address...", $contract_name);
                        self.divert_to_safe_address_if_destination_matches_address(self.[< $symbol:lower _ $contract_name _address>].clone())
                    }
                )*
            }

            $(
                pub fn [<maybe_divert_txs_to_safe_address_if_destination_is_ $contract_name _address>]<D: DatabaseInterface>(
                   state: $state<D>,
                ) -> Result<$state<D>> {
                    if state.[< $struct_name:snake s >].is_empty() {
                        Ok(state)
                    } else {
                        info!("✔ Maybe diverting txs to safe address if destination matches {} address...", $contract_name);
                        let new_infos = state
                            .[< $struct_name:snake s >]
                            .[<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>]();
                        state.[<replace_ $struct_name:snake s >](new_infos)
                    }
                }
            )*

            $(
                #[test]
                fn [<should_divert_ $struct_name:snake _to_safe_address_if_destination_is_ $contract_name _address>]() {
                    let info = [< $struct_name >]::default();
                    let result = info.[<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>]();
                    assert_eq!(result.destination_address, $safe_address);
                }
            )*
        }
    }
}
