#[macro_export]
macro_rules! impl_tx_info_trait {
    (
        $struct:ty,
        $vault_address_field:ident,
        $router_address_field:ident,
        $token_address_field:ident,
        $destination_address_field:ident,
        $destination_address_type:expr,
        $safe_address:expr
    ) => {
        impl $crate::traits::TxInfo for $struct {
            fn get_vault_address(&self) -> Address {
                Address::Eth(Self::convert_eth_address_to_string(&self.$vault_address_field))
            }

            fn get_router_address(&self) -> Address {
                Address::Eth(Self::convert_eth_address_to_string(&self.$router_address_field))
            }

            fn get_token_address(&self) -> Address {
                Address::Eth(Self::convert_eth_address_to_string(&self.$token_address_field))
            }

            fn get_destination_address(&self) -> Address {
                $destination_address_type(self.$destination_address_field.clone())
            }

            fn set_destination_to_safe_address(self) -> Self {
                let mut mutable_self = self.clone();
                mutable_self.destination_address = $safe_address.to_string();
                mutable_self
            }
        }
    };
}
