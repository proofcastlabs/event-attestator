use eos_primitives::{
    BlockHeader as EosBlockHeader,
    ProducerScheduleV2 as EosProducerScheduleV2,
};
use crate::{
    types::Result,
    errors::AppError,
    traits::DatabaseInterface,
    btc_on_eos::eos::eos_state::EosState,
};

pub const PRODUCER_REPETITIONS: u64 = 12;
pub const EOS_INTERVAL_MILLIS: u64 = 500;
pub const EOS_EPOCH_MILLIS: u64 = 946_684_800_000;

fn get_producer_index(
    num_producers: u64,
    block_timestamp: u64,
) -> usize {
    debug!("  Num producers: {}", num_producers);
    debug!("Block timestamp: {}", block_timestamp);
    let slot = (block_timestamp - EOS_EPOCH_MILLIS) / EOS_INTERVAL_MILLIS;
    (slot % (num_producers * PRODUCER_REPETITIONS)) as usize
}

fn validate_producer_slot(
    schedule: &EosProducerScheduleV2,
    block: &EosBlockHeader,
) -> Result<()> {
    let index = get_producer_index(
        schedule.producers.len() as u64,
        block.timestamp.as_u32() as u64,
    );
    match block.producer == schedule.producers[index].producer_name {
        true => Ok(()),
        _ => {
            debug!(" Calculated index: {}", index);
            debug!("Expected producer: {}", block.producer.to_string());
            debug!(
                "  Actual producer: {}",
                schedule.producers[index].producer_name.to_string()
            );
            Err(AppError::Custom("✘ Producer slot not valid!".to_string()))
        }
    }
}

pub fn validate_producer_slot_of_block_in_state<D>(
    state: EosState<D>
) ->Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Validation slot of producer of block...");
    validate_producer_slot(
        state.get_active_schedule()?,
        state.get_eos_block_header()?,
    )
        .and(Ok(state))
}
