use capability_engine::{
    ErrorCode, RouteConfig, RouteDecision, RoutingIndex, capability_digest, plan_reconcile,
};
use capability_protocol::{
    ApiVersion, AuthorityRef, Capability, CapabilityCatalog, CapabilityMetadata, CapabilityPolicy,
    CapabilitySpec, CatalogKind, LoadMode, LoadSpec, ObservedCapability, ObservedCatalog,
    ObservedKind, Permission, PolicyKind, PolicyMetadata, PolicySpec, ReconcileAction,
    ResourceLimits,
};
use proptest::prelude::*;
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
            allowed_authorities: BTreeSet::from(["fixture".into()]),
            allowed_permissions: BTreeSet::from([
                Permission::ReadMetadata,
                Permission::ReadContent,
            ]),
            limits: ResourceLimits {
                max_catalog_items: 100,
                max_query_bytes: 4096,
                max_query_terms: 512,
                max_field_bytes: 4096,
                max_triggers_per_capability: 32,
                max_tags_per_capability: 32,
                max_labels_per_capability: 32,
                max_terms_per_capability: 512,
                max_catalog_terms: 10_000,
                max_catalog_text_bytes: 1_048_576,
                max_top_k: 10,
                max_document_bytes: 1_048_576,
            },
        },
    }
}

fn capability(name: &str, description: &str, triggers: &[&str], generation: u32) -> Capability {
    Capability {
        metadata: CapabilityMetadata {
            name: name.into(),
            generation,
            labels: BTreeMap::new(),
        },
        spec: CapabilitySpec {
            description: description.into(),
            triggers: triggers.iter().map(ToString::to_string).collect(),
            tags: vec!["fixture".into()],
            authority: AuthorityRef {
                provider: "fixture".into(),
                package: "tests".into(),
                resource: name.replace('/', "-"),
            },
            load: LoadSpec {
                mode: LoadMode::MetadataOnly,
                estimated_tokens: 32,
            },
            permissions: BTreeSet::from([Permission::ReadMetadata]),
        },
    }
}

fn catalog(items: Vec<Capability>) -> CapabilityCatalog {
    CapabilityCatalog {
        api_version: ApiVersion::V1Alpha1,
        kind: CatalogKind::CapabilityCatalog,
        resource_version: 1,
        items,
    }
}

#[test]
fn routes_cjk_intent_without_loading_content() -> Result<(), Box<dyn std::error::Error>> {
    let desired = catalog(vec![
        capability(
            "fixture/session-resume",
            "recover historical context",
            &["恢复会话", "resume session"],
            1,
        ),
        capability(
            "fixture/design-review",
            "review architecture evidence",
            &["架构审查"],
            1,
        ),
    ]);
    let index = RoutingIndex::build(&desired, &policy())?;
    let result = index.route("请恢复会话并接续上下文", RouteConfig::default())?;
    assert_eq!(result.decision, RouteDecision::Selected);
    assert_eq!(
        result
            .matches
            .first()
            .map(|item| item.capability_id.as_str()),
        Some("fixture/session-resume")
    );
    Ok(())
}

#[test]
fn routing_is_stable_under_catalog_order() -> Result<(), Box<dyn std::error::Error>> {
    let left = capability("fixture/left", "alpha recovery", &["alpha"], 1);
    let right = capability("fixture/right", "beta review", &["beta"], 1);
    let first = RoutingIndex::build(&catalog(vec![left.clone(), right.clone()]), &policy())?;
    let second = RoutingIndex::build(&catalog(vec![right, left]), &policy())?;
    assert_eq!(
        first.route("alpha", RouteConfig::default())?,
        second.route("alpha", RouteConfig::default())?
    );
    Ok(())
}

#[test]
fn fails_closed_on_ambiguous_results() -> Result<(), Box<dyn std::error::Error>> {
    let desired = catalog(vec![
        capability("fixture/alpha", "shared operation", &["shared route"], 1),
        capability("fixture/beta", "shared operation", &["shared route"], 1),
    ]);
    let result =
        RoutingIndex::build(&desired, &policy())?.route("shared route", RouteConfig::default())?;
    assert_eq!(result.decision, RouteDecision::Ambiguous);
    Ok(())
}

#[test]
fn top_one_still_uses_runner_up_for_ambiguity() -> Result<(), Box<dyn std::error::Error>> {
    let desired = catalog(vec![
        capability("fixture/alpha", "shared operation", &["shared route"], 1),
        capability("fixture/beta", "shared operation", &["shared route"], 1),
    ]);
    let result = RoutingIndex::build(&desired, &policy())?.route(
        "shared route",
        RouteConfig {
            top_k: 1,
            ..RouteConfig::default()
        },
    )?;
    assert_eq!(result.decision, RouteDecision::Ambiguous);
    assert_eq!(result.matches.len(), 1);
    Ok(())
}

#[test]
fn returns_no_match_for_unseen_intent() -> Result<(), Box<dyn std::error::Error>> {
    let desired = catalog(vec![capability(
        "fixture/alpha",
        "architecture review",
        &["design"],
        1,
    )]);
    let result = RoutingIndex::build(&desired, &policy())?
        .route("quantum banana", RouteConfig::default())?;
    assert_eq!(result.decision, RouteDecision::NoMatch);
    Ok(())
}

#[test]
fn reconcile_retains_orphans_and_emits_preconditions() -> Result<(), Box<dyn std::error::Error>> {
    let item = capability("fixture/alpha", "architecture review", &["design"], 2);
    let desired = catalog(vec![item.clone()]);
    let observed = ObservedCatalog {
        api_version: ApiVersion::V1Alpha1,
        kind: ObservedKind::ObservedCatalog,
        observed_resource_version: 0,
        items: vec![
            ObservedCapability {
                name: "fixture/alpha".into(),
                generation: 1,
                digest: "0".repeat(64),
            },
            ObservedCapability {
                name: "fixture/orphan".into(),
                generation: 1,
                digest: "1".repeat(64),
            },
        ],
    };
    let plan = plan_reconcile(&desired, &observed, &policy())?;
    assert_eq!(plan.actions.len(), 1);
    let ReconcileAction::Update {
        expected_observed_generation,
        expected_observed_digest,
        desired_digest,
        ..
    } = &plan.actions[0]
    else {
        return Err("expected update action".into());
    };
    assert_eq!(*expected_observed_generation, 1);
    assert_eq!(expected_observed_digest, &"0".repeat(64));
    assert_eq!(desired_digest, &capability_digest(&item)?);
    assert_eq!(plan.retained_orphans, vec!["fixture/orphan"]);
    Ok(())
}

#[test]
fn reconcile_rejects_observed_generation_ahead_of_desired() {
    let desired = catalog(vec![capability(
        "fixture/alpha",
        "architecture review",
        &["design"],
        1,
    )]);
    let observed = ObservedCatalog {
        api_version: ApiVersion::V1Alpha1,
        kind: ObservedKind::ObservedCatalog,
        observed_resource_version: 2,
        items: vec![ObservedCapability {
            name: "fixture/alpha".into(),
            generation: 2,
            digest: "0".repeat(64),
        }],
    };
    let error = plan_reconcile(&desired, &observed, &policy()).err();
    assert_eq!(
        error.map(|value| value.code),
        Some(ErrorCode::StaleGeneration)
    );
}

#[test]
fn reconcile_create_requires_observed_absence() -> Result<(), Box<dyn std::error::Error>> {
    let desired = catalog(vec![capability(
        "fixture/alpha",
        "architecture review",
        &["design"],
        1,
    )]);
    let observed = ObservedCatalog {
        api_version: ApiVersion::V1Alpha1,
        kind: ObservedKind::ObservedCatalog,
        observed_resource_version: 0,
        items: Vec::new(),
    };
    let plan = plan_reconcile(&desired, &observed, &policy())?;
    assert_eq!(plan.actions.len(), 1);
    assert!(matches!(plan.actions[0], ReconcileAction::Create { .. }));
    assert_eq!(plan.policy.generation, 1);
    assert_eq!(plan.policy.digest.len(), 64);
    Ok(())
}

#[test]
fn reconcile_action_wire_shape_cannot_omit_or_mix_preconditions() {
    let update_without_preconditions = r#"{
        "kind":"update",
        "capabilityId":"fixture/alpha",
        "desiredGeneration":2,
        "desiredDigest":"0000000000000000000000000000000000000000000000000000000000000000"
    }"#;
    let create_with_update_precondition = r#"{
        "kind":"create",
        "capabilityId":"fixture/alpha",
        "desiredGeneration":1,
        "expectedObservedGeneration":1,
        "desiredDigest":"0000000000000000000000000000000000000000000000000000000000000000"
    }"#;
    assert!(serde_json::from_str::<ReconcileAction>(update_without_preconditions).is_err());
    assert!(serde_json::from_str::<ReconcileAction>(create_with_update_precondition).is_err());
}

#[test]
fn protocol_rejects_unknown_fields_and_permissions() {
    let unknown_field = r#"{
        "apiVersion":"capctl.dev/v1alpha1",
        "kind":"CapabilityCatalog",
        "resourceVersion":1,
        "items":[],
        "ambient":true
    }"#;
    assert!(serde_json::from_str::<CapabilityCatalog>(unknown_field).is_err());

    let unknown_permission = r#"{
        "apiVersion":"capctl.dev/v1alpha1",
        "kind":"CapabilityPolicy",
        "metadata":{"name":"test","generation":1},
        "spec":{
            "allowedAuthorities":["fixture"],
            "allowedPermissions":["executeShell"],
            "limits":{
                "maxCatalogItems":1,"maxQueryBytes":1,"maxFieldBytes":1,
                "maxQueryTerms":1,"maxTriggersPerCapability":1,
                "maxTagsPerCapability":1,"maxLabelsPerCapability":1,
                "maxTermsPerCapability":1,"maxCatalogTerms":1,
                "maxCatalogTextBytes":1,"maxTopK":1,"maxDocumentBytes":1
            }
        }
    }"#;
    assert!(serde_json::from_str::<CapabilityPolicy>(unknown_permission).is_err());
}

#[test]
fn fixed_width_generation_rejects_negative_fraction_and_overflow() {
    for generation in ["-1", "1.5", "4294967296"] {
        let json = format!(r#"{{"name":"test","generation":{generation}}}"#);
        assert!(serde_json::from_str::<PolicyMetadata>(&json).is_err());
    }
}

#[test]
fn route_output_does_not_echo_query_material() -> Result<(), Box<dyn std::error::Error>> {
    let canary = "uniquecanarytoken";
    let desired = catalog(vec![capability(
        "fixture/private",
        "private selector",
        &[canary],
        1,
    )]);
    let result = RoutingIndex::build(&desired, &policy())?.route(canary, RouteConfig::default())?;
    let serialized = serde_json::to_string(&result)?;
    assert!(!serialized.contains(canary));
    Ok(())
}

proptest! {
    #[test]
    fn arbitrary_queries_are_deterministic(query in "\\PC{0,1024}") {
        let desired = catalog(vec![
            capability("fixture/alpha", "architecture review", &["design"], 1),
            capability("fixture/session", "recover context", &["恢复会话"], 1),
        ]);
        let index = RoutingIndex::build(&desired, &policy());
        prop_assert!(index.is_ok());
        if let Ok(index) = index {
            let first = index.route(&query, RouteConfig::default());
            let second = index.route(&query, RouteConfig::default());
            prop_assert_eq!(first, second);
        }
    }
}
