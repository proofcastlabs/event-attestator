#[macro_use]
extern crate log;
#[macro_use]
extern crate paste;
#[macro_use]
extern crate common_docopt_macros;

mod get_cli_args;
mod usage_info;

use anyhow::Result;
use common::AppError;
#[cfg(feature = "file-logger")]
use common_file_logger::init_logger;
#[cfg(feature = "json-rpc")]
use common_jsonrpc_db::get_db;
#[cfg(feature = "rocks-db")]
use common_rocksdb_database::get_db;
#[cfg(feature = "stderr-logger")]
use common_stderr_logger::init_logger;
use erc20_on_eos::{
    debug_add_debug_signer,
    debug_add_multiple_debug_signers,
    debug_add_new_eos_schedule,
    debug_add_token_dictionary_entry,
    debug_disable_eos_protocol_feature,
    debug_enable_eos_protocol_feature,
    debug_get_add_supported_token_tx,
    debug_get_all_db_keys,
    debug_get_erc20_vault_migrate_single_tx,
    debug_get_key_from_db,
    debug_get_perc20_migration_tx,
    debug_get_remove_supported_token_tx,
    debug_remove_debug_signer,
    debug_remove_token_dictionary_entry,
    debug_reprocess_eos_block,
    debug_reprocess_eos_block_with_fee_accrual,
    debug_reprocess_eos_block_with_nonce,
    debug_reprocess_eth_block,
    debug_reprocess_eth_block_with_fee_accrual,
    debug_reset_eth_chain,
    debug_set_accrued_fees_in_dictionary,
    debug_set_eos_account_nonce,
    debug_set_eos_fee_basis_points,
    debug_set_eth_account_nonce,
    debug_set_eth_fee_basis_points,
    debug_set_eth_gas_price,
    debug_set_key_in_db_to_value,
    debug_update_incremerkle,
    debug_withdraw_fees_and_save_in_db,
    get_enclave_state,
    get_latest_block_numbers,
    maybe_add_vault_contract_address_to_db,
    maybe_initialize_eos_core,
    maybe_initialize_eth_core,
    sign_ascii_msg_with_eth_key_with_no_prefix,
    sign_hex_msg_with_eth_key_with_prefix,
    submit_eos_block_to_core,
    submit_eth_block_to_core,
};

use crate::{
    get_cli_args::{get_cli_args, CliArgs},
    usage_info::USAGE_INFO,
};

fn main() {
    match program() {
        Ok(s) => {
            info!("{}", s);
            println!("{}", s);
        },
        Err(e) => {
            error!("{}", e);
            println!("{}", e);
            std::process::exit(1);
        },
    }
}

fn program() -> Result<String> {
    init_logger()?;
    let db = get_db()?;
    let cli_args = get_cli_args(USAGE_INFO)?;
    Ok(match cli_args {
        CliArgs {
            cmd_debugMigrateSingle: true,
            ..
        } => {
            info!("✔ Debug migrating single token from vault..");
            Ok(debug_get_erc20_vault_migrate_single_tx(
                &db,
                &cli_args.arg_ethAddress,
                &cli_args.arg_tokenAddress,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugAddDebugSigners: true,
            ..
        } => {
            info!("✔ Debug adding mulitple debug signers...");
            Ok(debug_add_multiple_debug_signers(
                &db,
                &cli_args.arg_debugSignersJson,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_initializeEos: true,
            ..
        } => {
            info!("✔ Maybe initializing EOS core...");
            Ok(maybe_initialize_eos_core(
                &db,
                &cli_args.flag_chainId,
                &cli_args.arg_eosJson,
            )?)
        },
        CliArgs {
            cmd_initializeEth: true,
            ..
        } => {
            info!("✔ Maybe initializing ETH core...");
            let chain_id = cli_args.flag_chainId.parse()?;
            Ok(maybe_initialize_eth_core(
                &db,
                &cli_args.arg_blockJson,
                chain_id,
                cli_args.flag_gasPrice,
                cli_args.flag_confs,
                &cli_args.flag_vaultAddress,
            )?)
        },
        CliArgs {
            cmd_debugReprocessEosBlock: true,
            ..
        } => {
            info!("✔ Debug reprocess EOS block...");
            Ok(debug_reprocess_eos_block(
                &db,
                &cli_args.arg_blockJson,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugReprocessEosBlockWithNonce: true,
            ..
        } => {
            info!("✔ Debug reprocess EOS block with nonce...");
            Ok(debug_reprocess_eos_block_with_nonce(
                &db,
                &cli_args.arg_blockJson,
                cli_args.arg_nonce,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugReprocessEthBlock: true,
            ..
        } => {
            info!("✔ Debug reprocess ETH block...");
            Ok(debug_reprocess_eth_block(
                &db,
                &cli_args.arg_blockJson,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugAddSupportedToken: true,
            ..
        } => {
            info!("✔ Debug getting `addSupportedToken` signed transaction...");
            Ok(debug_get_add_supported_token_tx(
                &db,
                &cli_args.arg_ethAddress,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugRemoveSupportedToken: true,
            ..
        } => {
            info!("✔ Debug getting `removeSupportedToken` signed transaction...");
            Ok(debug_get_remove_supported_token_tx(
                &db,
                &cli_args.arg_ethAddress,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugMigrateContract: true,
            ..
        } => {
            info!("✔ Debug getting `migrate` transaction...");
            Ok(debug_get_perc20_migration_tx(
                &db,
                &cli_args.arg_ethAddress,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugAddDictionaryEntry: true,
            ..
        } => {
            info!("✔ Debug adding `EosErc20DictionaryJson...");
            Ok(debug_add_token_dictionary_entry(
                &db,
                &cli_args.arg_entryJson,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugRemoveDictionaryEntry: true,
            ..
        } => {
            info!("✔ Debug removing `EosErc20DictionaryJson...");
            Ok(debug_remove_token_dictionary_entry(
                &db,
                &cli_args.arg_ethAddress,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugGetAllDbKeys: true,
            ..
        } => {
            info!("✔ Debug getting all DB keys...");
            Ok(debug_get_all_db_keys(&db, &cli_args.flag_sig)?)
        },
        CliArgs {
            cmd_getEnclaveState: true,
            ..
        } => {
            info!("✔ Getting core state...");
            Ok(get_enclave_state(&db)?)
        },
        CliArgs {
            cmd_getLatestBlockNumbers: true,
            ..
        } => {
            info!("✔ Maybe getting block numbers...");
            Ok(get_latest_block_numbers(&db)?)
        },
        CliArgs {
            cmd_debugGetKeyFromDb: true,
            ..
        } => {
            info!("✔ Maybe getting a key from the database...");
            Ok(debug_get_key_from_db(
                &db,
                &cli_args.arg_key,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugUpdateIncremerkle: true,
            ..
        } => {
            info!("✔ Debug updating EOS incremerkle...");
            Ok(debug_update_incremerkle(
                &db,
                &cli_args.arg_eosJson,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_submitEosBlock: true,
            ..
        } => {
            info!("✔ Submitting EOS block to core...");
            Ok(submit_eos_block_to_core(&db, &cli_args.arg_blockJson)?)
        },
        CliArgs {
            cmd_submitEthBlock: true,
            ..
        } => {
            info!("✔ Submitting ETH block to core...");
            Ok(submit_eth_block_to_core(&db, &cli_args.arg_blockJson)?)
        },
        CliArgs {
            cmd_enableEosProtocolFeature: true,
            ..
        } => {
            info!("✔ Debug enabling EOS protocol feature...");
            Ok(debug_enable_eos_protocol_feature(
                &db,
                &cli_args.arg_featureHash,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_disableEosProtocolFeature: true,
            ..
        } => {
            info!("✔ Debug disabling EOS protocol feature...");
            Ok(debug_disable_eos_protocol_feature(
                &db,
                &cli_args.arg_featureHash,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugAddEosSchedule: true,
            ..
        } => {
            info!("✔ Adding EOS schedule to database...");
            Ok(debug_add_new_eos_schedule(
                &db,
                &cli_args.arg_scheduleJson,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_signHexMsgWithEthKeyWithPrefix: true,
            ..
        } => {
            info!("✔ Signing HEX message with ETH key & ETH-specific prefix...");
            Ok(sign_hex_msg_with_eth_key_with_prefix(&db, &cli_args.arg_message)?)
        },
        CliArgs {
            cmd_signMessageWithEthKey: true,
            ..
        }
        | CliArgs {
            cmd_signAsciiMsgWithEthKeyWithNoPrefix: true,
            ..
        } => {
            info!("✔ Signing ASCII message with ETH key & NO prefix...");
            Ok(sign_ascii_msg_with_eth_key_with_no_prefix(&db, &cli_args.arg_message)?)
        },
        CliArgs {
            cmd_debugSetKeyInDbToValue: true,
            ..
        } => {
            info!("✔ Setting a key in the database to a value...");
            Ok(debug_set_key_in_db_to_value(
                &db,
                &cli_args.arg_key,
                &cli_args.arg_value,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetEthGasPrice: true,
            ..
        } => {
            info!("✔ Debug setting ETH gas price to {} Wei...", cli_args.arg_wei);
            Ok(debug_set_eth_gas_price(
                &db,
                cli_args.arg_wei,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_addVaultContractAddress: true,
            ..
        } => {
            info!("✔ Adding ERC20 vault contract address to db...");
            Ok(maybe_add_vault_contract_address_to_db(&db, &cli_args.arg_ethAddress)?)
        },
        CliArgs {
            cmd_debugResetEthChain: true,
            ..
        } => {
            info!("✔ Debug resetting ETH chain...");
            Ok(debug_reset_eth_chain(
                &db,
                &cli_args.arg_blockJson,
                cli_args.flag_confs,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetEthAccountNonce: true,
            ..
        } => {
            info!("✔ Debug setting ETH account nonce...");
            Ok(debug_set_eth_account_nonce(
                &db,
                cli_args.arg_nonce,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs { flag_version: true, .. } => {
            let app_type = option_env!("CARGO_PKG_NAME").unwrap_or("unknown");
            let app_version = option_env!("CARGO_PKG_VERSION").unwrap_or("unkownn");
            Ok(format!("{{app_type: {app_type}, app_version: {app_version}}}"))
        },
        CliArgs {
            cmd_debugReprocessEthBlockWithFeeAccrual: true,
            ..
        } => {
            info!("✔ Debug reprocessing ETH block with fee accrual...");
            Ok(debug_reprocess_eth_block_with_fee_accrual(
                &db,
                &cli_args.arg_blockJson,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugReprocessEosBlockWithFeeAccrual: true,
            ..
        } => {
            info!("✔ Debug reprocessing EOS block with fee accrual...");
            Ok(debug_reprocess_eos_block_with_fee_accrual(
                &db,
                &cli_args.arg_blockJson,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetEthFeeBasisPoints: true,
            ..
        } => {
            info!("✔ Debug setting ETH fee basis points...");
            Ok(debug_set_eth_fee_basis_points(
                &db,
                &cli_args.arg_address,
                cli_args.arg_basisPoints,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetEosFeeBasisPoints: true,
            ..
        } => {
            info!("✔ Debug setting EOS fee basis points...");
            Ok(debug_set_eos_fee_basis_points(
                &db,
                &cli_args.arg_address,
                cli_args.arg_basisPoints,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugWithdrawFees: true,
            ..
        } => {
            info!("✔ Debug withdrawing fees...");
            Ok(debug_withdraw_fees_and_save_in_db(
                &db,
                &cli_args.arg_tokenAddress,
                &cli_args.arg_recipientAddress,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetAccruedFees: true,
            ..
        } => {
            info!("✔ Debug setting accrued fees...");
            Ok(debug_set_accrued_fees_in_dictionary(
                &db,
                &cli_args.arg_ethAddress,
                &cli_args.arg_amount,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetEosAccountNonce: true,
            ..
        } => {
            info!("✔ Debug setting EOS account nonce...");
            Ok(debug_set_eos_account_nonce(
                &db,
                cli_args.arg_nonce,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugAddDebugSigner: true,
            ..
        } => {
            info!("✔ Debug adding debug signer...");
            Ok(debug_add_debug_signer(
                &db,
                &cli_args.arg_name,
                &cli_args.arg_ethAddress,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugRemoveDebugSigner: true,
            ..
        } => {
            info!("✔ Debug removing debug signer...");
            Ok(debug_remove_debug_signer(
                &db,
                &cli_args.arg_ethAddress,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        _ => Err(AppError::Custom(USAGE_INFO.to_string())),
    }?)
}
