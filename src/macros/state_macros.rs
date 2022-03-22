#[macro_export]
macro_rules! make_state_setters_and_getters {
    ($state:ty; $($field_name:expr => $field_type:ty),*) => {
        paste! {
            impl <'a, D: DatabaseInterface> $state<'a, D> {
                $(
                    pub fn [<add_ $field_name>](mut self, thing: $field_type) -> Result<Self> {
                        info!("✔ Adding `{}` to state...", $field_name);
                        match self.[< $field_name:snake:lower >] {
                            Some(_) => Err(crate::utils::get_no_overwrite_state_err($field_name).into()),
                            None => {
                                self.[< $field_name >] = Some(thing);
                                Ok(self)
                            },
                        }
                    }

                    pub fn [<get_ $field_name>](&self) -> Result<&$field_type> {
                        info!("✔ Getting `{}` from state...", $field_name);
                        match self.[< $field_name:snake:lower >] {
                            Some(ref thing) => Ok(thing),
                            None => Err(crate::utils::get_not_in_state_err($field_name).into()),
                        }
                    }
                )*
            }
        }
    }
}
