use std::time::Duration;

/// The duration that the long polling connections should be kept alive.
pub const KEEP_ALIVE: Duration = Duration::from_secs(20);

/// The maximum duration that can be provided under `batch_ms`.
pub const MAX_BATCH_DURATION: Duration = Duration::from_secs(5);

/// The maximum time a connection will be kept alive without any messages.
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(55);

/// The maximum time a connection will be kept alive without any poll requests.
pub const CONNECTION_POLL_TIMEOUT: Duration = Duration::from_secs(30);

/// How long until a closed connection gets cleaned up, preventing the closed message from being retrieved.
pub const CLOSED_CONNECTION_EXPIRY: Duration = Duration::from_secs(15);
