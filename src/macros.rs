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
            }
        }
    }
}

