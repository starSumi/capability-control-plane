use capability_engine::{RouteConfig, RoutingIndex};
use capability_protocol::{
    ApiVersion, AuthorityRef, Capability, CapabilityCatalog, CapabilityMetadata, CapabilityPolicy,
    CapabilitySpec, CatalogKind, LoadMode, LoadSpec, Permission, PolicyKind, PolicyMetadata,
    PolicySpec, ResourceLimits,
};
use std::collections::{BTreeMap, BTreeSet};

fn main() {
    divan::main();
}

#[divan::bench(args = [32, 256, 412, 4096, 10_000])]
fn build_index(bencher: divan::Bencher, size: usize) {
    let policy = policy(size);
    let catalog = catalog(size);
    bencher.bench(|| RoutingIndex::build(divan::black_box(&catalog), divan::black_box(&policy)));
}

#[divan::bench(args = [32, 256, 412, 4096, 10_000])]
fn warm_route(bencher: divan::Bencher, size: usize) {
    let policy = policy(size);
    let catalog = catalog(size);
    let index = RoutingIndex::build(&catalog, &policy);
    if let Ok(index) = index {
        bencher.bench(|| {
            index.route(
                divan::black_box("恢复会话 architecture review capability 17"),
                RouteConfig::default(),
            )
        });
    }
}

fn policy(size: usize) -> CapabilityPolicy {
    let catalog_items = u32::try_from(size.max(4096)).unwrap_or(u32::MAX);
    CapabilityPolicy {
        api_version: ApiVersion::V1Alpha1,
        kind: PolicyKind::CapabilityPolicy,
        metadata: PolicyMetadata {
            name: "benchmark-policy".into(),
            generation: 1,
        },
        spec: PolicySpec {
            allowed_authorities: BTreeSet::from(["fixture".into()]),
            allowed_permissions: BTreeSet::from([Permission::ReadMetadata]),
            limits: ResourceLimits {
                max_catalog_items: catalog_items,
                max_query_bytes: 16_384,
                max_query_terms: 4_096,
                max_field_bytes: 65_536,
                max_triggers_per_capability: 64,
                max_tags_per_capability: 64,
                max_labels_per_capability: 32,
                max_terms_per_capability: 4_096,
                max_catalog_terms: 2_000_000,
                max_catalog_text_bytes: 8 * 1024 * 1024,
                max_top_k: 50,
                max_document_bytes: 8 * 1024 * 1024,
            },
        },
    }
}

fn catalog(size: usize) -> CapabilityCatalog {
    let items = (0..size)
        .map(|index| Capability {
            metadata: CapabilityMetadata {
                name: format!("fixture/capability-{index}"),
                generation: 1,
                labels: BTreeMap::from([("domain".into(), format!("domain-{}", index % 17))]),
            },
            spec: CapabilitySpec {
                description: format!(
                    "Capability {index} handles deterministic architecture and recovery workflow"
                ),
                triggers: vec![
                    format!("capability {index}"),
                    "恢复会话".into(),
                    format!("topic {}", index % 31),
                ],
                tags: vec!["benchmark".into(), format!("group-{}", index % 11)],
                authority: AuthorityRef {
                    provider: "fixture".into(),
                    package: "benchmark".into(),
                    resource: format!("capability-{index}"),
                },
                load: LoadSpec {
                    mode: LoadMode::MetadataOnly,
                    estimated_tokens: 32,
                },
                permissions: BTreeSet::from([Permission::ReadMetadata]),
            },
        })
        .collect();
    CapabilityCatalog {
        api_version: ApiVersion::V1Alpha1,
        kind: CatalogKind::CapabilityCatalog,
        resource_version: 1,
        items,
    }
}
