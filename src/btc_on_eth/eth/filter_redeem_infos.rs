use ethereum_types::U256;
use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eth::eth::eth_state::EthState,
    chains::{
        btc::btc_constants::MINIMUM_REQUIRED_SATOSHIS,
        eth::eth_redeem_info::{
            RedeemInfo,
            RedeemInfos,
        },
    },
};

fn filter_redeem_infos(redeem_infos: &RedeemInfos) -> Result<RedeemInfos> {
    Ok(RedeemInfos::new(
        &redeem_infos
            .0
            .iter()
            .filter(|infos| {
                match infos.amount >= U256::from(MINIMUM_REQUIRED_SATOSHIS) {
                    true => true,
                    false => {
                        trace!("✘ Filtering redeem infos ∵ amount too low: {:?}", infos);
                        false
                    }
                }
            })
            .cloned()
            .collect::<Vec<RedeemInfo>>()
    ))
}

pub fn maybe_filter_redeem_infos_in_state<D>(state: EthState<D>) -> Result<EthState<D>> where D: DatabaseInterface {
    info!("✔ Maybe filtering any redeem infos below minimum # of Satoshis...");
    filter_redeem_infos(&state.btc_on_eth_redeem_infos)
        .and_then(|new_infos| state.replace_btc_on_eth_redeem_infos(new_infos))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use ethereum_types::U256;
    use crate::chains::eth::{
        eth_redeem_info::RedeemInfo,
        eth_types::{
            EthHash,
            EthAddress,
        },
    };

    #[test]
    fn should_filter_redeem_infos() {
        let expected_length = 2;
        let infos = RedeemInfos::new(&vec![
            RedeemInfo {
                amount: U256::from_dec_str("4999").unwrap(),
                from: EthAddress::from_str(
                    "edb86cd455ef3ca43f0e227e00469c3bdfa40628"
                ).unwrap(),
                recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM".to_string(),
                originating_tx_hash: EthHash::from_slice(
                    &hex::decode("17f84a414c183bfafa4cd05e9ad13185e5eb6983085c222cae5afa4bba212da5")
                    .unwrap()[..]
                ),
            },
            RedeemInfo {
                amount: U256::from_dec_str("5000").unwrap(),
                from: EthAddress::from_str(
                    "edb86cd455ef3ca43f0e227e00469c3bdfa40628"
                ).unwrap(),
                recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM".to_string(),
                originating_tx_hash: EthHash::from_slice(
                    &hex::decode("17f84a414c183bfafa4cd05e9ad13185e5eb6983085c222cae5afa4bba212da5")
                    .unwrap()[..]
                ),
            },
            RedeemInfo {
                amount: U256::from_dec_str("5001").unwrap(),
                from: EthAddress::from_str(
                    "edb86cd455ef3ca43f0e227e00469c3bdfa40628"
                ).unwrap(),
                recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM".to_string(),
                originating_tx_hash: EthHash::from_slice(
                    &hex::decode("17f84a414c183bfafa4cd05e9ad13185e5eb6983085c222cae5afa4bba212da5")
                    .unwrap()[..]
                ),
            },
        ]);
        let length_before = infos.len();
        let result = filter_redeem_infos(&infos).unwrap();
        let length_after = result.len();
        assert!(length_before > length_after);
        assert_eq!(length_after, expected_length);
    }
}
