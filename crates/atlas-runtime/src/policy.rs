//! Capability policy enforcement.
//!
//! Every `SourceInstance` holds a `CapabilityPolicy` derived from the source's
//! manifest. Before executing any host import, the runtime checks the policy.

use std::collections::HashSet;
use atlas_spec::capability::Capability;
use crate::error::RuntimeError;

/// Enforces that a source only calls host imports it has declared in its manifest.
#[derive(Debug, Clone)]
pub struct CapabilityPolicy {
    granted: HashSet<String>,
}

impl CapabilityPolicy {
    /// Build a policy from the capabilities declared in a source manifest.
    pub fn from_capabilities(caps: &[Capability]) -> Self {
        let granted = caps.iter().map(|c| c.identifier().to_string()).collect();
        Self { granted }
    }

    /// Returns `Ok(())` if the capability is granted, or a `CapabilityDenied` error.
    pub fn check(&self, capability: &Capability) -> Result<(), RuntimeError> {
        let id = capability.identifier();
        if self.granted.contains(id) {
            Ok(())
        } else {
            Err(RuntimeError::CapabilityDenied {
                capability: id.to_string(),
            })
        }
    }

    /// Returns `true` if the capability is granted.
    pub fn is_granted(&self, capability: &Capability) -> bool {
        self.granted.contains(capability.identifier())
    }
}
