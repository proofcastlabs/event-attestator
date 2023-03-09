use common::{constants::CORE_IS_VALIDATING, traits::DatabaseInterface, types::Result};
use common_chain_ids::EthChainId;

use crate::{EthDbUtilsExt, EthState, EthStateCompatible};

fn validate_block_in_state<D: DatabaseInterface>(state: &impl EthStateCompatible<D>, is_for_eth: bool) -> Result<()> {
    let symbol = if is_for_eth { "ETH" } else { "EVM" };
    if !CORE_IS_VALIDATING {
        info!("✔ Skipping {} block header validaton!", symbol);
        Ok(())
    } else {
        let chain_id = if is_for_eth {
            state.get_eth_db_utils().get_eth_chain_id_from_db()?
        } else {
            state.get_evm_db_utils().get_eth_chain_id_from_db()?
        };
        info!("✔ Validating block in {} state using chain ID: {}", symbol, chain_id);
        if chain_id == EthChainId::Rinkeby {
            // NOTE: We cannot validate Rinkeby blocks. However it's been deprecated now so
            // this no longer matters.
            info!("✔ Skipping RINKEBY {} block header validaton!", symbol);
            Ok(())
        } else {
            info!("✔ Validating {} block header...", symbol);
            state
                .get_sub_mat()?
                .get_block()?
                .is_valid(&chain_id)
                .and_then(|is_valid| {
                    if is_valid {
                        Ok(())
                    } else {
                        Err(format!("✘ Not accepting {} block - header hash not valid!", symbol).into())
                    }
                })
        }
    }
}

pub fn validate_evm_block_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Validating EVM block in state...");
    validate_block_in_state(&state, false).and(Ok(state))
}

pub fn validate_eth_block_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Validating ETH block in state...");
    validate_block_in_state(&state, true).and(Ok(state))
}
