use common::types::Result;
use int_on_algo::CORE_TYPE;

make_cli_args_struct!(
    CORE_TYPE;
    flag_fee => u64,
    flag_confs => u64,
    flag_note => String,
    flag_chainId => u64,
    flag_gasPrice => u64,
    flag_version => bool,
    flag_userData => String,
    flag_genesisId => String,
    flag_vaultAddress => String,
    flag_routerAddress => String,
    arg_key => String,
    arg_nonce => u64,
    arg_appId => u64,
    arg_amount => u64,
    arg_assetId => u64,
    arg_gasPrice => u64,
    arg_value => String,
    arg_firstValid => u64,
    arg_receiver => String,
    arg_entryJson => String,
    arg_evmAddress => String,
    arg_vaultAddress => String,
    arg_routerAddress => String,
    arg_metadataChainId => String,
    arg_destinationAddress => String,
    cmd_optInToApp => bool,
    cmd_optInToAsset => bool,
    cmd_initializeInt => bool,
    cmd_debugAlgoPayTx => bool,
    cmd_submitIntBlock => bool,
    cmd_initializeAlgo => bool,
    cmd_submitIntBlocks => bool,
    cmd_getEnclaveState => bool,
    cmd_submitAlgoBlock => bool,
    cmd_debugResetIntChain => bool,
    cmd_debugResetAlgoChain => bool,
    cmd_debugSetIntGasPrice => bool,
    cmd_getLatestBlockNumbers => bool,
    cmd_debugAddSupportedToken => bool,
    cmd_encodeAlgoNoteMetadata => bool,
    cmd_debugReprocessIntBlock => bool,
    cmd_debugSetIntAccountNonce => bool,
    cmd_debugAddDictionaryEntry => bool,
    cmd_debugReprocessAlgoBlock => bool,
    cmd_debugSetAlgoAccountNonce => bool,
    cmd_debugRemoveDictionaryEntry => bool,
    cmd_debugReprocessAlgoBlockWithNonce => bool
);

impl CliArgs {
    pub fn maybe_set_entry_json(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_debugAddDictionaryEntry {
            self.read_file_to_string().map(|s| self.update_arg_entry_json(s))
        } else {
            Ok(self)
        }
    }
}

pub fn get_cli_args(usage_info: &str) -> Result<CliArgs> {
    CliArgs::parse(usage_info).and_then(CliArgs::maybe_set_entry_json)
}
