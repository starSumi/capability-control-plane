use crate::admission::Denial;
use crate::router::{RouteConfig, RoutingIndex};
use capability_protocol::{
    ApiVersion, CapabilityCatalog, CapabilityPolicy, GroundTruthSource, RouteDecision,
    ShadowEvaluation, ShadowEvaluationKind, ShadowGroundTruth, ShadowObservation,
    ShadowObservationKind, validate_identifier,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const FIXTURE_FORMAT: &str = "capctl.dev/shadow-heldout/v1alpha1";
const CATALOG_FILE: &str = "catalog.v1alpha1.json";
const POLICY_FILE: &str = "policy.v1alpha1.json";
const SELECTOR_NAME: &str = "weighted-bm25";
const SELECTOR_VERSION: &str = "1";

/// Query-bearing evaluation input. It is never included in replay output.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShadowFixture {
    pub format: String,
    pub id: String,
    pub catalog: String,
    pub policy: String,
    pub selector: ShadowFixtureSelector,
    pub provenance: ShadowFixtureProvenance,
    pub cases: Vec<ShadowCase>,
    pub revision: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShadowFixtureProvenance {
    pub source: String,
    pub basis: String,
    pub reviewed_by: String,
    pub reviewed_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShadowFixtureSelector {
    pub name: String,
    pub version: String,
    pub config: RouteConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShadowCase {
    pub id: String,
    pub query: String,
    pub expected_decision: RouteDecision,
    pub expected_matches: Vec<String>,
}

/// A replay result containing aggregate metrics and privacy-safe observations.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShadowReplay {
    pub evaluation: ShadowEvaluation,
    pub observations: Vec<ShadowObservation>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RevisionInput<'a> {
    format: &'a str,
    id: &'a str,
    catalog: &'a str,
    policy: &'a str,
    selector: &'a ShadowFixtureSelector,
    provenance: &'a ShadowFixtureProvenance,
    cases: &'a [ShadowCase],
}

/// Replay held-out cases against one immutable index without executing or
/// loading any capability. The query only exists in the input fixture.
///
/// # Errors
///
/// Returns [`Denial`] when fixture provenance, route bounds, or catalog/policy
/// admission is invalid.
pub fn replay_shadow(
    fixture: &ShadowFixture,
    catalog: &CapabilityCatalog,
    policy: &CapabilityPolicy,
) -> Result<ShadowReplay, Denial> {
    validate_fixture(fixture)?;
    let expected_revision = fixture_revision(fixture)?;
    if fixture.revision != expected_revision {
        return Err(invalid_fixture(&format!(
            "fixture revision does not match canonical content; expected {expected_revision}"
        )));
    }

    let index = RoutingIndex::build(catalog, policy)?;
    let mut observations = Vec::with_capacity(fixture.cases.len());
    let mut counts = ReplayCounts::default();
    for case in &fixture.cases {
        let (observation, metrics) = replay_case(&index, fixture.selector.config, case)?;
        counts.add(&metrics);
        observations.push(observation);
    }

    let case_count = u32::try_from(fixture.cases.len())
        .map_err(|_| invalid_fixture("fixture case count is not representable"))?;
    let first = observations
        .first()
        .ok_or_else(|| invalid_fixture("fixture has no cases"))?;
    Ok(ShadowReplay {
        evaluation: ShadowEvaluation {
            api_version: ApiVersion::V1Alpha1,
            kind: ShadowEvaluationKind::ShadowEvaluation,
            fixture_id: fixture.id.clone(),
            fixture_revision: fixture.revision.clone(),
            catalog_digest: first.catalog_digest.clone(),
            policy: first.policy.clone(),
            selector: first.selector.clone(),
            ground_truth_source: GroundTruthSource::HeldOutReview,
            case_count,
            exact_decision: counts.exact_decision,
            hit_at_1: counts.hit_at_1,
            hit_at_3: counts.hit_at_3,
            false_positive: counts.false_positive,
            ambiguous: counts.ambiguous,
            no_match: counts.no_match,
        },
        observations,
    })
}

#[derive(Default)]
struct ReplayCounts {
    exact_decision: u32,
    hit_at_1: u32,
    hit_at_3: u32,
    false_positive: u32,
    ambiguous: u32,
    no_match: u32,
}

impl ReplayCounts {
    fn add(&mut self, metrics: &CaseMetrics) {
        self.exact_decision += metrics.exact_decision;
        self.hit_at_1 += metrics.hit_at_1;
        self.hit_at_3 += metrics.hit_at_3;
        self.false_positive += metrics.false_positive;
        self.ambiguous += metrics.ambiguous;
        self.no_match += metrics.no_match;
    }
}

struct CaseMetrics {
    exact_decision: u32,
    hit_at_1: u32,
    hit_at_3: u32,
    false_positive: u32,
    ambiguous: u32,
    no_match: u32,
}

fn replay_case(
    index: &RoutingIndex,
    config: RouteConfig,
    case: &ShadowCase,
) -> Result<(ShadowObservation, CaseMetrics), Denial> {
    let result = index.route(&case.query, config)?;
    let actual_ids = result
        .matches
        .iter()
        .map(|item| item.capability_id.as_str())
        .collect::<Vec<_>>();
    let expected_ids = case
        .expected_matches
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let selected = case.expected_decision == RouteDecision::Selected;
    let hit_at_1 = selected
        && actual_ids
            .first()
            .is_some_and(|id| expected_ids.contains(id));
    let hit_at_3 = selected
        && actual_ids
            .iter()
            .take(3)
            .any(|id| expected_ids.contains(id));
    let false_positive = (case.expected_decision == RouteDecision::NoMatch
        && result.decision != RouteDecision::NoMatch)
        || (selected
            && result.decision == RouteDecision::Selected
            && !actual_ids.iter().any(|id| expected_ids.contains(id)));
    let metrics = CaseMetrics {
        exact_decision: u32::from(result.decision == case.expected_decision),
        hit_at_1: u32::from(hit_at_1),
        hit_at_3: u32::from(hit_at_3),
        false_positive: u32::from(false_positive),
        ambiguous: u32::from(result.decision == RouteDecision::Ambiguous),
        no_match: u32::from(result.decision == RouteDecision::NoMatch),
    };
    let observation = ShadowObservation {
        api_version: ApiVersion::V1Alpha1,
        kind: ShadowObservationKind::ShadowObservation,
        case_id: case.id.clone(),
        catalog_digest: result.catalog_digest,
        policy: result.policy,
        selector: result.selector,
        predicted_decision: result.decision,
        predicted_matches: result.matches,
        ground_truth: ShadowGroundTruth {
            source: GroundTruthSource::HeldOutReview,
            decision: case.expected_decision,
            capability_ids: case.expected_matches.clone(),
        },
        elapsed_micros: None,
    };
    Ok((observation, metrics))
}

fn validate_fixture(fixture: &ShadowFixture) -> Result<(), Denial> {
    if fixture.format != FIXTURE_FORMAT
        || fixture.catalog != CATALOG_FILE
        || fixture.policy != POLICY_FILE
        || fixture.cases.is_empty()
        || fixture.cases.len() > 10_000
        || fixture.selector.name != SELECTOR_NAME
        || fixture.selector.version != SELECTOR_VERSION
    {
        return Err(invalid_fixture(
            "fixture metadata is outside the held-out contract",
        ));
    }
    if fixture.provenance.source != "manual-review"
        || fixture.provenance.basis.trim().is_empty()
        || fixture.provenance.basis.len() > 512
        || fixture.provenance.reviewed_by.trim().is_empty()
        || fixture.provenance.reviewed_by.len() > 128
        || fixture.provenance.reviewed_at.trim().is_empty()
        || fixture.provenance.reviewed_at.len() > 64
    {
        return Err(invalid_fixture("fixture provenance is incomplete"));
    }
    validate_identifier(&fixture.id).map_err(Denial::from)?;
    if fixture.revision.len() != 64
        || !fixture
            .revision
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(invalid_fixture(
            "fixture revision must be 64 hexadecimal characters",
        ));
    }
    for case in &fixture.cases {
        validate_identifier(&case.id).map_err(Denial::from)?;
        if case.query.trim().is_empty() || case.query.len() > 16 * 1024 {
            return Err(invalid_fixture(
                "fixture query is empty or exceeds the query ceiling",
            ));
        }
        if case.expected_decision == RouteDecision::Selected && case.expected_matches.is_empty() {
            return Err(invalid_fixture(
                "selected ground truth requires a capability id",
            ));
        }
        for capability_id in &case.expected_matches {
            validate_identifier(capability_id).map_err(Denial::from)?;
        }
    }
    Ok(())
}

fn fixture_revision(fixture: &ShadowFixture) -> Result<String, Denial> {
    let unsigned = RevisionInput {
        format: &fixture.format,
        id: &fixture.id,
        catalog: &fixture.catalog,
        policy: &fixture.policy,
        selector: &fixture.selector,
        provenance: &fixture.provenance,
        cases: &fixture.cases,
    };
    let bytes = serde_json::to_vec(&unsigned)
        .map_err(|_| invalid_fixture("fixture provenance could not be canonicalized"))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn invalid_fixture(message: &str) -> Denial {
    Denial::new(capability_protocol::ErrorCode::InvalidFixture, message)
}
