extern crate docopt;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lib;
#[macro_use]
extern crate paste;

mod get_cli_args;
mod usage_info;

use lib::{
    btc_on_eth::{
        debug_add_debug_signer,
        debug_add_multiple_debug_signers,
        debug_add_multiple_utxos,
        debug_clear_all_utxos,
        debug_consolidate_utxos,
        debug_consolidate_utxos_to_address,
        debug_get_all_db_keys,
        debug_get_child_pays_for_parent_btc_tx,
        debug_get_fee_withdrawal_tx,
        debug_get_key_from_db,
        debug_get_signed_erc777_change_pnetwork_tx,
        debug_get_signed_erc777_proxy_change_pnetwork_by_proxy_tx,
        debug_get_signed_erc777_proxy_change_pnetwork_tx,
        debug_maybe_add_utxo_to_db,
        debug_put_btc_on_eth_peg_in_basis_points_in_db,
        debug_put_btc_on_eth_peg_out_basis_points_in_db,
        debug_remove_debug_signer,
        debug_remove_utxo,
        debug_reprocess_btc_block,
        debug_reprocess_btc_block_with_fee_accrual,
        debug_reprocess_btc_block_with_nonce,
        debug_reprocess_eth_block,
        debug_reprocess_eth_block_with_fee_accrual,
        debug_reset_eth_chain,
        debug_set_accrued_fees,
        debug_set_btc_account_nonce,
        debug_set_btc_fee,
        debug_set_eth_account_nonce,
        debug_set_eth_gas_price,
        debug_set_key_in_db_to_value,
        get_all_utxos,
        get_enclave_state,
        get_latest_block_numbers,
        maybe_add_erc777_contract_address,
        maybe_initialize_btc_enclave,
        maybe_initialize_eth_enclave,
        sign_ascii_msg_with_eth_key_with_no_prefix,
        sign_hex_msg_with_eth_key_with_prefix,
        submit_btc_block_to_enclave,
        submit_eth_block_to_enclave,
    },
    get_db,
    init_logger,
    AppError,
};

use crate::{
    get_cli_args::{get_cli_args, CliArgs},
    usage_info::USAGE_INFO,
};

fn main() -> Result<(), AppError> {
    let db = get_db()?;
    match init_logger()
        .and_then(|_| get_cli_args(USAGE_INFO))
        .and_then(|cli_args| match cli_args {
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
                cmd_initializeEth: true,
                ..
            } => {
                info!("✔ Initializing ETH enclave...");
                Ok(maybe_initialize_eth_enclave(
                    &db,
                    &cli_args.arg_blockJson,
                    cli_args.flag_chainId,
                    cli_args.flag_gasPrice,
                    cli_args.flag_confs,
                    &cli_args.flag_pTokenAddress,
                )?)
            },
            CliArgs {
                cmd_debugSetAccruedFees: true,
                ..
            } => {
                info!("✔ Debug setting accrued fees to {}...", cli_args.arg_amount);
                Ok(debug_set_accrued_fees(&db, cli_args.arg_amount, &cli_args.flag_sig)?)
            },
            CliArgs {
                cmd_initializeBtc: true,
                ..
            } => {
                info!("✔ Initializing BTC enclave...");
                Ok(maybe_initialize_btc_enclave(
                    &db,
                    &cli_args.arg_blockJson,
                    cli_args.flag_fee,
                    cli_args.flag_difficulty,
                    &cli_args.flag_network,
                    cli_args.flag_confs,
                )?)
            },
            CliArgs {
                cmd_debugGetChildPaysForParentTx: true,
                ..
            } => {
                info!("✔ Debug getting `child-pays-for-parent` tx...");
                Ok(debug_get_child_pays_for_parent_btc_tx(
                    &db,
                    cli_args.flag_fee,
                    &cli_args.arg_txId,
                    cli_args.arg_vOut,
                    &CliArgs::core_type(),
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_getEnclaveState: true,
                ..
            } => {
                info!("✔ Getting enclave state...");
                Ok(get_enclave_state(&db)?)
            },
            CliArgs {
                cmd_debugGetAllDbKeys: true,
                ..
            } => {
                info!("✔ Debug getting all DB keys....");
                Ok(debug_get_all_db_keys(&db, &cli_args.flag_sig)?)
            },
            CliArgs {
                cmd_getLatestBlockNumbers: true,
                ..
            } => {
                info!("✔ Maybe getting block numbers...");
                Ok(get_latest_block_numbers(&db)?)
            },
            CliArgs {
                cmd_getAllUtxos: true, ..
            } => {
                info!("✔ Getting all UTXOs from the database...");
                Ok(get_all_utxos(&db)?)
            },
            CliArgs {
                cmd_debugClearAllUtxos: true,
                ..
            } => {
                info!("✔ Debug clearing all UTXOs from the database...");
                Ok(debug_clear_all_utxos(&db, &CliArgs::core_type(), &cli_args.flag_sig)?)
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
                cmd_debugMaybeAddUtxoToDb: true,
                ..
            } => {
                info!("✔ Debug maybe adding UTXO to db...");
                Ok(debug_maybe_add_utxo_to_db(
                    &db,
                    &cli_args.arg_blockJson,
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_debugAddUtxos: true,
                ..
            } => {
                info!("✔ Debug adding multiple UTXOs...");
                Ok(debug_add_multiple_utxos(
                    &db,
                    &cli_args.arg_utxosJson,
                    &CliArgs::core_type(),
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_addErc777ContractAddress: true,
                ..
            } => {
                info!("✔ Maybe adding ERC777 contract address...");
                Ok(maybe_add_erc777_contract_address(&db, &cli_args.arg_address)?)
            },
            CliArgs {
                cmd_debugReprocessEthBlock: true,
                ..
            } => {
                info!("✔ Debug reprocessing ETH block...");
                Ok(debug_reprocess_eth_block(
                    &db,
                    &cli_args.arg_blockJson,
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_debugReprocessBtcBlock: true,
                ..
            } => {
                info!("✔ Debug reprocessing BTC block...");
                Ok(debug_reprocess_btc_block(
                    &db,
                    &cli_args.arg_blockJson,
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_debugReprocessBtcBlockWithNonce: true,
                ..
            } => {
                info!("✔ Debug reprocessing BTC block with nonce...");
                Ok(debug_reprocess_btc_block_with_nonce(
                    &db,
                    &cli_args.arg_blockJson,
                    cli_args.arg_nonce,
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_submitEthBlock: true,
                ..
            } => {
                info!("✔ Submitting ETH block to enclave...");
                Ok(submit_eth_block_to_enclave(&db, &cli_args.arg_blockJson)?)
            },
            CliArgs {
                cmd_submitBtcBlock: true,
                ..
            } => {
                info!("✔ Submitting BTC block to enclave...");
                Ok(submit_btc_block_to_enclave(&db, &cli_args.arg_blockJson)?)
            },
            CliArgs {
                cmd_debugRemoveUtxo: true,
                ..
            } => {
                info!("✔ Debug removing UTXO...");
                Ok(debug_remove_utxo(
                    &db,
                    &cli_args.arg_txId,
                    cli_args.arg_vOut,
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
                cmd_debugConsolidateUtxos: true,
                ..
            } => {
                info!("✔ Debug consolidating utxos...");
                Ok(debug_consolidate_utxos(
                    &db,
                    cli_args.flag_fee,
                    cli_args.arg_numUtxos,
                    &CliArgs::core_type(),
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_debugConsolidateUtxosToAddress: true,
                ..
            } => {
                info!("✔ Debug consolidating UTXOS...");
                Ok(debug_consolidate_utxos_to_address(
                    &db,
                    cli_args.flag_fee,
                    cli_args.arg_numUtxos,
                    &cli_args.arg_address,
                    &CliArgs::core_type(),
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_debugErc777ChangePNetwork: true,
                ..
            } => {
                info!("✔ Debug getting `changePNetwork` tx...");
                Ok(debug_get_signed_erc777_change_pnetwork_tx(
                    &db,
                    &cli_args.arg_address,
                    &cli_args.flag_sig,
                )?)
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
                cmd_debugErc777ProxyChangePNetwork: true,
                ..
            } => {
                info!("✔ Debug getting `changePNetwork` in the proxy tx...");
                Ok(debug_get_signed_erc777_proxy_change_pnetwork_tx(
                    &db,
                    &cli_args.arg_address,
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_debugWithdrawFees: true,
                ..
            } => {
                info!("✔ Debug withdrawing fees...");
                Ok(debug_get_fee_withdrawal_tx(
                    &db,
                    &cli_args.arg_address,
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_debugErc777ProxyChangePNetworkByProxy: true,
                ..
            } => {
                info!("✔ Debug getting `changePNetworkByProxy` tx...");
                Ok(debug_get_signed_erc777_proxy_change_pnetwork_by_proxy_tx(
                    &db,
                    &cli_args.arg_address,
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_debugSetPegInFee: true,
                ..
            } => {
                info!(
                    "✔ Debug setting peg-in fee to {} basis points...",
                    cli_args.arg_basisPoints
                );
                Ok(debug_put_btc_on_eth_peg_in_basis_points_in_db(
                    &db,
                    cli_args.arg_basisPoints,
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_debugSetEthGasPrice: true,
                ..
            } => {
                info!("✔ Debug setting ETH gas price to {} Wei..", cli_args.arg_wei);
                Ok(debug_set_eth_gas_price(
                    &db,
                    cli_args.arg_wei,
                    &CliArgs::core_type(),
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_debugSetPegOutFee: true,
                ..
            } => {
                info!(
                    "✔ Debug setting peg-out fee to {} basis points...",
                    cli_args.arg_basisPoints
                );
                Ok(debug_put_btc_on_eth_peg_out_basis_points_in_db(
                    &db,
                    cli_args.arg_basisPoints,
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs {
                cmd_debugSetBtcFee: true,
                ..
            } => {
                info!("✔ Debug setting BTC fee to {} Satoshis-per-byte...", cli_args.arg_fee);
                Ok(debug_set_btc_fee(
                    &db,
                    cli_args.arg_fee,
                    &CliArgs::core_type(),
                    &cli_args.flag_sig,
                )?)
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
            CliArgs {
                cmd_debugSetBtcAccountNonce: true,
                ..
            } => {
                info!("✔ Debug setting BTC account nonce...");
                Ok(debug_set_btc_account_nonce(
                    &db,
                    cli_args.arg_nonce,
                    &CliArgs::core_type(),
                    &cli_args.flag_sig,
                )?)
            },
            CliArgs { flag_version: true, .. } => get_versions!(),
            CliArgs {
                cmd_debugReprocessEthBlockAccruingFees: true,
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
                cmd_debugReprocessBtcBlockAccruingFees: true,
                ..
            } => {
                info!("✔ Debug reprocessing BTC block with fee accrual...");
                Ok(debug_reprocess_btc_block_with_fee_accrual(
                    &db,
                    &cli_args.arg_blockJson,
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
                    &cli_args.arg_address,
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
                    &cli_args.arg_address,
                    &CliArgs::core_type(),
                    &cli_args.flag_sig,
                )?)
            },
            _ => Err(AppError::Custom(USAGE_INFO.to_string())),
        }) {
        Ok(json_string) => {
            info!("{}", json_string);
            println!("{}", json_string);
            Ok(())
        },
        Err(e) => {
            error!("{}", e);
            println!("{}", e);
            std::process::exit(1);
        },
    }
}
