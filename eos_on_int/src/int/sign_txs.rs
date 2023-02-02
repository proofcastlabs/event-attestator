use common::{
    chains::eos::{
        eos_actions::PTokenPegOutAction,
        eos_chain_id::EosChainId,
        eos_constants::{EOS_ACCOUNT_PERMISSION_LEVEL, PEGOUT_ACTION_NAME},
        eos_crypto::{
            eos_private_key::EosPrivateKey,
            eos_transaction::{EosSignedTransaction, EosSignedTransactions},
        },
        eos_utils::get_eos_tx_expiration_timestamp_with_offset,
    },
    metadata::metadata_traits::ToMetadata,
    state::EthState,
    traits::DatabaseInterface,
    types::{Byte, Result},
};
use eos_chain::{AccountName as EosAccountName, Action as EosAction, PermissionLevel, Transaction as EosTransaction};

use crate::int::eos_tx_info::{EosOnIntEosTxInfo, EosOnIntEosTxInfos};

const ZERO_ETH_ASSET_STR: &str = "0.0000 EOS";

impl EosOnIntEosTxInfos {
    pub fn to_eos_signed_txs(
        &self,
        ref_block_num: u16,
        ref_block_prefix: u32,
        chain_id: &EosChainId,
        pk: &EosPrivateKey,
        eos_smart_contract: &EosAccountName,
    ) -> Result<EosSignedTransactions> {
        info!("✔ Signing {} EOS txs from `EosOnIntEosTxInfos`...", self.len());
        Ok(EosSignedTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, tx_info)| {
                    info!("✔ Signing EOS tx from `EosOnIntEosTxInfo`: {:?}", tx_info);
                    tx_info.to_eos_signed_tx(
                        ref_block_num,
                        ref_block_prefix,
                        eos_smart_contract,
                        ZERO_ETH_ASSET_STR,
                        chain_id,
                        pk,
                        get_eos_tx_expiration_timestamp_with_offset(i as u32)?,
                    )
                })
                .collect::<Result<Vec<EosSignedTransaction>>>()?,
        ))
    }
}

impl EosOnIntEosTxInfo {
    fn get_eos_ptoken_peg_out_action(
        from: &str,
        actor: &str,
        permission_level: &str,
        token_contract: &str,
        quantity: &str,
        recipient: &str,
        metadata: &[Byte],
    ) -> Result<EosAction> {
        debug!(
            "from: {}\nactor: {}\npermission_level: {}\ntoken_contract: {}\nquantity: {}\nrecipient: {}\nmetadata: '0x{}'",
            from, actor, permission_level, token_contract, quantity, recipient, hex::encode(metadata),
        );
        Ok(EosAction::from_str(
            from,
            PEGOUT_ACTION_NAME,
            vec![PermissionLevel::from_str(actor, permission_level)?],
            PTokenPegOutAction::from_str(token_contract, quantity, recipient, metadata)?,
        )?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn to_eos_signed_tx(
        &self,
        ref_block_num: u16,
        ref_block_prefix: u32,
        eos_smart_contract: &EosAccountName,
        amount: &str,
        chain_id: &EosChainId,
        pk: &EosPrivateKey,
        timestamp: u32,
    ) -> Result<EosSignedTransaction> {
        info!("✔ Signing eos tx...");
        let metadata = if self.user_data.is_empty() {
            Ok(vec![])
        } else {
            info!("✔ Wrapping `user_data` in metadata for `EosOnIntEosTxInfo`...");
            self.to_metadata_bytes()
        }?;
        debug!(
            "smart-contract: {}\namount: {}\nchain ID: {}\nmetadata: {}",
            &eos_smart_contract,
            &amount,
            &chain_id.to_hex(),
            hex::encode(&metadata),
        );
        Self::get_eos_ptoken_peg_out_action(
            &eos_smart_contract.to_string(),
            &eos_smart_contract.to_string(),
            EOS_ACCOUNT_PERMISSION_LEVEL,
            &self.eos_token_address,
            &self.eos_asset_amount,
            &self.destination_address,
            &metadata,
        )
        .map(|action| EosTransaction::new(timestamp, ref_block_num, ref_block_prefix, vec![action]))
        .and_then(|ref unsigned_tx| {
            EosSignedTransaction::from_unsigned_tx(&eos_smart_contract.to_string(), amount, chain_id, pk, unsigned_tx)
        })
    }
}

pub fn maybe_sign_eos_txs_and_add_to_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Maybe signing `EosOnIntEosTxInfos`...");
    let submission_material = state.get_eth_submission_material()?;
    EosOnIntEosTxInfos::from_bytes(&state.tx_infos)
        .and_then(|tx_infos| {
            tx_infos.to_eos_signed_txs(
                submission_material.get_eos_ref_block_num()?,
                submission_material.get_eos_ref_block_prefix()?,
                &state.eos_db_utils.get_eos_chain_id_from_db()?,
                &EosPrivateKey::get_from_db(state.db)?,
                &state.eos_db_utils.get_eos_account_name_from_db()?,
            )
        })
        .and_then(|signed_txs| state.add_eos_transactions(signed_txs))
}
