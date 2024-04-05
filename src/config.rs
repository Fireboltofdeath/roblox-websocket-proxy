use std::time::Duration;

/// The duration that the long polling connections should be kept alive.
pub const KEEP_ALIVE: Duration = Duration::from_secs(20);

/// The maximum duration that can be provided under `batch_ms`.
pub const MAX_BATCH_DURATION: Duration = Duration::from_secs(5);
