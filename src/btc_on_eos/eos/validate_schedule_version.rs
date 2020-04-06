use eos_primitives::{
    BlockHeader as EosBlockHeader,
    ProducerSchedule as EosProducerSchedule,
};
use crate::btc_on_eos::{
    types::Result,
    errors::AppError,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
    },
};

fn check_bp_schedule_version(
    block_header: &EosBlockHeader,
    active_schedule: &EosProducerSchedule,
) -> Result<()> {
    match block_header.schedule_version == active_schedule.version {
        true => Ok(()),
        _ => Err(AppError::Custom("✘ BP schedule mismatch!".to_string()))
    }
}

pub fn validate_bp_schedule_version<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    check_bp_schedule_version(
        state.get_eos_block_header()?,
        state.get_active_schedule()?,
    )
        .and(Ok(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eos::eos::{
        eos_test_utils::{
            NUM_SAMPLES,
            get_sample_eos_submission_material_n,
        },
    };

    #[test]
    fn should_validate_bp_schedule_versions() {
        vec![0; NUM_SAMPLES]
            .iter()
            .enumerate()
            .map(|(i, _)| get_sample_eos_submission_material_n(i + 1))
            .map(|submission_material|
                 check_bp_schedule_version(
                    &submission_material.block_header,
                    &submission_material.active_schedule,
                 )
             )
            .collect::<Result<Vec<()>>>()
            .unwrap();
    }

    #[test]
    fn should_err_if_schedule_version_invalid() {
        let block_header = get_sample_eos_submission_material_n(1)
            .block_header
            .clone();
        let wrong_schedule = get_sample_eos_submission_material_n(5)
            .active_schedule
            .clone();
        assert_ne!(block_header.schedule_version, wrong_schedule.version);
        let result = check_bp_schedule_version(
            &block_header,
            &wrong_schedule,
        );
        assert!(result.is_err());
    }
}
