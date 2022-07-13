macro_rules! impl_to_erc20_token_event {
    ($path:path, $value:ident, $to:ident, $from:ident, $token_address:ident) => {
        use $crate::chains::eth::eth_contracts::erc20_token::{Erc20TokenTransferEvent, ToErc20TokenTransferEvent};

        impl ToErc20TokenTransferEvent for $path {
            fn to_erc20_token_transfer_event(&self) -> Erc20TokenTransferEvent {
                Erc20TokenTransferEvent::new(self.$value, self.$to, self.$from, self.$token_address)
            }
        }
    };
}
