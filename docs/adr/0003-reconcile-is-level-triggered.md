# ADR 0003: Reconciliation is level-triggered and rebuildable

- Status: accepted
- Date: 2026-08-20

Future actuators must Observe -> Diff -> Act -> Verify. Events are hints and may
coalesce keys; startup, queue overflow, and restart trigger a full scan. Status
is evidence, not proof of an effect. A stale generation or digest aborts the
action and causes a fresh plan. Rollback changes desired state and reconciles;
it does not replay inverse side effects.

