#[macro_export]
// NOTE: This makes asserting equality of large structs much easier in tests, since it'll show any
// fields that are mismatching, rather than just displaying the entire struct making you have to
// hunt through it. For non-tests, the equality check is made with no such assertions.
macro_rules! make_struct_with_test_assertions_on_equality_check {
    (struct $struct_name:ident { $($field_name:ident : $field_type:ty),*$(,)?}) => {

        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $struct_name { $(pub $field_name : $field_type),* }

        impl PartialEq for $struct_name {
            fn eq(&self, other: &Self) -> bool {
                if cfg!(test) {
                    $(
                        if stringify!($field_name).contains("timestamp") {
                            // NOTE: We skip the assertion if the field is a timestamp since it's (probably)
                            // not deterministic...
                            assert!(true);
                        } else if stringify!($field_name).contains("eos_serialized_tx") {
                            // NOTE: We skip this and must assert it manually since it has a
                            // timestamp encoded in it....
                            assert!(true);
                        } else if stringify!($field_name).contains("eos_tx_signature") {
                            // NOTE: We skip this because it's not deterministic...
                            assert!(true);
                        } else {
                            assert_eq!(
                                self.$field_name,
                                other.$field_name,
                                "{}",
                                format!("`{}` field is not equal!", stringify!($field_name))
                            );
                        }
                    )*
                    // NOTE: Now we can return true since if false one of the above assertions
                    // would have panicked.
                    true
                } else {
                    self == other
                }
            }
        }

        impl Eq for $struct_name {}
    }
}
