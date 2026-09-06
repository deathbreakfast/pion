//! Candidate [Chronon](https://docs.rs/uf-chronon) (`uf-chronon`, cron + run-once job scheduler)
//! job specs for `pion`'s periodic maintenance sweeps.
//!
//! # External blocker
//!
//! `chronon` is **not** a workspace dependency of this crate (see `Cargo.toml`
//! `[workspace.dependencies]` — cores such as `valence`, `spectra`, and `photon` are pinned
//! via crates.io / git). Wiring Chronon in means adding a published or git-pinned
//! `uf-chronon` (and related crates) to `[workspace.dependencies]`, which is out of scope here.
//!
//! Until then, `pion`'s periodic maintenance today runs **inline**, opportunistically, on the hot
//! ingest/claim path rather than on a schedule — e.g. `control_plane::node_actions`'s private
//! `reconcile_expired_lease_node_action` helper only reconciles a node action's expired lease when
//! that specific row is next touched by a claim. There is no background sweep that guarantees
//! timely cleanup of rows nobody happens to touch again (an abandoned enrollment ticket, a node
//! that stops heartbeating mid-lease, etc).
//!
//! This module documents the maintenance jobs that *should* run on a Chronon cron schedule once
//! the dependency lands, as plain, `chronon`-independent data ([`ChrononJobSpec`]) so:
//!
//! - the intent and target cron cadence are captured and reviewable now, without inventing a fake
//!   Chronon integration against a crate that isn't linked;
//! - wiring the real dependency later is a matter of mapping each [`ChrononJobSpec`] onto
//!   `chronon_core::Job` (`schedule_kind = Cron`, `cron_expr`, `timezone`) and registering an
//!   executor, not re-deriving what jobs are needed.
//!
//! ```
//! use pion::chronon_hooks::ALL_JOBS;
//!
//! for job in ALL_JOBS {
//!     println!("{}: {} ({})", job.name, job.cron_expr, job.description);
//! }
//! ```

/// A prospective Chronon cron job: name, standard 5-field cron expression, and human-readable
/// intent. Field shapes intentionally mirror `chronon_core::Job`'s `name` / `cron_expr` so mapping
/// onto a real `Job` later is mechanical.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChrononJobSpec {
    /// Stable job name (would become `chronon_core::Job::name`).
    pub name: &'static str,
    /// Standard 5-field cron expression (minute hour day-of-month month day-of-week), UTC.
    pub cron_expr: &'static str,
    /// What the job does and why it needs a guaranteed schedule rather than opportunistic cleanup.
    pub description: &'static str,
}

/// Sweep node actions whose `lease_expires_at` has passed without a report, forcing them back to
/// retryable/failed state. Today this only happens when a claim attempt happens to touch the same
/// row again (`reconcile_expired_lease_node_action`, called from the claim path in
/// `control_plane::node_actions`); a node that goes permanently offline mid-lease otherwise leaves
/// its action stuck `running` forever.
pub const EXPIRED_LEASE_SWEEP: ChrononJobSpec = ChrononJobSpec {
    name: "pion.node_actions.expired_lease_sweep",
    cron_expr: "*/2 * * * *",
    description:
        "Reconcile node actions whose lease expired without a report back to retryable/failed.",
};

/// Revoke host-enrollment tickets (`pion_agent_host_enrollment`) past `expires_at` that are still
/// `pending`. Today an expired-but-unclaimed ticket is only rejected lazily, at the moment someone
/// tries to redeem it (see `control_plane::agent_enrollment`'s `expires_at` check); it otherwise
/// sits `pending` indefinitely.
pub const EXPIRED_ENROLLMENT_TICKET_SWEEP: ChrononJobSpec = ChrononJobSpec {
    name: "pion.agent_enrollment.expired_ticket_sweep",
    cron_expr: "*/15 * * * *",
    description: "Mark pending host-enrollment tickets past expires_at as revoked/expired.",
};

/// Prune `pion_container_observation` rows that have been stale (no heartbeat touching them) for
/// well beyond [`crate::OBSERVED_STALE_AFTER_SECS`], to bound table growth from nodes that were
/// decommissioned without a clean teardown.
pub const STALE_CONTAINER_OBSERVATION_PRUNE: ChrononJobSpec = ChrononJobSpec {
    name: "pion.container_observation.stale_prune",
    cron_expr: "0 * * * *",
    description: "Delete container-observation rows stale well beyond the freshness window (decommissioned nodes).",
};

/// All candidate jobs, in the order they'd likely be registered.
pub const ALL_JOBS: &[ChrononJobSpec] = &[
    EXPIRED_LEASE_SWEEP,
    EXPIRED_ENROLLMENT_TICKET_SWEEP,
    STALE_CONTAINER_OBSERVATION_PRUNE,
];

#[cfg(test)]
mod tests {
    use super::*;

    fn is_plausible_five_field_cron(expr: &str) -> bool {
        expr.split_whitespace().count() == 5
    }

    #[test]
    fn all_jobs_have_non_empty_fields_and_plausible_cron() {
        for job in ALL_JOBS {
            assert!(!job.name.is_empty());
            assert!(!job.description.is_empty());
            assert!(
                is_plausible_five_field_cron(job.cron_expr),
                "job {} has non-5-field cron expr: {}",
                job.name,
                job.cron_expr
            );
        }
    }

    #[test]
    fn job_names_are_unique_and_namespaced_under_pion() {
        let mut names: Vec<&str> = ALL_JOBS.iter().map(|j| j.name).collect();
        let before = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), before, "duplicate chronon job name");
        assert!(ALL_JOBS.iter().all(|j| j.name.starts_with("pion.")));
    }
}
