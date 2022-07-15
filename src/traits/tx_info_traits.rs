use crate::{address::Address, chains::eth::eth_constants::ETH_ZERO_ADDRESS_STR};

pub trait TxInfo {
    fn get_vault_address(&self) -> Address;
    fn get_token_address(&self) -> Address;
    fn get_router_address(&self) -> Address;
    fn get_destination_address(&self) -> Address;
    fn set_destination_to_safe_address(self) -> Self
    where
        Self: Sized;

    fn divert_to_safe_address_if_destination_is_vault_address(self) -> Self
    where
        Self: Sized,
    {
        if self.get_destination_address() == self.get_vault_address() {
            warn!("✘ Diverting to safe address because the destination address is the vault address!");
            self.set_destination_to_safe_address()
        } else {
            self
        }
    }

    fn divert_to_safe_address_if_destination_is_token_address(self) -> Self
    where
        Self: Sized,
    {
        if self.get_destination_address() == self.get_token_address() {
            warn!("✘ Diverting to safe address because the destination address is the token address!");
            self.set_destination_to_safe_address()
        } else {
            self
        }
    }

    fn divert_to_safe_address_if_destination_is_router_address(self) -> Self
    where
        Self: Sized,
    {
        if self.get_destination_address() == self.get_router_address() {
            warn!("✘ Diverting to safe address because the destination address is the router address!");
            self.set_destination_to_safe_address()
        } else {
            self
        }
    }

    fn divert_to_safe_address_if_destination_is_zero_address(self) -> Self
    where
        Self: Sized,
    {
        if self.get_destination_address() == Address::Eth(ETH_ZERO_ADDRESS_STR.to_string()) {
            warn!("✘ Diverting to safe address because the destination address is the zero address!");
            self.set_destination_to_safe_address()
        } else {
            self
        }
    }
}
