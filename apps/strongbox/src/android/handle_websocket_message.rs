use std::result::Result;

use common_sentinel::{
    SentinelError,
    WebSocketMessagesEncodable as Msg,
    WebSocketMessagesEncodableDbOps,
    WebSocketMessagesError,
};

use crate::android::{check_and_handle_java_exceptions, constants::PRINT_JAVA_ERRORS, State};

pub fn handle_websocket_message(state: State) -> Result<State, SentinelError> {
    info!("handling web socket message...");

    let msg = state.msg();

    if msg.is_hard_reset() {
        // NOTE: For a hard reset, the db transaction is handled on the java side.
        warn!("skipping starting db transaction due to hard reset");
    } else {
        match state.db().start_transaction() {
            Err(e) => {
                error!("error starting db tx: {e}");
                check_and_handle_java_exceptions(state.env(), PRINT_JAVA_ERRORS)?;
                Err(e)
            },
            Ok(_) => {
                check_and_handle_java_exceptions(state.env(), PRINT_JAVA_ERRORS)?;
                Ok(())
            },
        }?;
    };

    info!("handling websocket msg: '{msg}'...");
    let final_state = match msg {
        Msg::Initialize(args) => super::handlers::init(*args.clone(), state),
        Msg::GetStatus(nids) => super::handlers::get_status(nids.clone(), state),
        Msg::ResetChain(args) => super::handlers::reset_chain(*args.clone(), state),
        Msg::CheckInit(network_id) => super::handlers::check_init(*network_id, state),
        Msg::ProcessBatch(args) => super::handlers::process_batch(*args.clone(), state),
        Msg::GetCoreState(nids) => super::handlers::get_core_state(nids.clone(), state),
        Msg::HardReset(debug_sig) => super::handlers::hard_reset(debug_sig.clone(), state),
        Msg::GetAttestationCertificate => super::handlers::get_attestation_certificate(state),
        Msg::GetLatestBlockInfos(nids) => super::handlers::get_latest_block_infos(nids.clone(), state),
        Msg::GetAttestationSignature(bytes) => super::handlers::get_attestation_signature(bytes.clone(), state),
        Msg::GetPublicKey => super::handlers::get_public_key(state),
        Msg::GetAddress => super::handlers::get_address(state),
        Msg::AddDebugSigners(signers, sig) => super::handlers::add_debug_signers(signers.clone(), sig.clone(), state),
        Msg::RemoveDebugSigner(signer, sig) => super::handlers::remove_debug_signer(signer.clone(), sig.clone(), state),
        Msg::GetRegistrationSignature(owner, nonce, sig) => {
            super::handlers::get_registration_signature(*owner, *nonce, sig.clone(), state)
        },
        Msg::DbOps(WebSocketMessagesEncodableDbOps::Get(k, sig)) => super::handlers::get(k.clone(), sig.clone(), state),
        Msg::DbOps(WebSocketMessagesEncodableDbOps::Delete(k, sig)) => {
            super::handlers::delete(k.clone(), sig.clone(), state)
        },
        Msg::DbOps(WebSocketMessagesEncodableDbOps::Put(k, v, sig)) => {
            super::handlers::put(k.clone(), v.clone(), sig.clone(), state)
        },
        m => Err(WebSocketMessagesError::Unhandled(m.to_string()).into()),
    }?;

    if final_state.msg().is_hard_reset() {
        warn!("skipping ending db transaction due to hard reset");
    } else {
        match final_state.db().end_transaction() {
            Err(e) => {
                error!("error ending db tx: {e}");
                check_and_handle_java_exceptions(final_state.env(), PRINT_JAVA_ERRORS)?;
                Err(e)
            },
            Ok(_) => {
                check_and_handle_java_exceptions(final_state.env(), PRINT_JAVA_ERRORS)?;
                Ok(())
            },
        }?;
    }

    Ok(final_state)
}
