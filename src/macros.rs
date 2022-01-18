#[macro_export]
macro_rules! create_db_keys_and_json {
    ($prefix:expr; $($key:expr => $value:expr),*) => {
        paste! {
            lazy_static! {
                $(pub static ref [< $key:upper >]: [u8; 32] = crate::utils::get_prefixed_db_key($value);)*
            }

            #[allow(non_snake_case)]
            #[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
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
