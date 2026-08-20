use capability_engine::{
    RouteConfig, RouteDecision, RouteMatch, RoutingIndex, catalog_digest, policy_ref,
};
use capability_protocol::{CapabilityCatalog, CapabilityPolicy, PolicyRef};
use serde::{Deserialize, Serialize};
use std::error::Error;

const CATALOG_JSON: &str = include_str!("../../../fixtures/catalog.v1alpha1.json");
const POLICY_JSON: &str = include_str!("../../../fixtures/policy.v1alpha1.json");
const GOLDEN_JSON: &str = include_str!("../../../fixtures/route-golden.v1alpha1.json");

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GoldenFixture {
    format: String,
    catalog: String,
    policy: String,
    cases: Vec<GoldenCase>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GoldenCase {
    id: String,
    query: String,
    expected_decision: RouteDecision,
    expected_matches: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoldenArtifact {
    format: &'static str,
    catalog: &'static str,
    policy: &'static str,
    catalog_digest: String,
    policy_ref: PolicyRef,
    selector: GoldenSelector,
    cases: Vec<GoldenResult>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoldenSelector {
    name: &'static str,
    version: &'static str,
    config: RouteConfig,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoldenResult {
    id: String,
    decision: RouteDecision,
    matches: Vec<RouteMatch>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let fixture: GoldenFixture = serde_json::from_str(GOLDEN_JSON)?;
    if fixture.format != "capctl.dev/route-golden/v1alpha1"
        || fixture.catalog != "catalog.v1alpha1.json"
        || fixture.policy != "policy.v1alpha1.json"
    {
        return Err("route golden fixture references an unexpected contract".into());
    }

    let catalog: CapabilityCatalog = serde_json::from_str(CATALOG_JSON)?;
    let policy: CapabilityPolicy = serde_json::from_str(POLICY_JSON)?;
    let index = RoutingIndex::build(&catalog, &policy)?;
    let config = RouteConfig::default();
    let mut cases = Vec::with_capacity(fixture.cases.len());

    for case in fixture.cases {
        let result = index.route(&case.query, config)?;
        let actual_matches = result
            .matches
            .iter()
            .map(|item| item.capability_id.clone())
            .collect::<Vec<_>>();
        if result.decision != case.expected_decision || actual_matches != case.expected_matches {
            return Err(format!(
                "golden case {} drifted: got {:?} {:?}, expected {:?} {:?}",
                case.id,
                result.decision,
                actual_matches,
                case.expected_decision,
                case.expected_matches
            )
            .into());
        }
        cases.push(GoldenResult {
            id: case.id,
            decision: result.decision,
            matches: result.matches,
        });
    }

    let artifact = GoldenArtifact {
        format: "capctl.dev/route-golden-result/v1alpha1",
        catalog: "catalog.v1alpha1.json",
        policy: "policy.v1alpha1.json",
        catalog_digest: catalog_digest(&catalog)?,
        policy_ref: policy_ref(&policy)?,
        selector: GoldenSelector {
            name: "weighted-bm25",
            version: "1",
            config,
        },
        cases,
    };
    serde_json::to_writer_pretty(std::io::stdout(), &artifact)?;
    println!();
    Ok(())
}
