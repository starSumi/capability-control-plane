mod store;

use capability_engine::{
    Denial, RouteConfig, RoutingIndex, ShadowFixture, admit, catalog_digest, plan_reconcile,
    policy_ref, replay_shadow, validate_policy,
};
use capability_protocol::{
    ApiVersion, COMPILED_LIMITS, CapabilityCatalog, CapabilityPolicy, ErrorCode, ErrorReport,
    ErrorReportKind, ObservedCatalog, ProtocolDocumentKind, ReconcilePlan, RouteResult,
    ShadowEvaluation, ShadowObservation, ValidationReport, ValidationReportKind,
};
use clap::{Parser, Subcommand};
use schemars::schema_for;
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use store::{RootedJsonStore, StoreError};
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(
    name = "capctl",
    version,
    about = "Fail-closed capability discovery and reconcile planner"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate a desired catalog under an admission policy.
    Validate {
        /// Root directory containing all input documents.
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Root-relative desired catalog JSON path.
        #[arg(long)]
        catalog: PathBuf,
        /// Root-relative policy JSON path.
        #[arg(long)]
        policy: PathBuf,
    },
    /// Route an intent against an admitted catalog without loading capabilities.
    Route {
        /// Root directory containing all input documents.
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Root-relative desired catalog JSON path.
        #[arg(long)]
        catalog: PathBuf,
        /// Root-relative policy JSON path.
        #[arg(long)]
        policy: PathBuf,
        /// Untrusted intent text. Output does not echo it or a stable digest.
        #[arg(long)]
        query: String,
        /// Maximum number of candidates to return.
        #[arg(long, default_value_t = 5)]
        top_k: u32,
    },
    /// Compute a read-only plan from desired and observed JSON resources.
    Reconcile {
        /// Root directory containing all input documents.
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Root-relative desired catalog JSON path.
        #[arg(long)]
        desired: PathBuf,
        /// Root-relative observed snapshot JSON path.
        #[arg(long)]
        observed: PathBuf,
        /// Root-relative policy JSON path.
        #[arg(long)]
        policy: PathBuf,
    },
    /// Replay a provenance-backed held-out route fixture without mutation.
    Shadow {
        /// Root directory containing all input documents.
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Root-relative held-out fixture JSON path. Queries are read only here
        /// and are absent from the structured replay output.
        #[arg(long)]
        fixture: PathBuf,
        /// Root-relative desired catalog JSON path.
        #[arg(long)]
        catalog: PathBuf,
        /// Root-relative policy JSON path.
        #[arg(long)]
        policy: PathBuf,
    },
    /// Emit the JSON Schema for one versioned resource kind.
    Schema {
        /// Resource kind to describe.
        kind: ProtocolDocumentKind,
    },
}

#[derive(Debug, Error)]
enum AppError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Denied(#[from] Denial),
    #[error("could not serialize structured output: {0}")]
    Output(serde_json::Error),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match execute(cli) {
        Ok(value) => match write_json(&value) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                write_error(&error);
                ExitCode::from(2)
            }
        },
        Err(error) => {
            write_error(&error);
            ExitCode::from(2)
        }
    }
}

fn execute(cli: Cli) -> Result<Value, AppError> {
    match cli.command {
        Command::Validate {
            root,
            catalog,
            policy,
        } => {
            let (catalog, policy) = load_desired(&root, &catalog, &policy)?;
            admit(&catalog, &policy)?;
            to_value(&ValidationReport {
                api_version: ApiVersion::V1Alpha1,
                kind: ValidationReportKind::ValidationReport,
                ok: true,
                resource_version: catalog.resource_version,
                capability_count: u32::try_from(catalog.items.len()).map_err(|_| {
                    Denial::new(
                        ErrorCode::ResourceLimitExceeded,
                        "capability count overflow",
                    )
                })?,
                catalog_digest: catalog_digest(&catalog)?,
                policy: policy_ref(&policy)?,
            })
        }
        Command::Route {
            root,
            catalog,
            policy,
            query,
            top_k,
        } => {
            let (catalog, policy) = load_desired(&root, &catalog, &policy)?;
            let index = RoutingIndex::build(&catalog, &policy)?;
            let result = index.route(
                &query,
                RouteConfig {
                    top_k,
                    ..RouteConfig::default()
                },
            )?;
            to_value(&result)
        }
        Command::Reconcile {
            root,
            desired,
            observed,
            policy,
        } => {
            let (desired, policy) = load_desired(&root, &desired, &policy)?;
            let store = RootedJsonStore::new(
                &root,
                usize::try_from(policy.spec.limits.max_document_bytes).map_err(|_| {
                    Denial::new(
                        ErrorCode::InvalidPolicy,
                        "document limit is not representable",
                    )
                })?,
            )?;
            let observed: ObservedCatalog = store.read(&observed)?;
            to_value(&plan_reconcile(&desired, &observed, &policy)?)
        }
        Command::Schema { kind } => {
            let schema = match kind {
                ProtocolDocumentKind::Catalog => to_value(&schema_for!(CapabilityCatalog))?,
                ProtocolDocumentKind::Policy => to_value(&schema_for!(CapabilityPolicy))?,
                ProtocolDocumentKind::Observed => to_value(&schema_for!(ObservedCatalog))?,
                ProtocolDocumentKind::Route => to_value(&schema_for!(RouteResult))?,
                ProtocolDocumentKind::Reconcile => to_value(&schema_for!(ReconcilePlan))?,
                ProtocolDocumentKind::Validation => to_value(&schema_for!(ValidationReport))?,
                ProtocolDocumentKind::Error => to_value(&schema_for!(ErrorReport))?,
                ProtocolDocumentKind::ShadowObservation => {
                    to_value(&schema_for!(ShadowObservation))?
                }
                ProtocolDocumentKind::ShadowEvaluation => to_value(&schema_for!(ShadowEvaluation))?,
            };
            Ok(schema)
        }
        Command::Shadow {
            root,
            fixture,
            catalog,
            policy,
        } => {
            let (catalog, policy) = load_desired(&root, &catalog, &policy)?;
            let store = RootedJsonStore::new(
                &root,
                usize::try_from(policy.spec.limits.max_document_bytes).map_err(|_| {
                    Denial::new(
                        ErrorCode::InvalidPolicy,
                        "document limit is not representable",
                    )
                })?,
            )?;
            let fixture: ShadowFixture = store.read(&fixture)?;
            to_value(&replay_shadow(&fixture, &catalog, &policy)?)
        }
    }
}

fn load_desired(
    root: &Path,
    catalog_path: &Path,
    policy_path: &Path,
) -> Result<(CapabilityCatalog, CapabilityPolicy), AppError> {
    let compiled_document_limit =
        usize::try_from(COMPILED_LIMITS.max_document_bytes).map_err(|_| {
            Denial::new(
                ErrorCode::InvalidPolicy,
                "compiled document limit is not representable",
            )
        })?;
    let bootstrap = RootedJsonStore::new(root, compiled_document_limit)?;
    let policy: CapabilityPolicy = bootstrap.read(policy_path)?;
    validate_policy(&policy)?;
    let store = RootedJsonStore::new(
        root,
        usize::try_from(policy.spec.limits.max_document_bytes).map_err(|_| {
            Denial::new(
                ErrorCode::InvalidPolicy,
                "document limit is not representable",
            )
        })?,
    )?;
    let catalog = store.read(catalog_path)?;
    admit(&catalog, &policy)?;
    Ok((catalog, policy))
}

fn to_value(value: &impl Serialize) -> Result<Value, AppError> {
    serde_json::to_value(value).map_err(AppError::Output)
}

fn write_json(value: &Value) -> Result<(), AppError> {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    serde_json::to_writer_pretty(&mut lock, value).map_err(AppError::Output)
}

fn write_error(error: &AppError) {
    let (code, message) = match error {
        AppError::Store(inner) => (inner.code(), inner.to_string()),
        AppError::Denied(inner) => (inner.code, inner.to_string()),
        AppError::Output(_) => (ErrorCode::OutputError, error.to_string()),
    };
    let report = ErrorReport {
        api_version: ApiVersion::V1Alpha1,
        kind: ErrorReportKind::ErrorReport,
        ok: false,
        code,
        message: bounded_message(message),
    };
    let stderr = std::io::stderr();
    let mut lock = stderr.lock();
    let _ignored = serde_json::to_writer_pretty(&mut lock, &report);
}

fn bounded_message(mut message: String) -> String {
    const MAX_BYTES: usize = 4_096;
    if message.len() <= MAX_BYTES {
        return message;
    }
    let mut boundary = MAX_BYTES;
    while !message.is_char_boundary(boundary) {
        boundary -= 1;
    }
    message.truncate(boundary);
    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn invalid_policy_is_rejected_before_other_reconcile_reads()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        fs::write(
            directory.path().join("policy.json"),
            r#"{
                "apiVersion":"capctl.dev/v1alpha1",
                "kind":"CapabilityPolicy",
                "metadata":{"name":"test-policy","generation":1},
                "spec":{
                    "allowedAuthorities":["fixture"],
                    "allowedPermissions":["readMetadata"],
                    "limits":{
                        "maxCatalogItems":1,"maxQueryBytes":1,"maxQueryTerms":1,
                        "maxFieldBytes":1,"maxTriggersPerCapability":1,
                        "maxTagsPerCapability":1,"maxLabelsPerCapability":1,
                        "maxTermsPerCapability":1,"maxCatalogTerms":1,
                        "maxCatalogTextBytes":1,"maxTopK":1,
                        "maxDocumentBytes":8388609
                    }
                }
            }"#,
        )?;
        let result = execute(Cli {
            command: Command::Reconcile {
                root: directory.path().to_path_buf(),
                desired: PathBuf::from("missing-desired.json"),
                observed: PathBuf::from("missing-observed.json"),
                policy: PathBuf::from("policy.json"),
            },
        });
        assert!(matches!(
            result,
            Err(AppError::Denied(Denial {
                code: ErrorCode::InvalidPolicy,
                ..
            }))
        ));
        Ok(())
    }
}
