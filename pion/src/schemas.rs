#[cfg(feature = "runtime")]
mod agent_handoff_directive_schema {
    include!("../schemas/valence/agent_handoff_directive_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod control_plane_handoff_schema {
    include!("../schemas/valence/control_plane_handoff_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod control_plane_authority_schema {
    include!("../schemas/valence/control_plane_authority_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod control_plane_handoff_migration_schema {
    include!("../schemas/valence/control_plane_handoff_migration_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod control_plane_cell_schema {
    include!("../schemas/valence/control_plane_cell_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod control_plane_node_schema {
    include!("../schemas/valence/control_plane_node_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod node_reachability_schema {
    include!("../schemas/valence/node_reachability_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod agent_host_enrollment_schema {
    include!("../schemas/valence/agent_host_enrollment_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod control_plane_node_action_capability_schema {
    include!("../schemas/valence/control_plane_node_action_capability_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod control_plane_observed_status_schema {
    include!("../schemas/valence/control_plane_observed_status_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod container_observation_schema {
    include!("../schemas/valence/container_observation_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod control_plane_action_event_schema {
    include!("../schemas/valence/control_plane_action_event_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod control_plane_lease_schema {
    include!("../schemas/valence/control_plane_lease_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod control_plane_service_schema {
    include!("../schemas/valence/control_plane_service_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod control_plane_endpoints_schema {
    include!("../schemas/valence/control_plane_endpoints_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod control_plane_virtual_pool_schema {
    include!("../schemas/valence/control_plane_virtual_pool_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod control_plane_pool_cell_map_schema {
    include!("../schemas/valence/control_plane_pool_cell_map_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod node_action_command_schema {
    include!("../schemas/valence/node_action_command_valence_schema.rs");
}

#[cfg(feature = "runtime")]
mod node_action_result_schema {
    include!("../schemas/valence/node_action_result_valence_schema.rs");
}
