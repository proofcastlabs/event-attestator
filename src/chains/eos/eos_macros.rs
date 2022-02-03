#[macro_export]
macro_rules! create_eos_safe_address_diversion_fxns {
    ($struct_name:expr => $state_name:expr => $tx_infos_name:expr => $($contract_name:expr),*) => {
        paste! {
            use crate::constants::SAFE_EOS_ADDRESS;

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
                fn update_destination_address(&self, new_address: &str) -> Self {
                    let mut new_self = self.clone();
                    new_self.destination_address = new_address.to_string();
                    new_self
                }

                fn divert_to_safe_address_if_destination_matches_address(&self, address: &str) -> Self {
                    if self.destination_address == address {
                        self.update_destination_address(SAFE_EOS_ADDRESS)
                    } else {
                        self.clone()
                    }
                }

                $(
                    pub fn [<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>](&self) -> Self {
                        info!("✔ Checking if the destination address matches the {} contract address...", $contract_name);
                        self.divert_to_safe_address_if_destination_matches_address(&self.[<eos_ $contract_name _address>])
                    }
                )*
            }

            $(
                pub fn [<maybe_divert_txs_to_safe_address_if_destination_is_ $contract_name _address>]<D: DatabaseInterface>(
                    state: [< $state_name State>]<D>,
                ) -> Result<[< $state_name State>]<D>> {
                    if state.[< $tx_infos_name >].is_empty() {
                        Ok(state)
                    } else {
                        info!("✔ Maybe diverting txs to safe address if destination matches {} address...", $contract_name);
                        let new_infos = state
                            .[< $tx_infos_name >]
                            .[<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>]();
                        state.[<replace_ $tx_infos_name >](new_infos)
                    }
                }
            )*

            #[cfg(test)]
            mod tests {
                use super::*;

                $(
                    #[test]
                    fn [<should_divert_to_safe_address_if_destination_is_ $contract_name _address_for $struct_name:snake >]() {
                        let mut info = [< $struct_name >]::default();
                        let destination_address = "someaddress".to_string();
                        info.destination_address = destination_address.clone();
                        info.[< eos_ $contract_name _address >] = destination_address.clone();
                        assert_eq!(info.destination_address, destination_address);
                        let result = info
                            .[<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>]();
                        assert_eq!(result.destination_address, *SAFE_EOS_ADDRESS);
                    }
                )*
            }
        }
    }
}
