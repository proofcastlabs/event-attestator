use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eth::eth::eth_state::EthState,
    chains::eth::{
        eth_log::EthLog,
        eth_receipt::EthReceipt,
        eth_types::RedeemParams,
        eth_block_and_receipts::EthBlockAndReceipts,
        eth_database_utils::get_eth_canon_block_from_db,
    },
};

fn parse_redeem_params_from_log_and_receipt(log: &EthLog, receipt: &EthReceipt) -> Result<RedeemParams> {
    info!("✔ Parsing redeems from logs...");
    Ok(RedeemParams::new(log.get_redeem_amount()?, receipt.from, log.get_btc_address()?, receipt.transaction_hash))
}

fn parse_amount_and_address_tuples_from_receipt(receipt: &EthReceipt) -> Result<Vec<RedeemParams>> {
    info!("✔ Parsing amount & address tuples from receipt...");
    receipt
        .logs
        .0
        .iter()
        .filter(|log| matches!(log.is_ptoken_redeem(), Ok(true)))
        .map(|log| parse_redeem_params_from_log_and_receipt(log, receipt))
        .collect::<Result<Vec<RedeemParams>>>()
}

pub fn parse_redeem_params_from_block(eth_block_and_receipts: EthBlockAndReceipts) -> Result<Vec<RedeemParams>> {
    info!("✔ Parsing redeem params from block...");
    let mut redeem_params_vec = Vec::new();
    for receipt in eth_block_and_receipts.get_receipts() {
        let structures = parse_amount_and_address_tuples_from_receipt(&receipt)?;
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
    use ethereum_types::{
        U256,
        H256 as EthHash,
        Address as EthAddress,
    };
    use crate::btc_on_eth::eth::eth_test_utils::{
        get_sample_log_n,
        get_sample_eth_block_and_receipts_n,
    };

    fn get_sample_log_with_p2sh_redeem() -> EthLog {
        get_sample_log_n(5, 23, 2)
            .unwrap()
    }

    fn get_sample_block_with_redeem() -> EthBlockAndReceipts {
        get_sample_eth_block_and_receipts_n(4)
            .unwrap()
    }

    fn get_tx_hash_of_redeem_tx() -> &'static str {
        "442612aba789ce873bb3804ff62ced770dcecb07d19ddcf9b651c357eebaed40"
    }

    fn get_expected_redeem_params() -> RedeemParams {
        let amount = U256::from_dec_str("666").unwrap();
        let from = EthAddress::from_str(
            "edb86cd455ef3ca43f0e227e00469c3bdfa40628"
        ).unwrap();
        let recipient = "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM"
            .to_string();
        let originating_tx_hash = EthHash::from_slice(&hex::decode(
            get_tx_hash_of_redeem_tx()
        ).unwrap()[..]);
        RedeemParams::new(amount, from, recipient, originating_tx_hash)
    }

    fn get_sample_receipt_with_redeem() -> EthReceipt {
        let hash = EthHash::from_str(get_tx_hash_of_redeem_tx())
            .unwrap();
        get_sample_block_with_redeem()
            .receipts
            .0
            .iter()
            .filter(|receipt| receipt.transaction_hash == hash)
            .collect::<Vec<&EthReceipt>>()
            [0]
            .clone()
    }

    fn get_sample_log_with_redeem() -> EthLog {
        get_sample_receipt_with_redeem().logs.0[2].clone()
    }

    #[test]
    fn should_parse_redeem_params_from_log_and_receipt() {
        let result = parse_redeem_params_from_log_and_receipt(
            &get_sample_log_with_redeem(),
            &get_sample_receipt_with_redeem(),
        ).unwrap();
        assert_eq!(result, get_expected_redeem_params());
    }

    #[test]
    fn should_parse_amount_and_address_tuples_from_receipt() {
        let expected_num_results = 1;
        let result = parse_amount_and_address_tuples_from_receipt(
            &get_sample_receipt_with_redeem()
        ).unwrap();
        assert_eq!(result.len(), expected_num_results);
        assert_eq!(result[0], get_expected_redeem_params());
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
