use crate::normalize::terms;
use capability_protocol::{
    Capability, CapabilityCatalog, CapabilityPolicy, ErrorCode, ProtocolViolation, ResourceLimits,
    validate_catalog_semantics, validate_policy_semantics,
};
use serde::Serialize;
use thiserror::Error;

/// A fail-closed engine decision with a stable code and bounded explanation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Error)]
#[error("{code:?}: {message}")]
pub struct Denial {
    pub code: ErrorCode,
    pub message: String,
}

impl Denial {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl From<ProtocolViolation> for Denial {
    fn from(value: ProtocolViolation) -> Self {
        Self::new(value.code, value.message)
    }
}

/// Validate a policy before any policy-controlled allocation or I/O.
///
/// # Errors
///
/// Returns [`Denial`] when its structure or a requested ceiling is invalid.
pub fn validate_policy(policy: &CapabilityPolicy) -> Result<(), Denial> {
    validate_policy_semantics(policy).map_err(Into::into)
}

/// Admit a desired catalog under a bounded policy.
///
/// # Errors
///
/// Returns [`Denial`] when policy ceilings are invalid or any descriptor is
/// malformed, duplicated, over budget, or requests unadmitted authority.
pub fn admit(catalog: &CapabilityCatalog, policy: &CapabilityPolicy) -> Result<(), Denial> {
    validate_policy(policy)?;
    validate_catalog_semantics(catalog)?;
    let limits = &policy.spec.limits;

    if catalog.items.len() > to_usize(limits.max_catalog_items)? {
        return Err(limit("catalog item count exceeds policy"));
    }

    let mut catalog_terms = 0_usize;
    let mut catalog_text_bytes = 0_usize;
    for capability in &catalog.items {
        validate_capability(
            capability,
            policy,
            &mut catalog_terms,
            &mut catalog_text_bytes,
        )?;
    }
    if catalog_terms > to_usize(limits.max_catalog_terms)? {
        return Err(limit("catalog term count exceeds policy"));
    }
    if catalog_text_bytes > to_usize(limits.max_catalog_text_bytes)? {
        return Err(limit("catalog text byte count exceeds policy"));
    }
    Ok(())
}

fn validate_capability(
    capability: &Capability,
    policy: &CapabilityPolicy,
    catalog_terms: &mut usize,
    catalog_text_bytes: &mut usize,
) -> Result<(), Denial> {
    let limits = &policy.spec.limits;
    if !policy
        .spec
        .allowed_authorities
        .contains(&capability.spec.authority.provider)
    {
        return Err(Denial::new(
            ErrorCode::AuthorityDenied,
            format!(
                "{} requests untrusted authority {}",
                capability.metadata.name, capability.spec.authority.provider
            ),
        ));
    }
    if !capability
        .spec
        .permissions
        .is_subset(&policy.spec.allowed_permissions)
    {
        return Err(Denial::new(
            ErrorCode::PermissionDenied,
            format!("{} requests a denied permission", capability.metadata.name),
        ));
    }
    if capability.spec.triggers.len() > to_usize(limits.max_triggers_per_capability)?
        || capability.spec.tags.len() > to_usize(limits.max_tags_per_capability)?
        || capability.metadata.labels.len() > to_usize(limits.max_labels_per_capability)?
    {
        return Err(limit(format!(
            "{} exceeds a policy collection limit",
            capability.metadata.name
        )));
    }

    let mut capability_terms = 0_usize;
    for value in descriptor_fields(capability) {
        validate_field(value, limits)?;
        capability_terms = capability_terms
            .checked_add(terms(value).len())
            .ok_or_else(|| limit("capability term count overflow"))?;
        *catalog_text_bytes = catalog_text_bytes
            .checked_add(value.len())
            .ok_or_else(|| limit("catalog text byte count overflow"))?;
    }
    if capability_terms > to_usize(limits.max_terms_per_capability)? {
        return Err(limit(format!(
            "{} exceeds the per-capability term limit",
            capability.metadata.name
        )));
    }
    *catalog_terms = catalog_terms
        .checked_add(capability_terms)
        .ok_or_else(|| limit("catalog term count overflow"))?;
    Ok(())
}

fn descriptor_fields(capability: &Capability) -> Vec<&str> {
    let mut values = Vec::with_capacity(
        5 + capability.spec.triggers.len()
            + capability.spec.tags.len()
            + capability.metadata.labels.len() * 2,
    );
    values.push(capability.metadata.name.as_str());
    values.push(capability.spec.description.as_str());
    values.push(capability.spec.authority.provider.as_str());
    values.push(capability.spec.authority.package.as_str());
    values.push(capability.spec.authority.resource.as_str());
    values.extend(capability.spec.triggers.iter().map(String::as_str));
    values.extend(capability.spec.tags.iter().map(String::as_str));
    for (key, value) in &capability.metadata.labels {
        values.push(key);
        values.push(value);
    }
    values
}

fn validate_field(value: &str, limits: &ResourceLimits) -> Result<(), Denial> {
    if value.is_empty() || value.len() > to_usize(limits.max_field_bytes)? {
        return Err(limit(format!(
            "descriptor fields must contain 1..={} UTF-8 bytes",
            limits.max_field_bytes
        )));
    }
    Ok(())
}

fn to_usize(value: u32) -> Result<usize, Denial> {
    usize::try_from(value).map_err(|_| {
        Denial::new(
            ErrorCode::InvalidPolicy,
            "policy value cannot be represented on this target",
        )
    })
}

fn limit(message: impl Into<String>) -> Denial {
    Denial::new(ErrorCode::ResourceLimitExceeded, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use capability_protocol::{
        ApiVersion, AuthorityRef, COMPILED_LIMITS, CapabilityMetadata, CapabilitySpec, CatalogKind,
        LoadMode, LoadSpec, Permission, PolicyKind, PolicyMetadata, PolicySpec,
    };
    use std::collections::{BTreeMap, BTreeSet};

    fn policy() -> CapabilityPolicy {
        CapabilityPolicy {
            api_version: ApiVersion::V1Alpha1,
            kind: PolicyKind::CapabilityPolicy,
            metadata: PolicyMetadata {
                name: "test-policy".into(),
                generation: 1,
            },
            spec: PolicySpec {
                allowed_authorities: BTreeSet::from(["local".into()]),
                allowed_permissions: BTreeSet::from([Permission::ReadMetadata]),
                limits: ResourceLimits {
                    max_catalog_items: 10,
                    max_query_bytes: 1024,
                    max_query_terms: 128,
                    max_field_bytes: 1024,
                    max_triggers_per_capability: 8,
                    max_tags_per_capability: 8,
                    max_labels_per_capability: 8,
                    max_terms_per_capability: 128,
                    max_catalog_terms: 1024,
                    max_catalog_text_bytes: 65_536,
                    max_top_k: 5,
                    max_document_bytes: 65_536,
                },
            },
        }
    }

    fn capability(provider: &str) -> Capability {
        Capability {
            metadata: CapabilityMetadata {
                name: "local/test".into(),
                generation: 1,
                labels: BTreeMap::new(),
            },
            spec: CapabilitySpec {
                description: "test descriptor".into(),
                triggers: vec!["test".into()],
                tags: vec!["test".into()],
                authority: AuthorityRef {
                    provider: provider.into(),
                    package: "core".into(),
                    resource: "test".into(),
                },
                load: LoadSpec {
                    mode: LoadMode::MetadataOnly,
                    estimated_tokens: 10,
                },
                permissions: BTreeSet::from([Permission::ReadMetadata]),
            },
        }
    }

    #[test]
    fn rejects_unknown_authority() {
        let catalog = CapabilityCatalog {
            api_version: ApiVersion::V1Alpha1,
            kind: CatalogKind::CapabilityCatalog,
            resource_version: 1,
            items: vec![capability("ambient")],
        };
        let error = admit(&catalog, &policy()).err();
        assert_eq!(
            error.map(|value| value.code),
            Some(ErrorCode::AuthorityDenied)
        );
    }

    #[test]
    fn rejects_policy_widening_past_compiled_limit() {
        let mut policy = policy();
        policy.spec.limits.max_document_bytes = COMPILED_LIMITS.max_document_bytes + 1;
        let error = validate_policy(&policy).err();
        assert_eq!(
            error.map(|value| value.code),
            Some(ErrorCode::InvalidPolicy)
        );
    }

    #[test]
    fn rejects_unbounded_tags_and_terms() {
        let mut policy = policy();
        policy.spec.limits.max_tags_per_capability = 1;
        let catalog = CapabilityCatalog {
            api_version: ApiVersion::V1Alpha1,
            kind: CatalogKind::CapabilityCatalog,
            resource_version: 1,
            items: vec![capability("local")],
        };
        let mut catalog = catalog;
        catalog.items[0].spec.tags.push("second".into());
        let error = admit(&catalog, &policy).err();
        assert_eq!(
            error.map(|value| value.code),
            Some(ErrorCode::ResourceLimitExceeded)
        );
    }
}
