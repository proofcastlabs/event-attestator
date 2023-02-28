quick_error! {
    #[derive(Debug)]
    pub enum JsonRpcDatabaseError {
        EnvError(err: std::env::VarError) {
            from()
            display("✘ Env var error: {}", err)
        }
        /*
        Custom(err: String) {
            from()
            from(err: &str) -> (err.into())
            display("✘ Program Error: {}", err)
        }
        IOError(err: std::io::Error) {
            from()
            display("✘ I/O Error: {}", err)
        }
        SystemTimeError(err: std::time::SystemTimeError) {
            from()
            display("✘ System time error: {}", err)
        }
        PTokenAppError(err: common::DatabaseError) {
            from()
            display("{}", err)
        }
        SetLoggerError(err: log::SetLoggerError) {
            from()
            display("✘ SetLogger error: {}", err)
        }
        ParseIntError(err: std::num::ParseIntError) {
            from()
            display("✘ Parse Int error: {}", err)
        }
        DocoptError(err: docopt::Error) {
            from()
            display("✘ Docopt error: {}", err)
        }
        SerdeJsonError(err: serde_json::Error) {
            from()
            display("✘ Serde-Json Error: {}", err)
        }
        FlexiLoggerError(err: flexi_logger::FlexiLoggerError) {
            from()
            display("✘ Flexilogger error: {}", err)
        }
        DebugSignerError(err: String) {
            // NOTE: This is to get around the above custom error, which canmnot be parsed as JSON
            // due to the message it displays. However we can't get rid of that due to legacy
            // functions using the ✘ to detect errors :/
            display("{}", err)
        }
        */
    }
}
