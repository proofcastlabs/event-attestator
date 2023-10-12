use std::result::Result;

use common::{BridgeSide, DatabaseInterface};
use common_eth::{Chain, ChainDbUtils, EthSubmissionMaterials};
use common_metadata::MetadataChainId;
use ethereum_types::Address as EthAddress;

use super::{maybe_handle_actors_propagated_event, process_single};
use crate::{Bytes4, ProcessorOutput, SentinelDbUtils, SentinelError, UserOps};

pub fn process_batch<D: DatabaseInterface>(
    db: &D,
    pnetwork_hub: &EthAddress,
    batch: &EthSubmissionMaterials,
    validate: bool,
    side: BridgeSide,
    network_id: &Bytes4,
    reprocess: bool,
    dry_run: bool,
    mcid: MetadataChainId,
    maybe_governance_address: Option<EthAddress>,
    sentinel_address: EthAddress,
) -> Result<ProcessorOutput, SentinelError> {
    info!("Processing {mcid} batch of submission material...");

    let chain_db_utils = ChainDbUtils::new(db);
    let mut chain = Chain::get(&chain_db_utils, mcid)?;

    let use_db_tx = !dry_run;

    if let Some(ref governance_address) = maybe_governance_address {
        // NOTE: If we find a governance address, it means we're on the governance chain, meaning
        // we need to watch out for `ActorsPropagated` events which are fired after epoch changes.
        // This changes this sentinel's `ActorInclusionProof` which is required to successfully
        // cancel a `UserOp`.
        batch.iter().try_for_each(|sub_mat| {
            maybe_handle_actors_propagated_event(
                &SentinelDbUtils::new(db),
                &mcid,
                governance_address,
                &sentinel_address,
                sub_mat,
            )
        })?;
    }

    let processed_user_ops = UserOps::from(
        batch
            .iter()
            .map(|sub_mat| {
                process_single(
                    db,
                    sub_mat.clone(),
                    pnetwork_hub,
                    validate,
                    use_db_tx,
                    dry_run,
                    side,
                    network_id,
                    reprocess,
                    &mut chain,
                )
            })
            .collect::<Result<Vec<UserOps>, SentinelError>>()?,
    );

    info!("finished processing {side} submission material");
    let r = ProcessorOutput::new(side, batch.get_last_block_num()?, processed_user_ops)?;
    Ok(r)
}
