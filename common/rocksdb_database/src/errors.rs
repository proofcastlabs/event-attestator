quick_error! {
    #[derive(Debug)]
    pub enum RocksdbDatabaseError {
        RocksDbError(err: rocksdb::Error) {
            from()
            display("✘ Rocks DB error: {}", err)
        }
    }
}
