#[macro_export]
macro_rules! create_db_utils_with_getters {
    ($prefix:expr; $($key:expr => $value:expr),*) => {
        paste! {
            lazy_static! {
                $(
                    static ref [< $prefix:upper $key:upper >]: [u8; 32] =
                        crate::utils::get_prefixed_db_key($value);
                )*
            }

            #[allow(non_snake_case)]
            #[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
            pub struct [< $prefix:camel DatabaseKeysJson >] {
                $([< $prefix:upper $key:upper >]: String,)*
            }

            impl [< $prefix:camel DatabaseKeysJson >] {
                pub fn new() -> Self {
                    Self {
                        $([< $prefix:upper $key:upper >]: hex::encode(&*[< $prefix:upper $key:upper >]),)*
                    }
                }
            }

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct [< $prefix:camel DbUtils>]<'a, D: DatabaseInterface> {
                db: &'a D,
                $([< $prefix:lower $key:lower>]: Bytes,)*
            }

            impl<'a, D: DatabaseInterface>[< $prefix:camel DbUtils>]<'a, D> {
                pub fn new(db: &'a D) -> Self {
                    Self {
                        db,
                        $([< $prefix:lower $key:lower >]: [< $prefix:upper $key:upper >].to_vec(),)*
                    }
                }

                pub fn get_db(&self) -> &D {
                    self.db
                }

                $(
                    pub fn [< get_ $prefix:lower $key:lower >](&self) -> Bytes {
                        self.[< $prefix:lower $key:lower >].clone()
                    }
                )*

                #[cfg(test)]
                fn get_all_db_keys_as_hex() -> Vec<String> {
                    vec![$(hex::encode(&*[< $prefix:upper $key:upper >]),)*]
                }
            }

            #[cfg(test)]
            mod [< $prefix:lower _db_utils_tests>] {
                use super::*;

                #[test]
                fn [<$prefix:lower _should_not_have_any_db_key_collisions>]() {
                    use crate::test_utils::TestDB;
                    let keys = [< $prefix:camel DbUtils>]::<'_, TestDB>::get_all_db_keys_as_hex();
                    let mut deduped_keys = keys.clone();
                    deduped_keys.sort();
                    deduped_keys.dedup();
                    assert_eq!(deduped_keys.len(), keys.len());
                }
            }
        }
    }
}

macro_rules! create_diversion_fxns {
    ($struct_name:expr => $state_name:expr => $tx_infos_name:expr => $($contract_name:expr),*) => {
        paste! {
            impl [< $struct_name s>] {
                $(
                    pub fn [<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>](&self) -> Self {
                        Self::new(
                            self.iter()
                                .map(|info| {
                                    info.[<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>]()
                                })
                                .collect::<Vec<[<$struct_name>]>>(),
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
                    if state.[< $tx_infos_name >].is_empty() {
                        Ok(state)
                    } else {
                        info!("✔ Maybe diverting txs to safe address if destination matches {} address...", $contract_name);
                        let new_infos = state
                            .[< $tx_infos_name >]
                            .[<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>]();
                        state.[<replace_ $tx_infos_name>](new_infos)
                    }
                }
            )*
        }
    }
}
