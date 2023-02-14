use common::{metadata::metadata_traits::ToMetadata, traits::DatabaseInterface, types::Result, EthChainId};
use common_btc::BtcState;
use common_eth::{
    encode_erc777_mint_fxn_maybe_with_data,
    EthDbUtils,
    EthDbUtilsExt,
    EthPrivateKey,
    EthSigningParams,
    EthTransaction,
    EthTransactions,
    ZERO_ETH_VALUE,
};

use crate::btc::{BtcOnIntIntTxInfo, BtcOnIntIntTxInfos};

impl BtcOnIntIntTxInfo {
    pub fn to_int_signed_tx(
        &self,
        nonce: u64,
        chain_id: &EthChainId,
        gas_limit: usize,
        gas_price: u64,
        pk: &EthPrivateKey,
    ) -> Result<EthTransaction> {
        let operator_data = None;
        let metadata_bytes = self.to_metadata_bytes()?;
        info!("✔ Signing INT transaction for tx info: {:?}", self);
        debug!("✔ Signing with nonce:     {}", nonce);
        debug!("✔ Signing with chain id:  {}", chain_id);
        debug!("✔ Signing with gas limit: {}", gas_limit);
        debug!("✔ Signing with gas price: {}", gas_price);
        debug!("✔ Signing with metadata : 0x{}", hex::encode(&metadata_bytes));
        encode_erc777_mint_fxn_maybe_with_data(
            &self.router_address,
            &self.host_token_amount,
            Some(metadata_bytes),
            operator_data,
        )
        .map(|data| {
            EthTransaction::new_unsigned(
                data,
                nonce,
                ZERO_ETH_VALUE,
                self.int_token_address,
                chain_id,
                gas_limit,
                gas_price,
            )
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(pk))
    }
}

impl BtcOnIntIntTxInfos {
    pub fn to_int_signed_txs(&self, signing_params: &EthSigningParams) -> Result<EthTransactions> {
        trace!("✔ Getting INT signed transactions...");
        Ok(EthTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, tx_info)| {
                    info!(
                        "✔ Signing INT tx for host amount: {}, to destination address: {}",
                        tx_info.host_token_amount, tx_info.destination_address,
                    );
                    tx_info.to_int_signed_tx(
                        signing_params.eth_account_nonce + i as u64,
                        &signing_params.chain_id,
                        signing_params.chain_id.get_erc777_mint_with_data_gas_limit(),
                        signing_params.gas_price,
                        &signing_params.eth_private_key,
                    )
                })
                .collect::<Result<Vec<EthTransaction>>>()?,
        ))
    }
}

pub fn maybe_sign_canon_block_txs<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    let tx_infos = BtcOnIntIntTxInfos::from_bytes(
        &state
            .btc_db_utils
            .get_btc_canon_block_from_db()?
            .get_btc_on_int_int_tx_infos(),
    )?;
    if tx_infos.is_empty() {
        info!("✔ No transactions to sign in canon block ∴ not signing anything!");
        Ok(state)
    } else {
        info!("✔ Signing INT txs from BTC canon block...");
        tx_infos
            .to_int_signed_txs(&EthDbUtils::new(state.db).get_signing_params_from_db()?)
            .and_then(|signed_txs| {
                debug!("✔ Signed transactions: {:?}", signed_txs);
                signed_txs.to_bytes()
            })
            .map(|bytes| state.add_eth_signed_txs(bytes))
    }
}
