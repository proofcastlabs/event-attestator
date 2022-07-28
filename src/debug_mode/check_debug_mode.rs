use crate::{constants::DEBUG_MODE, types::Result};

pub fn check_debug_mode() -> Result<()> {
    if DEBUG_MODE {
        info!("✔ Application is in debug mode! Continuing...");
        Ok(())
    } else {
        Err("✘ Application NOT in debug mode - exiting!".into())
    }
}
