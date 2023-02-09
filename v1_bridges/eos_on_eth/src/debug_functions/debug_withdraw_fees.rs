use common::{
    chains::{
        eos::{
            eos_actions::PTokenPegOutAction,
            eos_constants::{EOS_ACCOUNT_PERMISSION_LEVEL, PEGOUT_ACTION_NAME},
            eos_crypto::{eos_private_key::EosPrivateKey, eos_transaction::EosSignedTransaction},
            eos_database_utils::EosDbUtils,
            eos_utils::get_eos_tx_expiration_timestamp_with_offset,
        },
        eth::eth_utils::convert_hex_to_eth_address,
    },
    core_type::CoreType,
    dictionaries::eos_eth::EosEthTokenDictionary,
    traits::DatabaseInterface,
    types::Result,
};
use common_debug_signers::validate_debug_command_signature;
use eos_chain::{Action as EosAction, PermissionLevel, Transaction as EosTransaction};
use function_name::named;
use serde_json::json;

use crate::constants::CORE_TYPE;

/// # Debug Withwdraw Fees
///
/// This function takes an ETH address and uses it to search through the token dictionary to find a
/// corresponding entry. Once found, that entry's accrued fees are zeroed, a timestamp set in that
/// entry to mark the withdrawal date and the dictionary saved back in the database. Finally, an
/// EOS transaction is created to transfer the `<accrued_fees>` amount of tokens to the passed in
/// recipient address.
#[named]
pub fn debug_withdraw_fees<D: DatabaseInterface>(
    db: &D,
    token_address: &str,
    recipient_address: &str,
    ref_block_num: u16,
    ref_block_prefix: u32,
    signature: &str,
) -> Result<String> {
    db.start_transaction()?;
    CoreType::check_is_initialized(db)?;

    let dictionary = EosEthTokenDictionary::get_from_db(db)?;
    let dictionary_entry_eth_address = convert_hex_to_eth_address(token_address)?;
    let eos_smart_contract_address = EosDbUtils::new(db).get_eos_account_name_from_db()?.to_string();

    get_debug_command_hash!(
        function_name!(),
        token_address,
        recipient_address,
        &ref_block_num,
        &ref_block_prefix
    )()
    .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
    .and_then(|_| dictionary.withdraw_fees_and_save_in_db(db, &dictionary_entry_eth_address))
    .and_then(|(_, fee_amount)| {
        let amount = dictionary.convert_u256_to_eos_asset_string(&dictionary_entry_eth_address, &fee_amount)?;
        info!("Amount as EOS asset: {}", amount);
        let eos_action = EosAction::from_str(
            &eos_smart_contract_address,
            &PEGOUT_ACTION_NAME.into(),
            vec![PermissionLevel::from_str(
                &eos_smart_contract_address,
                &EOS_ACCOUNT_PERMISSION_LEVEL.into(),
            )?],
            PTokenPegOutAction::from_str(
                &dictionary
                    .get_entry_via_eth_address(&dictionary_entry_eth_address)?
                    .eos_address,
                &amount,
                recipient_address,
                &[],
            )?,
        )?;
        EosSignedTransaction::from_unsigned_tx(
            &eos_smart_contract_address,
            &amount,
            &EosDbUtils::new(db).get_eos_chain_id_from_db()?,
            &EosPrivateKey::get_from_db(db)?,
            &EosTransaction::new(
                get_eos_tx_expiration_timestamp_with_offset(0u32)?,
                ref_block_num,
                ref_block_prefix,
                vec![eos_action],
            ),
        )
    })
    .and_then(|eos_signed_tx| {
        db.end_transaction()?;
        Ok(json!({
            "success": true,
            "eos_tx_signature": eos_signed_tx.signature,
            "eos_serialized_tx": eos_signed_tx.transaction,
        })
        .to_string())
    })
}
