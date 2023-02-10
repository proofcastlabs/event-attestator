use bitcoin::{
    blockdata::transaction::Transaction as BtcTransaction,
    network::constants::Network as BtcNetwork,
    util::address::Address as BtcAddress,
};
use common::{
    chains::btc::{
        btc_chain_id::BtcChainId,
        btc_utils::convert_satoshis_to_wei,
        deposit_address_info::DepositInfoHashMap,
    },
    state::BtcState,
    traits::DatabaseInterface,
    types::{NoneError, Result},
};
use common_eth::{EthDbUtils, EthDbUtilsExt};
use ethereum_types::Address as EthAddress;

use crate::btc::{BtcOnIntIntTxInfo, BtcOnIntIntTxInfos};

impl BtcOnIntIntTxInfos {
    fn from_btc_tx(
        tx: &BtcTransaction,
        deposit_info: &DepositInfoHashMap,
        network: BtcNetwork,
        int_token_address: &EthAddress,
        router_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Parsing INT tx infos from single `P2SH` transaction...");
        Ok(Self::new(
            tx.output
                .iter()
                .filter(|tx_out| tx_out.script_pubkey.is_p2sh())
                .map(|tx_out| match BtcAddress::from_script(&tx_out.script_pubkey, network) {
                    Err(_) => {
                        info!("✘ Could not derive BTC address from tx: {:?}", tx);
                        (tx_out, None)
                    },
                    Ok(address) => {
                        info!("✔ BTC address extracted from `tx_out`: {}", address);
                        (tx_out, Some(address))
                    },
                })
                .filter(|(_, maybe_address)| maybe_address.is_some())
                .map(|(tx_out, address)| {
                    match deposit_info.get(&address.clone().ok_or(NoneError("Could not unwrap BTC address!"))?) {
                        None => {
                            info!(
                                "✘ BTC address {} not in deposit list!",
                                address.ok_or(NoneError("Could not unwrap BTC address!"))?
                            );
                            Err("Filtering out this err!".into())
                        },
                        Some(deposit_info) => {
                            info!("✔ Deposit info from list: {:?}", deposit_info);
                            Ok(BtcOnIntIntTxInfo {
                                originating_tx_hash: tx.txid(),
                                router_address: *router_address,
                                vault_address: EthAddress::zero(), // NOTE: There's no vault on this common!
                                native_token_amount: tx_out.value,
                                int_token_address: *int_token_address,
                                user_data: deposit_info.user_data.clone(),
                                destination_address: deposit_info.address.clone(),
                                destination_chain_id: deposit_info.chain_id.clone(),
                                host_token_amount: convert_satoshis_to_wei(tx_out.value),
                                origin_chain_id: BtcChainId::from_btc_network(&network)?
                                    .to_metadata_chain_id()
                                    .to_bytes()?,
                                originating_tx_address: address
                                    .ok_or(NoneError("Could not unwrap BTC address!"))?
                                    .to_string(),
                            })
                        },
                    }
                })
                .filter(|maybe_int_tx_infos| maybe_int_tx_infos.is_ok())
                .collect::<Result<Vec<BtcOnIntIntTxInfo>>>()?,
        ))
    }

    pub fn from_btc_txs(
        txs: &[BtcTransaction],
        deposit_info: &DepositInfoHashMap,
        network: BtcNetwork,
        int_token_address: &EthAddress,
        router_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Parsing INT tx infos from `P2SH` transactions...");
        Ok(Self::new(
            txs.iter()
                .flat_map(|tx| Self::from_btc_tx(tx, deposit_info, network, int_token_address, router_address))
                .flat_map(|int_tx_infos| int_tx_infos.0)
                .collect::<Vec<BtcOnIntIntTxInfo>>(),
        ))
    }
}

pub fn parse_int_tx_infos_from_p2sh_deposits_and_add_to_state<D: DatabaseInterface>(
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    info!("✔ Parsing INT tx infos from `P2SH` deposit txs in state...");
    let eth_db_utils = EthDbUtils::new(state.db);
    BtcOnIntIntTxInfos::from_btc_txs(
        state.get_p2sh_deposit_txs()?,
        state.get_deposit_info_hash_map()?,
        state.btc_db_utils.get_btc_network_from_db()?,
        &eth_db_utils.get_btc_on_int_smart_contract_address_from_db()?,
        &eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
    )
    .and_then(|params| params.to_bytes())
    .map(|bytes| state.add_tx_infos(bytes))
}
