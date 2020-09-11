use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eth::{
        eth_constants::PTOKEN_CONTRACT_TOPICS,
        eth_block_and_receipts::EthBlockAndReceipts,
        eth_receipt::{
            EthReceipt,
            EthReceipts,
        },
        eth_types::{
            EthHash,
            EthTopic,
            EthAddress,
        },
    },
    btc_on_eth::eth::{
        eth_state::EthState,
        eth_database_utils::get_erc777_contract_address_from_db,
    },
};

fn filter_receipts_for_address_and_topic(
    receipts: &EthReceipts,
    address: &EthAddress,
    topic: &EthHash
) -> Vec<EthReceipt> {
    receipts
        .0
        .iter()
        .filter(|receipt| receipt.logs.contain_address(address))
        .filter(|receipt| receipt.logs.contain_topic(topic))
        .cloned()
        .collect::<Vec<EthReceipt>>()
}

fn filter_receipts_for_address_and_topics(
    receipts: &EthReceipts,
    address: &EthAddress,
    topics: &[EthTopic],
) -> Vec<EthReceipt> {
    topics
        .iter()
        .map(|topic| filter_receipts_for_address_and_topic(receipts, &address, &topic))
        .flatten()
        .collect::<Vec<EthReceipt>>()
}

fn filter_eth_block_and_receipts(
    eth_block_and_receipts: &EthBlockAndReceipts,
    address: &EthAddress,
    topics: &[EthTopic],
) -> Result<EthBlockAndReceipts> {
    Ok(
        EthBlockAndReceipts {
            block: eth_block_and_receipts.block.clone(),
            receipts: EthReceipts::new(
                filter_receipts_for_address_and_topics(&eth_block_and_receipts.receipts, address, topics)
            )
        }
    )
}

pub fn filter_irrelevant_receipts_from_state<D>(state: EthState<D>) -> Result<EthState<D>> where D: DatabaseInterface {
    info!("✔ Filtering out non-pToken related receipts...");
    filter_eth_block_and_receipts(
        state.get_eth_block_and_receipts()?,
        &get_erc777_contract_address_from_db(&state.db)?,
        &PTOKEN_CONTRACT_TOPICS.to_vec(),
    )
        .and_then(|filtered_block_and_receipts| {
            info!("✔ Receipts filtered, amount remaining: {}", filtered_block_and_receipts.receipts.len());
            state.update_eth_block_and_receipts(filtered_block_and_receipts)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::eth_constants::REDEEM_EVENT_TOPIC_HEX,
        btc_on_eth::eth::eth_test_utils::{
            get_sample_contract_topic,
            get_sample_contract_topics,
            get_sample_contract_address,
            get_sample_eth_block_and_receipts,
            get_sample_eth_block_and_receipts_n,
        },
    };

    #[test]
    fn should_filter_receipts_for_topic() {
        let receipts = get_sample_eth_block_and_receipts().receipts;
        let num_receipts_before = receipts.len();
        let topic = get_sample_contract_topic();
        let address = get_sample_contract_address();
        let result = filter_receipts_for_address_and_topic(&receipts, &address, &topic);
        let num_receipts_after = result.len();
        assert!(num_receipts_before > num_receipts_after);
        result.iter().map(|receipt| assert!(receipt.logs.contain_topic(&topic))).for_each(drop);
    }

    #[test]
    fn should_filter_eth_block_and_receipts() {
        let block_and_receipts = get_sample_eth_block_and_receipts();
        let num_receipts_before = block_and_receipts.receipts.len();
        let address = get_sample_contract_address();
        let topics = get_sample_contract_topics();
        let result = filter_eth_block_and_receipts(&block_and_receipts, &address, &topics).unwrap();
        let num_receipts_after = result.receipts.len();
        assert!(num_receipts_before > num_receipts_after);
        result
            .receipts
            .0
            .iter()
            .map(|receipt| {
                assert!(receipt.logs.contain_topic(&topics[0]));
                receipt
            })
            .map(|receipt| assert!(receipt.logs.contain_address(&address)))
            .for_each(drop);
    }

    #[test]
    fn should_filter_eth_block_and_receipts_2() {
        let expected_num_receipts_after = 1;
        let block_and_receipts = get_sample_eth_block_and_receipts_n(6).unwrap();
        let num_receipts_before = block_and_receipts.receipts.len();
        let address = EthAddress::from_slice(&hex::decode("74630cfbc4066726107a4efe73956e219bbb46ab").unwrap());
        let topics = vec![EthHash::from_slice(&hex::decode(REDEEM_EVENT_TOPIC_HEX).unwrap()) ];
        let result = filter_eth_block_and_receipts(&block_and_receipts, &address, &topics).unwrap();
        let num_receipts_after = result.receipts.len();
        assert!(num_receipts_before > num_receipts_after);
        assert_eq!(num_receipts_after, expected_num_receipts_after);
        result
            .receipts
            .0
            .iter()
            .map(|receipt| {
                assert!(receipt.logs.contain_topic(&topics[0]));
                receipt
            })
            .map(|receipt| assert!(receipt.logs.contain_address(&address)))
            .for_each(drop);
    }
}
