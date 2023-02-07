#[macro_export]
macro_rules! make_output_structs {
    ($output_symbol:ident, $tx_symbol:ident) => {
        paste! {
            #[allow(clippy::redundant_field_names)]
            #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
            pub struct [< $output_symbol:camel Output>] {
                pub [< $output_symbol:lower _latest_block_number >]: usize,
                pub [< $tx_symbol:lower _signed_transactions >]: Vec<[< $tx_symbol:camel TxInfo>]>,
            }

            impl [< $output_symbol:camel Output>] {
                // NOTE: We could derive `constructor` here but it gives rise to linting errors due
                // to `redundant_field_names`. This manual impl has no such issue.
                pub fn new(
                    [< $output_symbol:lower _latest_block_number >]: usize,
                    [< $tx_symbol:lower _signed_transactions >]: Vec<[< $tx_symbol:camel TxInfo>]>,
                ) -> Self {
                    Self {
                        [< $tx_symbol:lower _signed_transactions >],
                        [<$output_symbol:lower _latest_block_number >],
                    }
                }
            }

            #[derive(
                Eq,
                Debug,
                PartialEq,
                serde::Serialize,
                serde::Deserialize,
                derive_more::Deref,
                derive_more::Constructor
            )]
            pub struct [< $output_symbol:camel Output s >](Vec<[< $output_symbol:camel Output>]>);

            impl [< $output_symbol:camel Output s>] {
                pub fn to_output(&self) -> [< $output_symbol:camel Output>] {
                    let latest_block_number = match self.last() {
                        Some(output) => output.[< $output_symbol:lower _latest_block_number >],
                        // NOTE: This field isn't actually used anywhere, so it's safe to default to zero here.
                        None => 0,
                    };
                    let tx_infos = self
                        .iter()
                        .map(|output| output.[< $tx_symbol:lower _signed_transactions >].clone())
                        .collect::<Vec<Vec<_>>>()
                        .concat();
                    [< $output_symbol:camel Output>]::new(latest_block_number, tx_infos)
                }
            }

            impl std::fmt::Display for [< $output_symbol:camel Output>] {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(
                        f,
                        "{}",
                        serde_json::to_string(self)
                            .unwrap_or_else(|_|
                                format!(
                                    "{{\"error\": \"Could not convert `{}` to string!\"}}",
                                    stringify!([< $output_symbol:camel Output>]),
                                ).into()
                            )
                    )
                }
            }

            #[cfg(test)]
            impl std::str::FromStr for [< $output_symbol:camel Output>] {
                type Err = $crate::errors::AppError;

                fn from_str(s: &str) -> Result<Self> {
                    #[derive(serde::Serialize, serde::Deserialize)]
                    struct Interim {
                        [< $output_symbol:lower _latest_block_number >]: usize,
                            [< $tx_symbol:lower _signed_transactions >]: Vec<serde_json::Value>,
                    }
                    let interim = serde_json::from_str::<Interim>(s)?;
                    let tx_infos = interim
                        .[< $tx_symbol:lower _signed_transactions >]
                        .iter()
                        .map(|json| [< $tx_symbol:camel TxInfo>]::from_str(&json.to_string()))
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Self::new(interim.[< $output_symbol:lower _latest_block_number >], tx_infos))
                }
            }

            #[cfg(test)]
            impl std::str::FromStr for [< $tx_symbol:camel TxInfo>] {
                type Err = $crate::errors::AppError;

                fn from_str(s: &str) -> $crate::types::Result<Self> {
                    Ok(serde_json::from_str(s)?)
                }
            }

        }
    };
}
