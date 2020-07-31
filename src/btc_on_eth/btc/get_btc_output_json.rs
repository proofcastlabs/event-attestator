use std::time::{
    SystemTime,
    UNIX_EPOCH
};
use crate::{
    types::{
        Byte,
        Result,
    },
    traits::DatabaseInterface,
    chains::{
        btc::btc_constants::DEFAULT_BTC_ADDRESS,
        eth::{
            any_sender::relay_transaction::RelayTransaction,
            eth_crypto::{
                eth_transaction::EthTransaction,
                eth_private_key::EthPrivateKey,
            },
        },
    },
    btc_on_eth::{
        eth::{
            eth_database_utils::{
                get_eth_private_key_from_db,
                get_any_sender_nonce_from_db,
                get_eth_account_nonce_from_db,
                get_public_eth_address_from_db,
                get_erc777_proxy_contract_address_from_db,
            },
        },
        btc::{
            btc_state::BtcState,
            btc_types::MintingParamStruct,
            btc_database_utils::{
                get_btc_canon_block_from_db,
                get_btc_latest_block_from_db,
            },
        },
    },
};
use ethereum_types::Address as EthAddress;

#[derive(Debug, Serialize, Deserialize)]
pub struct EthTxInfo {
    pub eth_tx_hex: Option<String>,
    pub eth_tx_hash: String,
    pub eth_tx_amount: String,
    pub eth_account_nonce: Option<u64>,
    pub eth_tx_recipient: String,
    pub signature_timestamp: u64,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub any_sender_tx: Option<RelayTransaction>,
    pub any_sender_nonce: Option<u64>,
}

impl EthTxInfo {
    pub fn new(
        eth_tx: &EthTransaction,
        minting_param_struct: &MintingParamStruct,
        eth_account_nonce: Option<u64>,
    ) -> Result<EthTxInfo> {
        let default_address = DEFAULT_BTC_ADDRESS.to_string();
        let retrieved_address = minting_param_struct
            .originating_tx_address
            .to_string();
        let address_string = match default_address == retrieved_address {
            false => retrieved_address,
            true => "✘ Could not retrieve sender address".to_string(),
        };

        Ok(
            EthTxInfo {
                eth_account_nonce,
                eth_tx_hash: format!("0x{}", eth_tx.get_tx_hash()),
                eth_tx_hex: Some(eth_tx.serialize_hex()),
                originating_address: address_string,
                eth_tx_amount: minting_param_struct.amount.to_string(),
                originating_tx_hash:
                    minting_param_struct.originating_tx_hash.to_string(),
                eth_tx_recipient: format!(
                    "0x{}",
                    hex::encode(minting_param_struct.eth_address.as_bytes())
                ),
                signature_timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)?
                    .as_secs(),
                any_sender_tx: None,
                any_sender_nonce: None,
            }
        )
    }

    pub fn new_with_any_sender(
        chain_id: Byte,
        minting_param_struct: &MintingParamStruct,
        any_sender_nonce: u64,
        from: EthAddress,
        eth_private_key: &EthPrivateKey,
        erc777_proxy_address: EthAddress,
    ) -> Result<EthTxInfo> {
        let default_address = DEFAULT_BTC_ADDRESS.to_string();
        let retrieved_address = minting_param_struct
            .originating_tx_address
            .to_string();
        let address_string = match default_address == retrieved_address {
            false => retrieved_address,
            true => "✘ Could not retrieve sender address".to_string(),
        };

        let any_sender_tx = RelayTransaction::new_mint_by_proxy_tx(
            chain_id,
            from,
            minting_param_struct.amount,
            any_sender_nonce,
            eth_private_key,
            erc777_proxy_address,
            minting_param_struct.eth_address,
        )?;

        Ok(EthTxInfo {
            eth_account_nonce: None,
            eth_tx_hash: format!("0x{}", any_sender_tx.get_tx_hash()),
            eth_tx_hex: None,
            originating_address: address_string,
            eth_tx_amount: minting_param_struct.amount.to_string(),
            originating_tx_hash: minting_param_struct.originating_tx_hash.to_string(),
            eth_tx_recipient: format!(
                "0x{}",
                hex::encode(minting_param_struct.eth_address.as_bytes())
            ),
            signature_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            any_sender_tx: Some(any_sender_tx),
            any_sender_nonce: Some(any_sender_nonce),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BtcOutput {
    pub btc_latest_block_number: u64,
    pub eth_signed_transactions: Vec<EthTxInfo>,
}

#[allow(clippy::too_many_arguments)]
pub fn get_eth_signed_tx_info_from_eth_txs(
    eth_txs: &[EthTransaction],
    minting_params: &[MintingParamStruct],
    eth_account_nonce: u64,
    use_any_sender_tx_type: bool,
    any_sender_nonce: u64,
    from: EthAddress,
    eth_private_key: &EthPrivateKey,
    erc777_proxy_address: EthAddress,
) -> Result<Vec<EthTxInfo>> {
    if use_any_sender_tx_type {
        info!("✔ Getting any.sender tx info from ETH txs...");
        let any_sender_start_nonce = any_sender_nonce - eth_txs.len() as u64;

        return eth_txs
            .iter()
            .enumerate()
            .map(|(i, tx)| {
                EthTxInfo::new_with_any_sender(
                    tx.chain_id,
                    &minting_params[i],
                    any_sender_start_nonce + i as u64,
                    from,
                    eth_private_key,
                    erc777_proxy_address,
                )
            })
            .collect::<Result<Vec<EthTxInfo>>>();
    }

    info!("✔ Getting ETH tx info from ETH txs...");
    let start_nonce = eth_account_nonce - eth_txs.len() as u64;
    eth_txs
        .iter()
        .enumerate()
        .map(|(i, tx)| EthTxInfo::new(tx, &minting_params[i], Some(start_nonce + i as u64)))
        .collect::<Result<Vec<EthTxInfo>>>()
}

pub fn create_btc_output_json_and_put_in_state<D>(
    state: BtcState<D>
) -> Result<BtcState<D>>
    where D: DatabaseInterface
{
    info!("✔ Getting BTC output json and putting in state...");
    Ok(serde_json::to_string(
        &BtcOutput {
            btc_latest_block_number: get_btc_latest_block_from_db(&state.db)?
                .height,
            eth_signed_transactions: match &state.eth_signed_txs {
                None => vec![],
                Some(txs) =>
                    get_eth_signed_tx_info_from_eth_txs(
                        txs,
                        &get_btc_canon_block_from_db(&state.db)?.minting_params,
                        get_eth_account_nonce_from_db(&state.db)?,
                        state.use_any_sender_tx_type(),
                        get_any_sender_nonce_from_db(&state.db)?,
                        get_public_eth_address_from_db(&state.db)?,
                        &get_eth_private_key_from_db(&state.db)?,
                        get_erc777_proxy_contract_address_from_db(&state.db)?,
                    )?,
            }
        }
    )?)
        .and_then(|output| state.add_output_json_string(output))
}

pub fn get_btc_output_as_string<D>(
    state: BtcState<D>
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Getting BTC output as string...");
    let output = state.get_output_json_string()?.to_string();
    info!("✔ BTC Output: {}", output);
    Ok(output)
}
