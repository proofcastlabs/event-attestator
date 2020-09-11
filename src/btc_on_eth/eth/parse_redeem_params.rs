use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eth::eth::eth_state::EthState,
    chains::eth::{
        eth_types::RedeemParams,
        eth_block_and_receipts::EthBlockAndReceipts,
        eth_database_utils::get_eth_canon_block_from_db,
    },
};

pub fn parse_redeem_params_from_block(eth_block_and_receipts: EthBlockAndReceipts) -> Result<Vec<RedeemParams>> {
    info!("✔ Parsing redeem params from block...");
    let mut redeem_params_vec = Vec::new();
    for receipt in eth_block_and_receipts.get_receipts() {
        let structures = receipt.get_redeem_params()?;
        for structure in structures {
            redeem_params_vec.push(structure);
        }
    };
    Ok(redeem_params_vec)
}

pub fn maybe_parse_redeem_params_and_add_to_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe parsing redeem params...");
    get_eth_canon_block_from_db(&state.db)
        .and_then(|block_and_receipts| {
            match block_and_receipts.receipts.is_empty() {
                true => {
                    info!("✔ No receipts in canon block ∴ no params to parse!");
                    Ok(state)
                }
                false => {
                    info!("✔ Receipts in canon block #{} ∴ parsing params...", block_and_receipts.block.number);
                    parse_redeem_params_from_block(block_and_receipts)
                        .and_then(|redeem_params| state.add_redeem_params(redeem_params))
                }
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::btc_on_eth::eth::eth_test_utils::get_sample_eth_block_and_receipts_n;
    use ethereum_types::{
        U256,
        H256 as EthHash,
        Address as EthAddress,
    };

    fn get_sample_block_with_redeem() -> EthBlockAndReceipts {
        get_sample_eth_block_and_receipts_n(4)
            .unwrap()
    }

    fn get_tx_hash_of_redeem_tx() -> &'static str {
        "442612aba789ce873bb3804ff62ced770dcecb07d19ddcf9b651c357eebaed40"
    }

    #[test]
    fn should_parse_redeem_params_from_block() {
        let result = parse_redeem_params_from_block(
            get_sample_block_with_redeem()
        ).unwrap();
        let expected_result = RedeemParams {
            amount: U256::from_dec_str("666").unwrap(),
            from: EthAddress::from_str(
                "edb86cd455ef3ca43f0e227e00469c3bdfa40628"
            ).unwrap(),
            recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM".to_string(),
            originating_tx_hash: EthHash::from_slice(
                &hex::decode(get_tx_hash_of_redeem_tx())
                .unwrap()[..]
            ),
        };
        assert_eq!(expected_result.from, result[0].from);
        assert_eq!(expected_result.amount, result[0].amount);
        assert_eq!(expected_result.recipient, result[0].recipient);
        assert_eq!(expected_result.originating_tx_hash, result[0].originating_tx_hash);
    }
}
