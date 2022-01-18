#[macro_export]
macro_rules! create_db_utils {
    ($prefix:expr; $($key:expr => $value:expr),*) => {
        paste! {
            lazy_static! {
                $(
                    pub static ref [< $prefix:upper $key:upper >]: [u8; 32] =
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

                #[cfg(test)]
                pub fn get_all_db_keys_as_hex() -> Vec<String> {
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
