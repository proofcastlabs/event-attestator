#[macro_export]
macro_rules! create_db_keys_and_json {
    ($prefix:expr; $($key:expr => $value:expr),*) => {
        use serde::{Deserialize, Serialize};
        use crate::utils::get_prefixed_db_key;
        paste! {
            lazy_static! {
                $(pub static ref [< $key:upper >]: [u8; 32] = get_prefixed_db_key($value);)*
            }

            #[allow(non_snake_case)]
            #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
            pub struct [< $prefix:camel DatabaseKeysJson >] {
                $([< $key:upper >]: String,)*
            }

            impl [< $prefix:camel DatabaseKeysJson >] {
                pub fn new() -> Self {
                    Self {
                        $([< $key:upper >]: hex::encode(&*[< $key:upper >]),)*
                    }
                }
            }
        }
    }
}
