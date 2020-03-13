use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        eos_types::{
            ActionData,
            ActionsData,
            RedeemParams,
        },
    },
};

impl RedeemParams {
    pub fn from_action_data(
        action_data: &ActionData,
    ) -> Result<Self> {
        Ok(
            RedeemParams {
                from: action_data.action_params.sender.clone(),
                recipient: action_data.action_params.memo.clone(),
                amount: action_data.action_params.quantity.clone(),
                originating_tx_id: action_data.action_proof.tx_id.clone(),
            }
        )
    }
}

fn parse_redeem_params_from_actions_data(
    actions_data: &ActionsData
) -> Result<Vec<RedeemParams>> {
    actions_data
        .iter()
        .map(|action_data| RedeemParams::from_action_data(action_data))
        .collect()
}

pub fn maybe_parse_redeem_params_and_put_in_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Parsing redeem params from actions data...");
    parse_redeem_params_from_actions_data(&state.actions_data)
        .and_then(|params| {
            debug!("✔ Parsed {} sets of params!", params.len());
            state.add_redeem_params(params)
        })
}
