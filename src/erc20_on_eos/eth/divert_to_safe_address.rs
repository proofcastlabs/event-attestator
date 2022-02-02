use crate::{
    chains::eth::eth_state::EthState,
    constants::SAFE_EOS_ADDRESS,
    erc20_on_eos::eth::peg_in_info::{Erc20OnEosPegInInfo, Erc20OnEosPegInInfos},
    traits::DatabaseInterface,
    types::Result,
};

impl Erc20OnEosPegInInfo {
    fn change_destination_to_safe_address(&self) -> Self {
        let mut mutable_self = self.clone();
        mutable_self.destination_address = SAFE_EOS_ADDRESS.to_string();
        mutable_self
    }

    fn divert_to_safe_address_if_destination_address_matches_address(&self, address: &str) -> Self {
        if self.destination_address == address {
            self.change_destination_to_safe_address()
        } else {
            self.clone()
        }
    }

    fn maybe_divert_to_safe_address_if_destination_address_matches_token_address(&self) -> Self {
        self.divert_to_safe_address_if_destination_address_matches_address(&self.eos_token_address)
    }
}

impl Erc20OnEosPegInInfos {
    fn divert_to_safe_address_if_destination_addresses_match_token_address(&self) -> Self {
        Self(
            self.iter()
                .map(|info| info.maybe_divert_to_safe_address_if_destination_address_matches_token_address())
                .collect::<Vec<Erc20OnEosPegInInfo>>(),
        )
    }
}

pub fn maybe_divert_txs_to_safe_address_if_destination_is_token_address<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe diverting tx infos to safe address if destinations match token address...");
    let tx_infos = state.erc20_on_eos_peg_in_infos.clone();
    if tx_infos.is_empty() {
        Ok(state)
    } else {
        let updated_infos = tx_infos.divert_to_safe_address_if_destination_addresses_match_token_address();
        state.replace_erc20_on_eos_peg_in_infos(updated_infos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_divert_to_safe_address_if_destination_address_it_token_address() {
        let mut info = Erc20OnEosPegInInfo::default();
        let address = "someaddress".to_string();
        info.eos_token_address = address.clone();
        info.destination_address = address.clone();
        assert_eq!(info.destination_address, info.eos_token_address);
        let result = info.maybe_divert_to_safe_address_if_destination_address_matches_token_address();
        assert_eq!(result.destination_address, *SAFE_EOS_ADDRESS);
    }
}
