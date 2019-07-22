pub mod ledger;
pub mod pool;

#[derive(Debug, Serialize, Deserialize)]
pub struct IndyConfig {
    pub crypto_thread_pool_size : Option<usize>,
    pub collect_backtrace: Option<bool>,
    pub freshness_threshold: Option<u64>
}