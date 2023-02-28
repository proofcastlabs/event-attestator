#[macro_export]
macro_rules! make_cli_args_struct {
    ($core_type:expr; $($name:ident => $type:ty),*) => {
        use serde::Deserialize;

        #[allow(non_snake_case)]
        #[derive(Clone, Default, Debug, PartialEq, Eq, Deserialize)]
        pub struct CliArgs {
            pub flag_file: String,
            pub flag_sig: String,
            pub arg_name: String,
            pub arg_blockJson: String,
            pub arg_debugSignersJson: String,
            pub cmd_debugGetAllDbKeys: bool,
            pub cmd_debugGetKeyFromDb: bool,
            pub cmd_debugAddDebugSigner: bool,
            pub cmd_debugAddDebugSigners: bool,
            pub cmd_debugRemoveDebugSigner: bool,
            pub cmd_debugSetKeyInDbToValue: bool,
            $( pub $name: $type,)*
        }

        impl CliArgs {
            // NOTE: The CLI arg parser will ALWAYS try and read a `blockJson` or `blocksJson` from
            // a path if extant, since they're used in ALL cores.
            pub fn parse(usage_info: &str) -> $crate::Result<Self> {
                Self::parse_from_usage_info(usage_info)
                    .and_then(|cli_args| cli_args.maybe_update_block_json())
            }

            fn parse_from_usage_info(usage_info: &str) -> $crate::Result<Self> {
                Ok($crate::docopt::Docopt::new(usage_info).and_then(|d| d.deserialize())?)
            }

            fn maybe_update_block_json(self) -> $crate::Result<Self> {
                if self.file_exists_at_path() {
                    self.read_file_to_string().map(|s| self.update_block_json(s))
                } else {
                    Ok(self)
                }
            }

            fn update_block_json(mut self, t: String) -> Self {
                self.arg_blockJson = t;
                self
            }

            pub fn read_file_to_string(&self) -> $crate::Result<String> {
                Ok(std::fs::read_to_string(&self.flag_file)?)
            }

            fn file_exists_at_path(&self) -> bool {
                std::path::Path::new(&self.flag_file).exists()
            }

            pub fn core_type() -> $crate::CoreType {
                $core_type
            }

            paste! {
                $(
                    // NOTE: These updaters are used in cases where supplied args are read from files etc.
                    pub fn [<update_ $name:snake:lower >](mut self, t: $type) -> Self {
                        self.$name = t;
                        self
                    }
                )*
            }
        }
    }
}
