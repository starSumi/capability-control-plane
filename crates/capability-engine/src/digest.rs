use crate::{Denial, ErrorCode};
use capability_protocol::{Capability, CapabilityCatalog, CapabilityPolicy, PolicyRef};
use serde::Serialize;

pub(crate) fn digest_value<T: Serialize>(value: &T) -> Result<String, Denial> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        Denial::new(
            ErrorCode::InvalidDigest,
            format!("could not serialize resource for digest: {error}"),
        )
    })?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

/// Compute the deterministic descriptor digest used by observed state.
///
/// # Errors
///
/// Returns [`Denial`] if the versioned resource cannot be serialized.
pub fn capability_digest(capability: &Capability) -> Result<String, Denial> {
    digest_value(capability)
}

/// Compute an order-independent digest of catalog identity and descriptors.
///
/// # Errors
///
/// Returns [`Denial`] if a versioned resource cannot be serialized.
pub fn catalog_digest(catalog: &CapabilityCatalog) -> Result<String, Denial> {
    let mut items = catalog
        .items
        .iter()
        .map(|item| capability_digest(item).map(|digest| (&item.metadata.name, digest)))
        .collect::<Result<Vec<_>, _>>()?;
    items.sort_by(|left, right| left.0.cmp(right.0));
    digest_value(&(
        catalog.api_version,
        catalog.kind,
        catalog.resource_version,
        items,
    ))
}

/// Compute the deterministic digest and identity carried by every decision.
///
/// # Errors
///
/// Returns [`Denial`] if the policy cannot be serialized.
pub fn policy_ref(policy: &CapabilityPolicy) -> Result<PolicyRef, Denial> {
    Ok(PolicyRef {
        name: policy.metadata.name.clone(),
        generation: policy.metadata.generation,
        digest: digest_value(policy)?,
    })
}
