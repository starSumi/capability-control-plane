//! Pure admission, routing, and reconciliation algorithms.
//!
//! The engine performs no I/O and executes no capability. Callers provide
//! versioned resources and receive deterministic decisions or plans.

mod admission;
mod digest;
mod normalize;
mod reconcile;
mod router;
mod shadow;

pub use admission::{Denial, admit, validate_policy};
pub use capability_protocol::{
    ErrorCode, ReconcileAction, ReconcilePlan, RouteDecision, RouteMatch, RouteResult,
};
pub use digest::{capability_digest, catalog_digest, policy_ref};
pub use reconcile::plan_reconcile;
pub use router::{RouteConfig, RoutingIndex};
pub use shadow::{ShadowCase, ShadowFixture, ShadowReplay, replay_shadow};
