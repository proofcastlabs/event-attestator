use btc_on_eth::CORE_TYPE;
use common::types::Result;

make_cli_args_struct!(
    CORE_TYPE;
    flag_fee => u64,
    flag_confs => u64,
    flag_chainId => u8,
    flag_gasPrice => u64,
    flag_version => bool,
    flag_network => String,
    flag_difficulty => u64,
    flag_recipient => String,
    flag_ethNetwork => String,
    flag_pTokenAddress => String,
    arg_wei => u64,
    arg_fee => u64,
    arg_vOut => u32,
    arg_nonce => u64,
    arg_key => String,
    arg_amount => u64,
    arg_txId => String,
    arg_value => String,
    arg_numUtxos => usize,
    arg_message => String,
    arg_address => String,
    arg_basisPoints => u64,
    arg_utxosJson => String,
    cmd_initializeEth => bool,
    cmd_initializeBtc => bool,
    cmd_submitBtcBlock => bool,
    cmd_getEnclaveState => bool,
    cmd_getLatestBlockNumbers => bool,
    cmd_addErc777ContractAddress => bool,
    cmd_getAllUtxos => bool,
    cmd_debugAddUtxos => bool,
    cmd_submitEthBlock => bool,
    cmd_debugSetBtcFee => bool,
    cmd_debugRemoveUtxo => bool,
    cmd_debugSetPegInFee => bool,
    cmd_debugSetPegOutFee => bool,
    cmd_debugWithdrawFees => bool,
    cmd_debugClearAllUtxos => bool,
    cmd_debugSetAccruedFees => bool,
    cmd_debugResetEthChain => bool,
    cmd_debugSetEthGasPrice => bool,
    cmd_debugMaybeAddUtxoToDb => bool,
    cmd_debugConsolidateUtxos => bool,
    cmd_debugReprocessBtcBlock => bool,
    cmd_debugReprocessEthBlock => bool,
    cmd_debugSetEthAccountNonce => bool,
    cmd_debugSetBtcAccountNonce => bool,
    cmd_debugErc777ChangePNetwork => bool,
    cmd_debugGetChildPaysForParentTx => bool,
    cmd_debugErc777ProxyChangePNetwork => bool,
    cmd_debugConsolidateUtxosToAddress => bool,
    cmd_debugReprocessBtcBlockWithNonce => bool,
    cmd_debugReprocessBtcBlockAccruingFees => bool,
    cmd_debugReprocessEthBlockAccruingFees => bool,
    cmd_debugErc777ProxyChangePNetworkByProxy => bool,
    cmd_signMessageWithEthKey => bool,
    cmd_signHexMsgWithEthKeyWithPrefix => bool,
    cmd_signAsciiMsgWithEthKeyWithNoPrefix => bool
);

impl CliArgs {
    pub fn maybe_set_utxos_json(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_debugAddUtxos {
            self.read_file_to_string().map(|s| self.update_arg_utxos_json(s))
        } else {
            Ok(self)
        }
    }
}

pub fn get_cli_args(usage_info: &str) -> Result<CliArgs> {
    CliArgs::parse(usage_info).and_then(CliArgs::maybe_set_utxos_json)
}
// use std::{fs::read_to_string, path::Path};
//
// use docopt::Docopt;
// use lib::types::Result;
//
// use crate::usage_info::USAGE_INFO;
//
// #[allow(non_snake_case)]
// #[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
// pub struct CliArgs {
// pub flag_fee: u64,
// pub flag_confs: u64,
// pub flag_chainId: u8,
// pub flag_file: String,
// pub flag_gasPrice: u64,
// pub flag_network: String,
// pub flag_version: bool,
// pub flag_difficulty: u64,
// pub flag_recipient: String,
// pub flag_ethNetwork: String,
// pub arg_wei: u64,
// pub arg_fee: u64,
// pub arg_vOut: u32,
// pub arg_nonce: u64,
// pub arg_key: String,
// pub arg_amount: u64,
// pub arg_txId: String,
// pub arg_value: String,
// pub arg_numUtxos: usize,
// pub arg_message: String,
// pub arg_address: String,
// pub arg_basisPoints: u64,
// pub arg_blockJson: String,
// pub arg_utxosJson: String,
// pub cmd_initializeEth: bool,
// pub cmd_initializeBtc: bool,
// pub cmd_submitBtcBlock: bool,
// pub cmd_getEnclaveState: bool,
// pub cmd_getLatestBlockNumbers: bool,
// pub cmd_addErc777ContractAddress: bool,
// pub cmd_debugAddUtxos: bool,
// pub cmd_submitEthBlock: bool,
// pub cmd_debugSetBtcFee: bool,
// pub cmd_debugRemoveUtxo: bool,
// pub cmd_debugGetAllUtxos: bool,
// pub cmd_debugSetPegInFee: bool,
// pub cmd_debugSetPegOutFee: bool,
// pub cmd_debugWithdrawFees: bool,
// pub cmd_debugGetAllDbKeys: bool,
// pub cmd_debugGetKeyFromDb: bool,
// pub cmd_debugClearAllUtxos: bool,
// pub cmd_debugSetAccruedFees: bool,
// pub cmd_debugResetEthChain: bool,
// pub cmd_debugSetEthGasPrice: bool,
// pub cmd_debugMaybeAddUtxoToDb: bool,
// pub cmd_debugConsolidateUtxos: bool,
// pub cmd_debugReprocessBtcBlock: bool,
// pub cmd_debugReprocessEthBlock: bool,
// pub cmd_debugSetKeyInDbToValue: bool,
// pub cmd_debugSetEthAccountNonce: bool,
// pub cmd_debugSetBtcAccountNonce: bool,
// pub cmd_debugErc777ChangePNetwork: bool,
// pub cmd_debugGetChildPaysForParentTx: bool,
// pub cmd_debugErc777ProxyChangePNetwork: bool,
// pub cmd_debugReprocessBtcBlockWithNonce: bool,
// pub cmd_debugReprocessBtcBlockAccruingFees: bool,
// pub cmd_debugReprocessEthBlockAccruingFees: bool,
// pub cmd_debugErc777ProxyChangePNetworkByProxy: bool,
// pub cmd_signMessageWithEthKey: bool,
// pub cmd_signHexMsgWithEthKeyWithPrefix: bool,
// pub cmd_signAsciiMsgWithEthKeyWithNoPrefix: bool,
// }
//
// impl CliArgs {
// pub fn update_block_in_cli_args(mut self, block_json: String) -> Result<Self> {
// self.arg_blockJson = block_json;
// Ok(self)
// }
//
// pub fn update_utxos_json_in_cli_args(mut self, json: String) -> Result<Self> {
// self.arg_utxosJson = json;
// Ok(self)
// }
// }
//
// pub fn parse_cli_args() -> Result<CliArgs> {
// match Docopt::new(USAGE_INFO).and_then(|d| d.deserialize()) {
// Ok(cli_args) => Ok(cli_args),
// Err(e) => Err(e.into()),
// }
// }
//
// pub fn maybe_read_block_json_from_file(cli_args: CliArgs) -> Result<CliArgs> {
// match Path::new(&cli_args.flag_file).exists() {
// true => {
// info!("✔ File exists @ path: {}, reading file...", cli_args.flag_file);
// match cli_args {
// CliArgs {
// cmd_debugAddUtxos: true,
// ..
// } => {
// info!("✔ Updating UTXOS in CLI args...");
// cli_args
// .clone()
// .update_utxos_json_in_cli_args(read_to_string(cli_args.flag_file)?)
// },
// _ => {
// info!("✔ Updating block in CLI args...");
// cli_args
// .clone()
// .update_block_in_cli_args(read_to_string(cli_args.flag_file)?)
// },
// }
// },
// false => {
// info!("✔ No file exists @ path: {}, not reading file...", cli_args.flag_file);
// Ok(cli_args)
// },
// }
// }
//
// pub fn get_cli_args() -> Result<CliArgs> {
// parse_cli_args().and_then(maybe_read_block_json_from_file)
// }
