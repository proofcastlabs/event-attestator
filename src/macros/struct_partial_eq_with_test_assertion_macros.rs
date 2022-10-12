#[macro_export]

// NOTE: This makes asserting equality of large structs much easier in tests, since it'll show any
// fields that are mismatching, rather than just displaying the entire struct making you have to
// hunt through it. For non-tests, the equality check is made with no such assertions.
macro_rules! impl_partial_eq_with_test_assertions_for_struct {
    ($struct:ty; $($name:ident),*) => {
        impl PartialEq for $struct {
            fn eq(&self, other: &Self) -> bool {
                if cfg!(test) {
                    $(
                        assert_eq!(
                            self.$name,
                            other.$name,
                            "{}",
                            format!("`{}` field is not equal!", stringify!($name))
                        );
                    )*
                true
                } else {
                    self == other
                }
            }
        }

        impl Eq for $struct {}
    }
}
