#[macro_export]

macro_rules! make_plural_output_struct {
    (
        $tx_output_struct_name:ident,
        $tx_info_struct_name:ident,
        $txs_field_name:ident,
        $latest_block_number_field_name:ident
    ) => {
        paste! {

            #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, derive_more::Constructor)]
            pub struct $tx_output_struct_name {
                pub $latest_block_number_field_name: usize,
                pub $txs_field_name: Vec<$tx_info_struct_name>,
            }

            #[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Deref, Constructor)]
            pub struct [<$tx_output_struct_name s>](Vec<$tx_output_struct_name>);
            impl [< $tx_output_struct_name s>] {
                pub fn to_output(&self) -> $tx_output_struct_name {
                    let latest_block_number = match self.last() {
                        Some(output) => output.$latest_block_number_field_name,
                        // NOTE: This field isn't actually used anywhere, so it's safe to default to zero here.
                        None => 0,
                    };
                    let tx_infos = self
                        .iter()
                        .map(|output| output.$txs_field_name.clone())
                        .collect::<Vec<Vec<_>>>()
                        .concat();
                    $tx_output_struct_name::new(latest_block_number, tx_infos)
                }
            }

            impl std::fmt::Display for $tx_output_struct_name {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(
                        f,
                        "{}",
                        serde_json::to_string(self)
                            .unwrap_or_else(|_|
                                format!(
                                    "{{\"error\": \"Could not convert `{}` to string!\"}}",
                                    stringify!($tx_output_struct_name),
                                ).into()
                            )
                    )
                }
            }

            #[cfg(test)]
            impl std::str::FromStr for $tx_output_struct_name {
                type Err = $crate::errors::AppError;

                fn from_str(s: &str) -> Result<Self> {
                    #[derive(serde::Serialize, serde::Deserialize)]
                    struct Interim {
                        $latest_block_number_field_name: usize,
                        $txs_field_name: Vec<serde_json::Value>,
                    }
                    let interim = serde_json::from_str::<Interim>(s)?;
                    let tx_infos = interim
                        .$txs_field_name
                        .iter()
                        .map(|json| $tx_info_struct_name::from_str(&json.to_string()))
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Self {
                        $latest_block_number_field_name: interim.$latest_block_number_field_name,
                        $txs_field_name: tx_infos,
                    })
                }
            }

            #[cfg(test)]
            impl std::str::FromStr for $tx_info_struct_name {
                type Err = $crate::errors::AppError;

                fn from_str(s: &str) -> $crate::types::Result<Self> {
                    Ok(serde_json::from_str(s)?)
                }
            }

        }
    };
}
