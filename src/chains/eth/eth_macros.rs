#[macro_export]
macro_rules! create_eth_safe_address_diversion_fxns {
    ($struct_name:expr => $state_name:expr => $($contract_name:expr),*) => {
        paste! {
            use crate::constants::SAFE_ETH_ADDRESS;

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
                fn update_destination_address(&self, new_address: EthAddress) -> Self {
                    let mut new_self = self.clone();
                    new_self.destination_address = new_address;
                    new_self
                }

                fn divert_to_safe_address_if_destination_matches_address(&self, address: &EthAddress) -> Self {
                    if self.destination_address == *address {
                        self.update_destination_address(*SAFE_ETH_ADDRESS)
                    } else {
                        self.clone()
                    }
                }

                $(
                    pub fn [<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>](&self) -> Self {
                        info!("✔ Checking if the destination address matches the {} contract address...", $contract_name);
                        self.divert_to_safe_address_if_destination_matches_address(&self.[<eth_ $contract_name _address>])
                    }
                )*
            }

            $(
                pub fn [<maybe_divert_txs_to_safe_address_if_destination_is_ $contract_name _address>]<D: DatabaseInterface>(
                    state: [< $state_name State>]<D>,
                ) -> Result<[< $state_name State>]<D>> {
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

            #[cfg(test)]
            mod tests {
                use super::*;
                use crate::chains::eth::eth_utils::convert_hex_to_eth_address;

                $(
                    #[test]
                    fn [<should_divert_to_safe_address_if_destination_is_ $contract_name _address_for $struct_name:snake >]() {
                        let mut info = [< $struct_name >]::default();
                        let eth_address = "0x89ab32156e46f46d02ade3fecbe5fc4243b9aaed";
                        let destination_address = convert_hex_to_eth_address(eth_address).unwrap();
                        info.destination_address = destination_address.clone();
                        info.[< eth_ $contract_name _address >] = destination_address.clone();
                        assert_eq!(info.destination_address, destination_address);
                        let result = info
                            .[<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>]();
                        assert_eq!(result.destination_address, *SAFE_ETH_ADDRESS);
                    }
                )*
            }
        }
    }
}
