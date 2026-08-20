use crate::admission::{Denial, admit};
use crate::digest::{capability_digest, catalog_digest, policy_ref};
use capability_protocol::{
    ApiVersion, CapabilityCatalog, CapabilityPolicy, ErrorCode, ObservedCatalog, ReconcileAction,
    ReconcilePlan, ReconcilePlanKind, validate_observed_semantics,
};
use std::collections::{BTreeMap, BTreeSet};

/// Compute a side-effect-free reconcile plan from desired and observed state.
///
/// # Errors
///
/// Returns [`Denial`] when desired state fails admission, observed state is
/// malformed, or an observed generation is ahead of desired state.
pub fn plan_reconcile(
    desired: &CapabilityCatalog,
    observed: &ObservedCatalog,
    policy: &CapabilityPolicy,
) -> Result<ReconcilePlan, Denial> {
    admit(desired, policy)?;
    validate_observed_semantics(observed)?;
    if observed.observed_resource_version > desired.resource_version {
        return Err(Denial::new(
            ErrorCode::StaleGeneration,
            "observed resourceVersion is ahead of desired state",
        ));
    }
    let mut observed_by_name = BTreeMap::new();
    for item in &observed.items {
        if observed_by_name.insert(item.name.as_str(), item).is_some() {
            return Err(Denial::new(
                ErrorCode::DuplicateCapability,
                format!("duplicate observed capability: {}", item.name),
            ));
        }
    }

    let mut desired_names = BTreeSet::new();
    let mut actions = Vec::new();
    let mut ordered = desired.items.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.metadata.name.cmp(&right.metadata.name));
    for capability in ordered {
        desired_names.insert(capability.metadata.name.as_str());
        let desired_digest = capability_digest(capability)?;
        match observed_by_name.get(capability.metadata.name.as_str()) {
            None => actions.push(ReconcileAction::Create {
                capability_id: capability.metadata.name.clone(),
                desired_generation: capability.metadata.generation,
                desired_digest,
            }),
            Some(current) if current.generation > capability.metadata.generation => {
                return Err(Denial::new(
                    ErrorCode::StaleGeneration,
                    format!(
                        "observed generation for {} is ahead of desired state",
                        capability.metadata.name
                    ),
                ));
            }
            Some(current)
                if current.generation != capability.metadata.generation
                    || current.digest != desired_digest =>
            {
                actions.push(ReconcileAction::Update {
                    capability_id: capability.metadata.name.clone(),
                    desired_generation: capability.metadata.generation,
                    expected_observed_generation: current.generation,
                    expected_observed_digest: current.digest.clone(),
                    desired_digest,
                });
            }
            Some(_) => {}
        }
    }

    let retained_orphans = observed_by_name
        .keys()
        .filter(|name| !desired_names.contains(**name))
        .map(|name| (*name).to_owned())
        .collect();

    Ok(ReconcilePlan {
        api_version: ApiVersion::V1Alpha1,
        kind: ReconcilePlanKind::ReconcilePlan,
        desired_resource_version: desired.resource_version,
        observed_resource_version: observed.observed_resource_version,
        desired_digest: catalog_digest(desired)?,
        policy: policy_ref(policy)?,
        actions,
        retained_orphans,
    })
}
