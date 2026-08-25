use capability_engine::{ShadowFixture, replay_shadow};
use capability_protocol::RouteDecision;
use std::collections::BTreeSet;

const CATALOG_JSON: &str = include_str!("../../../fixtures/catalog.v1alpha1.json");
const POLICY_JSON: &str = include_str!("../../../fixtures/policy.v1alpha1.json");
const FIXTURE_JSON: &str = include_str!("../../../fixtures/shadow-heldout.v1alpha1.json");

fn fixture() -> Result<ShadowFixture, serde_json::Error> {
    serde_json::from_str(FIXTURE_JSON)
}

#[test]
fn heldout_replay_is_provenance_bound_and_privacy_safe() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = fixture()?;
    let catalog = serde_json::from_str(CATALOG_JSON)?;
    let policy = serde_json::from_str(POLICY_JSON)?;
    let replay = replay_shadow(&fixture, &catalog, &policy)?;

    assert_eq!(replay.evaluation.fixture_id, "heldout/routing-v1");
    assert_eq!(replay.evaluation.fixture_revision, fixture.revision);
    assert_eq!(replay.evaluation.case_count, 7);
    assert_eq!(replay.evaluation.exact_decision, 7);
    assert_eq!(replay.evaluation.hit_at_1, 6);
    assert_eq!(replay.evaluation.hit_at_3, 6);
    assert_eq!(replay.evaluation.false_positive, 0);
    assert_eq!(replay.evaluation.no_match, 1);

    let output = serde_json::to_string(&replay)?;
    for case in &fixture.cases {
        assert!(!output.contains(&case.query));
    }
    assert!(!output.contains("manual-review"));
    Ok(())
}

#[test]
fn replay_rejects_changed_fixture_revision() -> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = fixture()?;
    fixture.cases[0].query.push('!');
    let catalog = serde_json::from_str(CATALOG_JSON)?;
    let policy = serde_json::from_str(POLICY_JSON)?;
    let Err(error) = replay_shadow(&fixture, &catalog, &policy) else {
        return Err("revision mutation unexpectedly replayed".into());
    };
    assert_eq!(error.code, capability_protocol::ErrorCode::InvalidFixture);
    Ok(())
}

#[test]
fn replay_ground_truth_ids_are_bounded_and_unique_per_case()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = fixture()?;
    for case in fixture.cases {
        let expected_count = case.expected_matches.len();
        let ids = case.expected_matches.into_iter().collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), expected_count);
        if case.expected_decision == RouteDecision::Selected {
            assert!(!ids.is_empty());
        }
    }
    Ok(())
}
