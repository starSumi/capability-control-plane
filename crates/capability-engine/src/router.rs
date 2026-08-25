use crate::admission::{Denial, admit};
use crate::digest::{catalog_digest, digest_value, policy_ref};
use crate::normalize::terms;
use capability_protocol::{
    ApiVersion, CapabilityCatalog, CapabilityPolicy, ErrorCode, RouteDecision, RouteMatch,
    RouteResult, RouteResultKind, SelectorRef,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

const NAME_WEIGHT: f64 = 5.0;
const TRIGGER_WEIGHT: f64 = 3.0;
const TAG_WEIGHT: f64 = 2.0;
const DESCRIPTION_WEIGHT: f64 = 1.0;
const BM25_K1: f64 = 1.2;
const BM25_B: f64 = 0.75;
const SCORE_SCALE: f64 = 1_000_000.0;
const EXPLICIT_SCORE_MICROS: u32 = u32::MAX;
const SELECTOR_NAME: &str = "weighted-bm25";
const SELECTOR_VERSION: &str = "1";

/// Runtime query bounds and deterministic, integer wire thresholds.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteConfig {
    pub top_k: u32,
    pub min_score_micros: u32,
    /// Runner-up/top threshold in basis points; `10_000` means equal score.
    pub ambiguity_basis_points: u32,
}

impl Default for RouteConfig {
    fn default() -> Self {
        Self {
            top_k: 5,
            min_score_micros: 100_000,
            ambiguity_basis_points: 9_200,
        }
    }
}

#[derive(Debug)]
struct Posting {
    document: usize,
    weighted_tf: f64,
}

#[derive(Debug)]
struct Document {
    id: String,
    normalized_id: Vec<String>,
    weighted_len: f64,
}

#[derive(Debug)]
struct Candidate {
    route_match: RouteMatch,
    explicit: bool,
}

/// Immutable sparse index built from one admitted desired catalog.
#[derive(Debug)]
pub struct RoutingIndex {
    resource_version: u32,
    catalog_digest: String,
    policy: capability_protocol::PolicyRef,
    max_query_bytes: usize,
    max_query_terms: usize,
    max_top_k: u32,
    documents: Vec<Document>,
    postings: BTreeMap<String, Vec<Posting>>,
    average_len: f64,
}

impl RoutingIndex {
    /// Build a deterministic weighted sparse index after policy admission.
    ///
    /// # Errors
    ///
    /// Returns [`Denial`] when admission or digest construction fails.
    pub fn build(catalog: &CapabilityCatalog, policy: &CapabilityPolicy) -> Result<Self, Denial> {
        admit(catalog, policy)?;
        let mut documents = Vec::with_capacity(catalog.items.len());
        let mut postings: BTreeMap<String, Vec<Posting>> = BTreeMap::new();

        let mut ordered = catalog.items.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| left.metadata.name.cmp(&right.metadata.name));
        for capability in ordered {
            let mut frequencies = BTreeMap::<String, f64>::new();
            add_terms(&mut frequencies, &capability.metadata.name, NAME_WEIGHT);
            add_terms(
                &mut frequencies,
                &capability.spec.description,
                DESCRIPTION_WEIGHT,
            );
            for trigger in &capability.spec.triggers {
                add_terms(&mut frequencies, trigger, TRIGGER_WEIGHT);
            }
            for tag in &capability.spec.tags {
                add_terms(&mut frequencies, tag, TAG_WEIGHT);
            }
            let weighted_len = frequencies.values().sum::<f64>().max(1.0);
            let document = documents.len();
            for (term, weighted_tf) in frequencies {
                postings.entry(term).or_default().push(Posting {
                    document,
                    weighted_tf,
                });
            }
            documents.push(Document {
                id: capability.metadata.name.clone(),
                normalized_id: terms(&capability.metadata.name),
                weighted_len,
            });
        }

        let average_len = if documents.is_empty() {
            1.0
        } else {
            documents
                .iter()
                .map(|document| document.weighted_len)
                .sum::<f64>()
                / bounded_f64(documents.len())
        };

        Ok(Self {
            resource_version: catalog.resource_version,
            catalog_digest: catalog_digest(catalog)?,
            policy: policy_ref(policy)?,
            max_query_bytes: to_usize(policy.spec.limits.max_query_bytes)?,
            max_query_terms: to_usize(policy.spec.limits.max_query_terms)?,
            max_top_k: policy.spec.limits.max_top_k,
            documents,
            postings,
            average_len,
        })
    }

    /// Route one untrusted query without loading or executing a capability.
    ///
    /// The public result contains no query text, tokens, or stable query digest.
    /// Scores are quantized before ordering to make replay comparisons stable.
    ///
    /// # Errors
    ///
    /// Returns [`Denial`] when query or route bounds are invalid.
    pub fn route(&self, query: &str, config: RouteConfig) -> Result<RouteResult, Denial> {
        if query.trim().is_empty() || query.len() > self.max_query_bytes {
            return Err(Denial::new(
                ErrorCode::InvalidQuery,
                format!(
                    "query must contain 1..={} UTF-8 bytes",
                    self.max_query_bytes
                ),
            ));
        }
        if config.top_k == 0
            || config.top_k > self.max_top_k
            || config.ambiguity_basis_points > 10_000
        {
            return Err(Denial::new(
                ErrorCode::InvalidPolicy,
                "route configuration is outside admitted bounds",
            ));
        }

        let query_terms = terms(query).into_iter().collect::<BTreeSet<_>>();
        if query_terms.len() > self.max_query_terms {
            return Err(Denial::new(
                ErrorCode::ResourceLimitExceeded,
                "query term count exceeds policy",
            ));
        }
        let normalized_query = query_terms.iter().cloned().collect::<Vec<_>>();
        let (scores, explicit) = self.score_documents(query, &query_terms, &normalized_query);

        let mut candidates = self
            .documents
            .iter()
            .enumerate()
            .filter(|(index, _)| explicit[*index] || scores[*index] > 0.0)
            .map(|(index, document)| Candidate {
                route_match: RouteMatch {
                    capability_id: document.id.clone(),
                    score_micros: if explicit[index] {
                        EXPLICIT_SCORE_MICROS
                    } else {
                        quantize_score(scores[index])
                    },
                },
                explicit: explicit[index],
            })
            .collect::<Vec<_>>();

        let output_count = to_usize(config.top_k)?;
        let decision_count = output_count.max(2).min(candidates.len());
        if candidates.len() > decision_count {
            candidates.select_nth_unstable_by(decision_count, candidate_order);
            candidates.truncate(decision_count);
        }
        candidates.sort_by(candidate_order);

        let decision = route_decision(&candidates, config);
        candidates.truncate(output_count);
        Ok(RouteResult {
            api_version: ApiVersion::V1Alpha1,
            kind: RouteResultKind::RouteResult,
            resource_version: self.resource_version,
            catalog_digest: self.catalog_digest.clone(),
            policy: self.policy.clone(),
            selector: SelectorRef {
                name: SELECTOR_NAME.into(),
                version: SELECTOR_VERSION.into(),
                config_digest: digest_value(&config)?,
            },
            decision,
            matches: candidates
                .into_iter()
                .map(|candidate| candidate.route_match)
                .collect(),
        })
    }

    fn score_documents(
        &self,
        query: &str,
        query_terms: &BTreeSet<String>,
        normalized_query: &[String],
    ) -> (Vec<f64>, Vec<bool>) {
        let mut scores = vec![0.0; self.documents.len()];
        let document_count = bounded_f64(self.documents.len());

        for term in query_terms {
            let Some(postings) = self.postings.get(term) else {
                continue;
            };
            let document_frequency = bounded_f64(postings.len());
            let idf = (1.0
                + (document_count - document_frequency + 0.5) / (document_frequency + 0.5))
                .ln();
            for posting in postings {
                let document = &self.documents[posting.document];
                let length_ratio = document.weighted_len / self.average_len;
                let denominator =
                    posting.weighted_tf + BM25_K1 * (1.0 - BM25_B + BM25_B * length_ratio);
                scores[posting.document] +=
                    idf * posting.weighted_tf * (BM25_K1 + 1.0) / denominator;
            }
        }

        let mut explicit = vec![false; self.documents.len()];
        for (index, document) in self.documents.iter().enumerate() {
            if query.trim().eq_ignore_ascii_case(&document.id)
                || normalized_query == document.normalized_id
            {
                explicit[index] = true;
            }
        }
        (scores, explicit)
    }
}

fn route_decision(candidates: &[Candidate], config: RouteConfig) -> RouteDecision {
    match candidates {
        [] => RouteDecision::NoMatch,
        [first, ..] if first.route_match.score_micros < config.min_score_micros => {
            RouteDecision::NoMatch
        }
        [first, second, ..]
            if !first.explicit
                && u64::from(second.route_match.score_micros) * 10_000
                    >= u64::from(first.route_match.score_micros)
                        * u64::from(config.ambiguity_basis_points) =>
        {
            RouteDecision::Ambiguous
        }
        _ => RouteDecision::Selected,
    }
}

fn add_terms(frequencies: &mut BTreeMap<String, f64>, value: &str, weight: f64) {
    for term in terms(value) {
        *frequencies.entry(term).or_default() += weight;
    }
}

fn candidate_order(left: &Candidate, right: &Candidate) -> Ordering {
    right
        .explicit
        .cmp(&left.explicit)
        .then_with(|| {
            right
                .route_match
                .score_micros
                .cmp(&left.route_match.score_micros)
        })
        .then_with(|| {
            left.route_match
                .capability_id
                .cmp(&right.route_match.capability_id)
        })
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn quantize_score(score: f64) -> u32 {
    let scaled = (score * SCORE_SCALE).round();
    if !scaled.is_finite() || scaled <= 0.0 {
        0
    } else if scaled >= f64::from(u32::MAX - 1) {
        u32::MAX - 1
    } else {
        scaled as u32
    }
}

fn bounded_f64(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}

fn to_usize(value: u32) -> Result<usize, Denial> {
    usize::try_from(value).map_err(|_| {
        Denial::new(
            ErrorCode::InvalidPolicy,
            "policy value cannot be represented on this target",
        )
    })
}
