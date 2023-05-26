use btc_on_int::CORE_TYPE;
use common::types::Result;

make_cli_args_struct!(
    CORE_TYPE;
    flag_fee => u64,
    flag_confs => u64,
    flag_chainId => u64,
    flag_gasPrice => u64,
    flag_version => bool,
    flag_network => String,
    flag_difficulty => u64,
    flag_recipient => String,
    flag_ethNetwork => String,
    flag_routerAddress => String,
    flag_pTokenAddress => String,
    arg_wei => u64,
    arg_fee => u64,
    arg_vOut => u32,
    arg_nonce => u64,
    arg_key => String,
    arg_txId => String,
    arg_value => String,
    arg_numUtxos => usize,
    arg_message => String,
    arg_address => String,
    arg_utxosJson => String,
    cmd_initializeInt => bool,
    cmd_initializeBtc => bool,
    cmd_submitBtcBlock => bool,
    cmd_getEnclaveState => bool,
    cmd_getLatestBlockNumbers => bool,
    cmd_getAllUtxos => bool,
    cmd_debugAddUtxos => bool,
    cmd_submitIntBlock => bool,
    cmd_debugSetBtcFee => bool,
    cmd_debugRemoveUtxo => bool,
    cmd_submitIntBlocks => bool,
    cmd_debugClearAllUtxos => bool,
    cmd_debugResetIntChain => bool,
    cmd_debugSetIntGasPrice => bool,
    cmd_debugMaybeAddUtxoToDb => bool,
    cmd_debugConsolidateUtxos => bool,
    cmd_debugReprocessBtcBlock => bool,
    cmd_debugReprocessIntBlock => bool,
    cmd_debugSetIntAccountNonce => bool,
    cmd_debugSetBtcAccountNonce => bool,
    cmd_debugErc777ChangePNetwork => bool,
    cmd_debugGetChildPaysForParentTx => bool,
    cmd_debugConsolidateUtxosToAddress => bool,
    cmd_debugErc777ProxyChangePNetwork => bool,
    cmd_debugReprocessBtcBlockWithNonce => bool,
    cmd_debugErc777ProxyChangePNetworkByProxy => bool,
    cmd_signMessageWithIntKey => bool,
    cmd_signHexMsgWithIntKeyWithPrefix => bool,
    cmd_signAsciiMsgWithIntKeyWithNoPrefix => bool
);

impl CliArgs {
    pub fn maybe_update_utxos(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_debugAddUtxos {
            info!("✔ Updating UTXOS in CLI args...");
            self.read_file_to_string().map(|s| self.update_arg_utxos_json(s))
        } else {
            Ok(self)
        }
    }
}

pub fn get_cli_args(usage_info: &str) -> Result<CliArgs> {
    CliArgs::parse(usage_info).and_then(CliArgs::maybe_update_utxos)
}
