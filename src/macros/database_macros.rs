#[macro_export]
macro_rules! create_db_utils_with_getters {
    ($prefix:expr; $($key:expr => $value:expr),*) => {
        paste! {
            lazy_static! {
                $(
                    static ref [< $prefix:upper $key:upper >]: [u8; 32] =
                        $crate::utils::get_prefixed_db_key($value);
                )*
            }

            #[allow(non_snake_case)]
            #[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
            pub struct [< $prefix:camel DatabaseKeysJson >] {
                HOST_CORE_IS_INITIALIZED_DB_KEY: String,
                NATIVE_CORE_IS_INITIALIZED_DB_KEY: String,
                $([< $prefix:upper $key:upper >]: String,)*
            }

            impl [< $prefix:camel DatabaseKeysJson >] {
                pub fn new() -> Self {
                    Self {
                        HOST_CORE_IS_INITIALIZED_DB_KEY:
                            hex::encode(&*$crate::core_type::HOST_CORE_IS_INITIALIZED_DB_KEY),
                        NATIVE_CORE_IS_INITIALIZED_DB_KEY:
                            hex::encode(&*$crate::core_type::NATIVE_CORE_IS_INITIALIZED_DB_KEY),
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
                    use $crate::test_utils::TestDB;
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
