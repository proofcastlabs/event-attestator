use std::result::Result;

use common::DatabaseInterface;
use common_eth::{Chain, ChainDbUtils, EthSubmissionMaterials};

use super::process_single;
use crate::{NetworkConfig, ProcessorOutput, SentinelError, SignedEvents};

pub fn process_batch<D: DatabaseInterface>(
    db: &D,
    batch: &EthSubmissionMaterials,
    validate: bool,
    network_config: &NetworkConfig,
    reprocess: bool,
    dry_run: bool,
) -> Result<ProcessorOutput, SentinelError> {
    let network_id = network_config.network_id();
    info!("processing {network_id} batch of submission material...");

    let c_db_utils = ChainDbUtils::new(db);

    let mut chain = Chain::get(&c_db_utils, network_id.try_into()?)?;

    let signed_events = SignedEvents::from(
        batch
            .iter()
            .map(|sub_mat| {
                process_single(
                    db,
                    sub_mat.clone(),
                    validate,
                    dry_run,
                    network_config,
                    reprocess,
                    &mut chain,
                )
            })
            .collect::<Result<Vec<SignedEvents>, SentinelError>>()?,
    );
    info!("finished processing {network_id} submission material");

    let r = ProcessorOutput::new(network_id, batch.get_last_block_num()?, signed_events)?;
    Ok(r)
}
