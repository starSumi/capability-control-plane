//! Versioned, side-effect-free protocol and structural semantics.
//!
//! This crate is the semantic source for generated client contracts. It owns
//! wire widths, resource kinds, compiled ceilings, stable error codes, and the
//! structural validation shared by every Rust consumer. It contains no
//! filesystem, network, process, or tool execution code.

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;
use ts_rs::TS;

pub const IDENTIFIER_MAX_BYTES: usize = 128;
pub const DIGEST_HEX_BYTES: usize = 64;

/// Absolute ceilings compiled into every protocol consumer.
pub const COMPILED_LIMITS: ResourceLimits = ResourceLimits {
    max_catalog_items: 10_000,
    max_query_bytes: 16 * 1024,
    max_query_terms: 4_096,
    max_field_bytes: 64 * 1024,
    max_triggers_per_capability: 512,
    max_tags_per_capability: 256,
    max_labels_per_capability: 128,
    max_terms_per_capability: 4_096,
    max_catalog_terms: 2_000_000,
    max_catalog_text_bytes: 8 * 1024 * 1024,
    max_top_k: 100,
    max_document_bytes: 8 * 1024 * 1024,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
pub enum ApiVersion {
    #[serde(rename = "capctl.dev/v1alpha1")]
    V1Alpha1,
}

macro_rules! kind {
    ($name:ident, $variant:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
        pub enum $name {
            $variant,
        }
    };
}

kind!(CatalogKind, CapabilityCatalog);
kind!(PolicyKind, CapabilityPolicy);
kind!(ObservedKind, ObservedCatalog);
kind!(RouteResultKind, RouteResult);
kind!(ReconcilePlanKind, ReconcilePlan);
kind!(ValidationReportKind, ValidationReport);
kind!(ErrorReportKind, ErrorReport);

macro_rules! protocol_documents {
    ($($variant:ident => ($cli:literal, $schema_path:literal, $root:ty)),+ $(,)?) => {
        /// Generated root documents. This declaration is the registry used by
        /// CLI parsing, schema paths, artifact enumeration, and TypeScript roots.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum ProtocolDocumentKind {
            $($variant),+
        }

        impl ProtocolDocumentKind {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $cli),+
                }
            }

            #[must_use]
            pub const fn schema_path(self) -> &'static str {
                match self {
                    $(Self::$variant => $schema_path),+
                }
            }

            #[must_use]
            pub fn typescript_root_declaration(self, config: &ts_rs::Config) -> String {
                match self {
                    $(Self::$variant => export_typescript_decl::<$root>(config)),+
                }
            }
        }
    };
}

protocol_documents! {
    Catalog => (
        "catalog",
        "generated/schemas/v1alpha1/capability-catalog.schema.json",
        CapabilityCatalog
    ),
    Policy => (
        "policy",
        "generated/schemas/v1alpha1/capability-policy.schema.json",
        CapabilityPolicy
    ),
    Observed => (
        "observed",
        "generated/schemas/v1alpha1/observed-catalog.schema.json",
        ObservedCatalog
    ),
    Route => (
        "route",
        "generated/schemas/v1alpha1/route-result.schema.json",
        RouteResult
    ),
    Reconcile => (
        "reconcile",
        "generated/schemas/v1alpha1/reconcile-plan.schema.json",
        ReconcilePlan
    ),
    Validation => (
        "validation",
        "generated/schemas/v1alpha1/validation-report.schema.json",
        ValidationReport
    ),
    Error => (
        "error",
        "generated/schemas/v1alpha1/error-report.schema.json",
        ErrorReport
    ),
}

fn export_typescript_decl<T: TS>(config: &ts_rs::Config) -> String {
    let mut output = T::docs().unwrap_or_default();
    output.push_str("export ");
    output.push_str(&T::decl(config));
    output
}

impl FromStr for ProtocolDocumentKind {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|kind| kind.as_str() == value)
            .ok_or("expected catalog, policy, observed, route, reconcile, validation, or error")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityCatalog {
    pub api_version: ApiVersion,
    pub kind: CatalogKind,
    #[schemars(range(min = 1))]
    pub resource_version: u32,
    #[schemars(length(max = 10_000))]
    pub items: Vec<Capability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Capability {
    pub metadata: CapabilityMetadata,
    pub spec: CapabilitySpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityMetadata {
    #[schemars(
        length(min = 1, max = 128),
        regex(
            pattern = "^[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?(?:/[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?)*$"
        )
    )]
    pub name: String,
    #[schemars(range(min = 1))]
    pub generation: u32,
    #[serde(default)]
    #[schemars(schema_with = "labels_schema")]
    pub labels: BTreeMap<String, String>,
}

fn labels_schema(_: &mut SchemaGenerator) -> Schema {
    schemars::json_schema!({
        "type": "object",
        "maxProperties": 128,
        "propertyNames": {
            "type": "string",
            "minLength": 1,
            "maxLength": 65536
        },
        "additionalProperties": {
            "type": "string",
            "minLength": 1,
            "maxLength": 65536
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilitySpec {
    #[schemars(length(min = 1, max = 65_536))]
    pub description: String,
    #[serde(default)]
    #[schemars(length(max = 512), inner(length(min = 1, max = 65_536)))]
    pub triggers: Vec<String>,
    #[serde(default)]
    #[schemars(length(max = 256), inner(length(min = 1, max = 65_536)))]
    pub tags: Vec<String>,
    pub authority: AuthorityRef,
    pub load: LoadSpec,
    #[serde(default)]
    pub permissions: BTreeSet<Permission>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthorityRef {
    #[schemars(
        length(min = 1, max = 128),
        regex(
            pattern = "^[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?(?:/[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?)*$"
        )
    )]
    pub provider: String,
    #[schemars(
        length(min = 1, max = 128),
        regex(
            pattern = "^[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?(?:/[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?)*$"
        )
    )]
    pub package: String,
    #[schemars(
        length(min = 1, max = 128),
        regex(
            pattern = "^[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?(?:/[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?)*$"
        )
    )]
    pub resource: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoadSpec {
    pub mode: LoadMode,
    pub estimated_tokens: u32,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema, TS,
)]
#[serde(rename_all = "camelCase")]
pub enum LoadMode {
    MetadataOnly,
    ThinRouter,
    FullWorkflow,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema, TS,
)]
#[serde(rename_all = "camelCase")]
pub enum Permission {
    ReadMetadata,
    ReadContent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityPolicy {
    pub api_version: ApiVersion,
    pub kind: PolicyKind,
    pub metadata: PolicyMetadata,
    pub spec: PolicySpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicyMetadata {
    #[schemars(
        length(min = 1, max = 128),
        regex(
            pattern = "^[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?(?:/[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?)*$"
        )
    )]
    pub name: String,
    #[schemars(range(min = 1))]
    pub generation: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicySpec {
    #[schemars(
        length(min = 1, max = 10_000),
        inner(
            length(min = 1, max = 128),
            regex(
                pattern = "^[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?(?:/[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?)*$"
            )
        )
    )]
    pub allowed_authorities: BTreeSet<String>,
    pub allowed_permissions: BTreeSet<Permission>,
    pub limits: ResourceLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceLimits {
    #[schemars(range(min = 1, max = 10_000))]
    pub max_catalog_items: u32,
    #[schemars(range(min = 1, max = 16_384))]
    pub max_query_bytes: u32,
    #[schemars(range(min = 1, max = 4_096))]
    pub max_query_terms: u32,
    #[schemars(range(min = 1, max = 65_536))]
    pub max_field_bytes: u32,
    #[schemars(range(min = 1, max = 512))]
    pub max_triggers_per_capability: u32,
    #[schemars(range(min = 1, max = 256))]
    pub max_tags_per_capability: u32,
    #[schemars(range(min = 1, max = 128))]
    pub max_labels_per_capability: u32,
    #[schemars(range(min = 1, max = 4_096))]
    pub max_terms_per_capability: u32,
    #[schemars(range(min = 1, max = 2_000_000))]
    pub max_catalog_terms: u32,
    #[schemars(range(min = 1, max = 8_388_608))]
    pub max_catalog_text_bytes: u32,
    #[schemars(range(min = 1, max = 100))]
    pub max_top_k: u32,
    #[schemars(range(min = 1, max = 8_388_608))]
    pub max_document_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObservedCatalog {
    pub api_version: ApiVersion,
    pub kind: ObservedKind,
    pub observed_resource_version: u32,
    #[schemars(length(max = 10_000))]
    pub items: Vec<ObservedCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObservedCapability {
    #[schemars(
        length(min = 1, max = 128),
        regex(
            pattern = "^[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?(?:/[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?)*$"
        )
    )]
    pub name: String,
    #[schemars(range(min = 1))]
    pub generation: u32,
    #[schemars(length(equal = 64), regex(pattern = "^[0-9a-f]{64}$"))]
    pub digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidPolicy,
    InvalidGeneration,
    InvalidIdentifier,
    DuplicateCapability,
    AuthorityDenied,
    PermissionDenied,
    ResourceLimitExceeded,
    InvalidQuery,
    StaleGeneration,
    InvalidDigest,
    InvalidRoot,
    InvalidPath,
    IndirectPath,
    DocumentTooLarge,
    IoError,
    InvalidJson,
    OutputError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolViolation {
    pub code: ErrorCode,
    pub message: String,
}

impl ProtocolViolation {
    fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for ProtocolViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for ProtocolViolation {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum RouteDecision {
    Selected,
    Ambiguous,
    NoMatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicyRef {
    #[schemars(
        length(min = 1, max = 128),
        regex(
            pattern = "^[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?(?:/[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?)*$"
        )
    )]
    pub name: String,
    #[schemars(range(min = 1))]
    pub generation: u32,
    #[schemars(length(equal = 64), regex(pattern = "^[0-9a-f]{64}$"))]
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectorRef {
    #[schemars(length(min = 1, max = 128))]
    pub name: String,
    #[schemars(length(min = 1, max = 64))]
    pub version: String,
    #[schemars(length(equal = 64), regex(pattern = "^[0-9a-f]{64}$"))]
    pub config_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RouteMatch {
    #[schemars(
        length(min = 1, max = 128),
        regex(
            pattern = "^[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?(?:/[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?)*$"
        )
    )]
    pub capability_id: String,
    /// Quantized score in millionths; this is ordering evidence, not probability.
    pub score_micros: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RouteResult {
    pub api_version: ApiVersion,
    pub kind: RouteResultKind,
    #[schemars(range(min = 1))]
    pub resource_version: u32,
    #[schemars(length(equal = 64), regex(pattern = "^[0-9a-f]{64}$"))]
    pub catalog_digest: String,
    pub policy: PolicyRef,
    pub selector: SelectorRef,
    pub decision: RouteDecision,
    #[schemars(length(max = 100))]
    pub matches: Vec<RouteMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ReconcileAction {
    /// Create is guarded by the absence of the capability in observed state.
    Create {
        #[schemars(
            length(min = 1, max = 128),
            regex(
                pattern = "^[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?(?:/[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?)*$"
            )
        )]
        capability_id: String,
        #[schemars(range(min = 1))]
        desired_generation: u32,
        #[schemars(length(equal = 64), regex(pattern = "^[0-9a-f]{64}$"))]
        desired_digest: String,
    },
    /// Update carries the complete compare-and-swap precondition.
    Update {
        #[schemars(
            length(min = 1, max = 128),
            regex(
                pattern = "^[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?(?:/[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?)*$"
            )
        )]
        capability_id: String,
        #[schemars(range(min = 1))]
        desired_generation: u32,
        #[schemars(range(min = 1))]
        expected_observed_generation: u32,
        #[schemars(length(equal = 64), regex(pattern = "^[0-9a-f]{64}$"))]
        expected_observed_digest: String,
        #[schemars(length(equal = 64), regex(pattern = "^[0-9a-f]{64}$"))]
        desired_digest: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReconcilePlan {
    pub api_version: ApiVersion,
    pub kind: ReconcilePlanKind,
    #[schemars(range(min = 1))]
    pub desired_resource_version: u32,
    pub observed_resource_version: u32,
    #[schemars(length(equal = 64), regex(pattern = "^[0-9a-f]{64}$"))]
    pub desired_digest: String,
    pub policy: PolicyRef,
    #[schemars(length(max = 10_000))]
    pub actions: Vec<ReconcileAction>,
    #[schemars(length(max = 10_000))]
    pub retained_orphans: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ValidationReport {
    pub api_version: ApiVersion,
    pub kind: ValidationReportKind,
    pub ok: bool,
    #[schemars(range(min = 1))]
    pub resource_version: u32,
    #[schemars(range(max = 10_000))]
    pub capability_count: u32,
    #[schemars(length(equal = 64), regex(pattern = "^[0-9a-f]{64}$"))]
    pub catalog_digest: String,
    pub policy: PolicyRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ErrorReport {
    pub api_version: ApiVersion,
    pub kind: ErrorReportKind,
    pub ok: bool,
    pub code: ErrorCode,
    #[schemars(length(max = 4_096))]
    pub message: String,
}

/// Validate policy structure and compiled ceilings before a consumer uses any
/// policy-controlled value for allocation or I/O.
///
/// # Errors
///
/// Returns [`ProtocolViolation`] for malformed identity, generation, authority,
/// or a limit outside the compiled envelope.
pub fn validate_policy_semantics(policy: &CapabilityPolicy) -> Result<(), ProtocolViolation> {
    validate_identifier(&policy.metadata.name)?;
    require_generation("policy", policy.metadata.generation)?;
    if policy.spec.allowed_authorities.is_empty() {
        return Err(ProtocolViolation::new(
            ErrorCode::InvalidPolicy,
            "policy must admit at least one authority",
        ));
    }
    for authority in &policy.spec.allowed_authorities {
        validate_identifier(authority)?;
    }
    validate_limits(&policy.spec.limits)
}

/// Validate compiled structural bounds independent of a selected policy.
///
/// # Errors
///
/// Returns [`ProtocolViolation`] for invalid identity, generation, duplicate
/// resources, or a collection/text ceiling breach.
pub fn validate_catalog_semantics(catalog: &CapabilityCatalog) -> Result<(), ProtocolViolation> {
    require_generation("catalog resourceVersion", catalog.resource_version)?;
    if catalog.items.len() > usize_from(COMPILED_LIMITS.max_catalog_items) {
        return Err(resource_limit(
            "catalog item count exceeds compiled ceiling",
        ));
    }
    let mut names = BTreeSet::new();
    let mut text_bytes = 0_usize;
    for capability in &catalog.items {
        require_generation(&capability.metadata.name, capability.metadata.generation)?;
        validate_identifier(&capability.metadata.name)?;
        validate_identifier(&capability.spec.authority.provider)?;
        validate_identifier(&capability.spec.authority.package)?;
        validate_identifier(&capability.spec.authority.resource)?;
        if !names.insert(&capability.metadata.name) {
            return Err(ProtocolViolation::new(
                ErrorCode::DuplicateCapability,
                format!("duplicate capability: {}", capability.metadata.name),
            ));
        }
        if capability.metadata.labels.len() > usize_from(COMPILED_LIMITS.max_labels_per_capability)
            || capability.spec.triggers.len()
                > usize_from(COMPILED_LIMITS.max_triggers_per_capability)
            || capability.spec.tags.len() > usize_from(COMPILED_LIMITS.max_tags_per_capability)
        {
            return Err(resource_limit(format!(
                "{} exceeds a compiled collection ceiling",
                capability.metadata.name
            )));
        }
        for value in descriptor_fields(capability) {
            validate_field(value)?;
            text_bytes = text_bytes
                .checked_add(value.len())
                .ok_or_else(|| resource_limit("catalog text byte count overflow"))?;
        }
    }
    if text_bytes > usize_from(COMPILED_LIMITS.max_catalog_text_bytes) {
        return Err(resource_limit("catalog text exceeds compiled ceiling"));
    }
    Ok(())
}

/// Validate a rebuildable observation before planning.
///
/// # Errors
///
/// Returns [`ProtocolViolation`] for invalid identity, generation, digest,
/// duplicate resource, or a compiled item ceiling breach.
pub fn validate_observed_semantics(observed: &ObservedCatalog) -> Result<(), ProtocolViolation> {
    if observed.items.len() > usize_from(COMPILED_LIMITS.max_catalog_items) {
        return Err(resource_limit(
            "observed item count exceeds compiled ceiling",
        ));
    }
    let mut names = BTreeSet::new();
    for item in &observed.items {
        validate_identifier(&item.name)?;
        require_generation(&item.name, item.generation)?;
        if !is_blake3_digest(&item.digest) {
            return Err(ProtocolViolation::new(
                ErrorCode::InvalidDigest,
                format!("invalid observed digest: {}", item.name),
            ));
        }
        if !names.insert(&item.name) {
            return Err(ProtocolViolation::new(
                ErrorCode::DuplicateCapability,
                format!("duplicate observed capability: {}", item.name),
            ));
        }
    }
    Ok(())
}

/// Validate the canonical ASCII identifier grammar.
///
/// # Errors
///
/// Returns [`ProtocolViolation`] when the value is empty, too long, non-ASCII,
/// or contains an invalid segment.
pub fn validate_identifier(value: &str) -> Result<(), ProtocolViolation> {
    if value.is_empty() || value.len() > IDENTIFIER_MAX_BYTES || !value.is_ascii() {
        return Err(ProtocolViolation::new(
            ErrorCode::InvalidIdentifier,
            "identifiers must be 1..=128 ASCII bytes",
        ));
    }
    for segment in value.split('/') {
        let bytes = segment.as_bytes();
        let (Some(first), Some(last)) = (bytes.first(), bytes.last()) else {
            return Err(ProtocolViolation::new(
                ErrorCode::InvalidIdentifier,
                format!("identifier has an empty segment: {value}"),
            ));
        };
        if (!first.is_ascii_lowercase() && !first.is_ascii_digit())
            || (!last.is_ascii_lowercase() && !last.is_ascii_digit())
            || !bytes.iter().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'-' | b'_' | b'.')
            })
        {
            return Err(ProtocolViolation::new(
                ErrorCode::InvalidIdentifier,
                format!("malformed identifier: {value}"),
            ));
        }
    }
    Ok(())
}

#[must_use]
pub fn is_blake3_digest(value: &str) -> bool {
    value.len() == DIGEST_HEX_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_limits(limits: &ResourceLimits) -> Result<(), ProtocolViolation> {
    let checks = [
        (
            "maxCatalogItems",
            limits.max_catalog_items,
            COMPILED_LIMITS.max_catalog_items,
        ),
        (
            "maxQueryBytes",
            limits.max_query_bytes,
            COMPILED_LIMITS.max_query_bytes,
        ),
        (
            "maxQueryTerms",
            limits.max_query_terms,
            COMPILED_LIMITS.max_query_terms,
        ),
        (
            "maxFieldBytes",
            limits.max_field_bytes,
            COMPILED_LIMITS.max_field_bytes,
        ),
        (
            "maxTriggersPerCapability",
            limits.max_triggers_per_capability,
            COMPILED_LIMITS.max_triggers_per_capability,
        ),
        (
            "maxTagsPerCapability",
            limits.max_tags_per_capability,
            COMPILED_LIMITS.max_tags_per_capability,
        ),
        (
            "maxLabelsPerCapability",
            limits.max_labels_per_capability,
            COMPILED_LIMITS.max_labels_per_capability,
        ),
        (
            "maxTermsPerCapability",
            limits.max_terms_per_capability,
            COMPILED_LIMITS.max_terms_per_capability,
        ),
        (
            "maxCatalogTerms",
            limits.max_catalog_terms,
            COMPILED_LIMITS.max_catalog_terms,
        ),
        (
            "maxCatalogTextBytes",
            limits.max_catalog_text_bytes,
            COMPILED_LIMITS.max_catalog_text_bytes,
        ),
        ("maxTopK", limits.max_top_k, COMPILED_LIMITS.max_top_k),
        (
            "maxDocumentBytes",
            limits.max_document_bytes,
            COMPILED_LIMITS.max_document_bytes,
        ),
    ];
    for (name, value, ceiling) in checks {
        if value == 0 || value > ceiling {
            return Err(ProtocolViolation::new(
                ErrorCode::InvalidPolicy,
                format!("{name} must be within 1..={ceiling}"),
            ));
        }
    }
    Ok(())
}

fn require_generation(name: &str, generation: u32) -> Result<(), ProtocolViolation> {
    if generation == 0 {
        return Err(ProtocolViolation::new(
            ErrorCode::InvalidGeneration,
            format!("{name} must be greater than zero"),
        ));
    }
    Ok(())
}

fn validate_field(value: &str) -> Result<(), ProtocolViolation> {
    if value.is_empty() || value.len() > usize_from(COMPILED_LIMITS.max_field_bytes) {
        return Err(resource_limit(format!(
            "descriptor fields must contain 1..={} UTF-8 bytes",
            COMPILED_LIMITS.max_field_bytes
        )));
    }
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

fn resource_limit(message: impl Into<String>) -> ProtocolViolation {
    ProtocolViolation::new(ErrorCode::ResourceLimitExceeded, message)
}

const fn usize_from(value: u32) -> usize {
    value as usize
}

#[cfg(test)]
mod tests {
    use super::{CapabilityCatalog, CapabilityPolicy, ReconcilePlan};
    use schemars::schema_for;

    #[test]
    fn structural_schema_projection_keeps_critical_constraints() -> Result<(), serde_json::Error> {
        let catalog = serde_json::to_value(schema_for!(CapabilityCatalog))?;
        assert_eq!(
            catalog
                .pointer("/$defs/CapabilityMetadata/properties/labels/maxProperties")
                .and_then(serde_json::Value::as_u64),
            Some(128)
        );
        assert_eq!(
            catalog
                .pointer("/$defs/CapabilitySpec/properties/triggers/items/maxLength")
                .and_then(serde_json::Value::as_u64),
            Some(65_536)
        );
        assert_eq!(
            catalog
                .pointer("/$defs/CapabilitySpec/properties/triggers/items/minLength")
                .and_then(serde_json::Value::as_u64),
            Some(1)
        );
        assert_eq!(
            catalog
                .pointer("/$defs/CapabilitySpec/properties/tags/items/minLength")
                .and_then(serde_json::Value::as_u64),
            Some(1)
        );
        assert_eq!(
            catalog
                .pointer("/$defs/CapabilityMetadata/properties/labels/propertyNames/minLength")
                .and_then(serde_json::Value::as_u64),
            Some(1)
        );
        assert_eq!(
            catalog
                .pointer("/$defs/CapabilityMetadata/properties/labels/propertyNames/maxLength")
                .and_then(serde_json::Value::as_u64),
            Some(65_536)
        );
        assert_eq!(
            catalog
                .pointer(
                    "/$defs/CapabilityMetadata/properties/labels/additionalProperties/minLength"
                )
                .and_then(serde_json::Value::as_u64),
            Some(1)
        );

        let policy = serde_json::to_value(schema_for!(CapabilityPolicy))?;
        assert!(
            policy
                .pointer("/$defs/PolicySpec/properties/allowedAuthorities/items/pattern")
                .and_then(serde_json::Value::as_str)
                .is_some()
        );

        let reconcile = serde_json::to_value(schema_for!(ReconcilePlan))?;
        assert_eq!(
            reconcile
                .pointer("/$defs/ReconcileAction/oneOf")
                .and_then(serde_json::Value::as_array)
                .map(Vec::len),
            Some(2)
        );
        assert!(
            reconcile
                .pointer("/$defs/ReconcileAction/oneOf/1/required")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|required| {
                    required
                        .iter()
                        .any(|value| value.as_str() == Some("expectedObservedGeneration"))
                        && required
                            .iter()
                            .any(|value| value.as_str() == Some("expectedObservedDigest"))
                })
        );
        Ok(())
    }
}
