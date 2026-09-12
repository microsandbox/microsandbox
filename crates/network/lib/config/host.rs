//! Operator-owned limits read from the runtime process, never from guest env.

use super::MAX_NETWORK_CONNECTIONS;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// Host runtime environment setting for the shared-host TCP connection budget.
pub const HOST_MAX_TCP_CONNECTIONS_ENV: &str = "MSB_HOST_MAX_TCP_CONNECTIONS";

/// Existing shared-host budget, retained unless the operator configures another.
pub const DEFAULT_HOST_MAX_TCP_CONNECTIONS: usize = 256;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Validated per-sandbox connection ceiling owned by a multi-tenant host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostNetworkLimits {
    max_tcp_connections: usize,
}

/// Invalid operator configuration must fail before allocating network resources.
#[derive(Debug, thiserror::Error)]
#[error("{HOST_MAX_TCP_CONNECTIONS_ENV} must be an integer from 1 to {MAX_NETWORK_CONNECTIONS}")]
pub struct HostNetworkLimitsError;

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl HostNetworkLimits {
    /// Validate an operator-owned ceiling against the engine's absolute bound.
    pub fn new(max_tcp_connections: usize) -> Result<Self, HostNetworkLimitsError> {
        if !(1..=MAX_NETWORK_CONNECTIONS).contains(&max_tcp_connections) {
            return Err(HostNetworkLimitsError);
        }
        Ok(Self {
            max_tcp_connections,
        })
    }

    /// Read the host process setting. Guest bootstrap env does not enter this path.
    pub fn from_environment() -> Result<Self, HostNetworkLimitsError> {
        match std::env::var(HOST_MAX_TCP_CONNECTIONS_ENV) {
            Ok(value) => Self::parse(&value),
            Err(std::env::VarError::NotPresent) => Ok(Self::default()),
            Err(std::env::VarError::NotUnicode(_)) => Err(HostNetworkLimitsError),
        }
    }

    /// Maximum tracked TCP connections allowed for one sandbox.
    pub fn max_tcp_connections(self) -> usize {
        self.max_tcp_connections
    }

    fn parse(value: &str) -> Result<Self, HostNetworkLimitsError> {
        let limit = value.parse().map_err(|_| HostNetworkLimitsError)?;
        Self::new(limit)
    }
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

impl Default for HostNetworkLimits {
    fn default() -> Self {
        Self {
            max_tcp_connections: DEFAULT_HOST_MAX_TCP_CONNECTIONS,
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_limits_are_bounded_and_invalid_values_fail_closed() {
        for value in ["0", "4097", "-1", "", "unlimited", "18446744073709551616"] {
            assert!(HostNetworkLimits::parse(value).is_err(), "{value}");
        }
        for limit in [1, 256, 1024, MAX_NETWORK_CONNECTIONS] {
            assert_eq!(
                HostNetworkLimits::new(limit).unwrap().max_tcp_connections(),
                limit
            );
        }
    }
}
