pub const MAX_DISCOUNT_RATE: u32 = 5000;
pub const DEFAULT_PAYER_SCORE: u32 = 50;
pub const DEFAULT_LP_SCORE: u32 = 50;
pub const TOP_PAYERS_CAPACITY: u32 = 50;
pub const CONTRACT_VERSION: &str = "1.0.0";
/// Issue #539: Current storage schema version. Bump this whenever the
/// persistent/instance storage layout changes so that `migrate()` can
/// detect and apply incremental upgrades.
pub const CURRENT_STORAGE_VERSION: u32 = 2;
