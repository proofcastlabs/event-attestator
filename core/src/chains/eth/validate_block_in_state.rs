use crate::{
    chains::eth::{eth_chain_id::EthChainId, eth_database_utils::EthDbUtilsExt, eth_state::EthState},
    constants::CORE_IS_VALIDATING,
    traits::DatabaseInterface,
    types::Result,
};

fn validate_block_in_state<D: DatabaseInterface>(state: EthState<D>, is_for_eth: bool) -> Result<EthState<D>> {
    let symbol = if is_for_eth { "ETH" } else { "EVM" };
    if !CORE_IS_VALIDATING {
        info!("✔ Skipping {} block header validaton!", symbol);
        Ok(state)
    } else {
        let chain_id = if is_for_eth {
            state.eth_db_utils.get_eth_chain_id_from_db()?
        } else {
            state.evm_db_utils.get_eth_chain_id_from_db()?
        };
        info!("✔ Validating block in {} state using chain ID: {}", symbol, chain_id);
        if chain_id == EthChainId::Rinkeby {
            // NOTE: We cannot validate Rinkeby blocks. However it's been deprecated now so
            // this no longer matters.
            info!("✔ Skipping RINKEBY {} block header validaton!", symbol);
            Ok(state)
        } else {
            info!("✔ Validating {} block header...", symbol);
            state
                .get_eth_submission_material()?
                .get_block()?
                .is_valid(&chain_id)
                .and_then(|is_valid| {
                    if is_valid {
                        Ok(state)
                    } else {
                        Err(format!("✘ Not accepting {} block - header hash not valid!", symbol).into())
                    }
                })
        }
    }
}

pub fn validate_evm_block_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Validating EVM block in state...");
    validate_block_in_state(state, false)
}

pub fn validate_eth_block_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Validating ETH block in state...");
    validate_block_in_state(state, true)
}
