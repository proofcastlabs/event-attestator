use std::str::FromStr;

use common_eth::EthSubmissionMaterial;
use common_network_ids::NetworkId;
use derive_getters::{Dissolve, Getters};
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

use crate::WebSocketMessagesError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Dissolve)]
pub struct WebSocketMessagesResetChainArgs {
    confs: u64,
    validate: bool,
    network_id: NetworkId,
    use_latest_block: bool,
    block_num: Option<u64>,
    hub: Option<EthAddress>,
    block: Option<EthSubmissionMaterial>,
}

impl WebSocketMessagesResetChainArgs {
    pub fn add_sub_mat(&mut self, m: EthSubmissionMaterial) {
        self.block = Some(m)
    }
}

// NOTE: Because these args are passed in via an RPC call
impl TryFrom<Vec<String>> for WebSocketMessagesResetChainArgs {
    type Error = WebSocketMessagesError;

    fn try_from(args: Vec<String>) -> Result<Self, WebSocketMessagesError> {
        // NOTE: Example: ["EthereumMainnet", "latest", "10", "false"]

        if args.is_empty() {
            return Err(WebSocketMessagesError::CannotCreate(args));
        };

        let expected_num_args = 4;
        if args.len() < expected_num_args {
            return Err(WebSocketMessagesError::NotEnoughArgs {
                got: args.len(),
                expected: expected_num_args,
                args,
            });
        }

        let arg1 = args[0].clone();

        let network_id = NetworkId::try_from(&arg1).map_err(|_| WebSocketMessagesError::UnrecognizedNetworkId(arg1))?;

        let arg2 = args[1].clone();
        let use_latest_block = matches!(arg2.to_lowercase().as_ref(), "latest");

        let block_num = if use_latest_block {
            None
        } else {
            let n = arg2
                .parse::<u64>()
                .map_err(|_| WebSocketMessagesError::ParseInt(arg2))?;
            Some(n)
        };

        let arg3 = args[2].clone();
        let confs = arg3
            .parse::<u64>()
            .map_err(|_| WebSocketMessagesError::ParseInt(arg3))?;

        let validate = matches!(args[3].as_ref(), "true");

        let block = None;

        let hub = if args.len() > expected_num_args {
            EthAddress::from_str(&args[4]).ok()
        } else {
            None
        };

        Ok(Self {
            hub,
            block,
            confs,
            validate,
            block_num,
            network_id,
            use_latest_block,
        })
    }
}
