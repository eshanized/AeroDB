//! Phase 7 Control Plane Handlers
//!
//! Per PHASE7_CONTROL_PLANE_ARCHITECTURE.md §4.4:
//! - Kernel Boundary Adapter translates control-plane requests into kernel calls
//! - Enforces strict API boundaries
//! - MUST expose only explicitly allowed kernel operations
//! - MUST NOT batch or chain kernel calls
//!
//! This is the ONLY component allowed to call kernel APIs.

use std::time::SystemTime;
use uuid::Uuid;

use super::authority::AuthorityContext;
use super::commands::{ControlCommand, ControlPlaneCommand, DiagnosticCommand, InspectionCommand};
use super::confirmation::{ConfirmationFlow, ConfirmationResult, ConfirmationToken};
use super::errors::{ControlPlaneError, ControlPlaneResult};
use super::types::{
    ClusterState, CommandOutcome, CommandRequest, CommandResponse, CommandResponseData,
    DiagnosticResult, NodeHealth, NodeRole, NodeState, PromotionResultData, PromotionStateView,
    ReplicaState, ReplicationStatus, SnapshotInfo, WalInfo,
};

/// Phase 7 Control Plane Handler.
///
/// Per PHASE7_CONTROL_PLANE_ARCHITECTURE.md:
/// - Routes requests to kernel boundary adapter
/// - Orchestrates confirmation flows
/// - Validates request structure and authority
#[derive(Debug, Default)]
pub struct ControlPlaneHandler {
    /// Confirmation flow manager (ephemeral state).
    confirmation: ConfirmationFlow,
}

impl ControlPlaneHandler {
    /// Create a new control plane handler.
    pub fn new() -> Self {
        Self {
            confirmation: ConfirmationFlow::new(),
        }
    }

    /// Handle a command request.
    ///
    /// Per PHASE7_CONTROL_PLANE_ARCHITECTURE.md §6:
    /// 1. Validate structure and authority
    /// 2. Generate pre-execution explanation (if mutating)
    /// 3. Request confirmation (if required)
    /// 4. Forward to kernel boundary adapter
    /// 5. Return result
    pub fn handle_command(
        &mut self,
        request: CommandRequest,
    ) -> ControlPlaneResult<CommandResponse> {
        // Step 1: Validate authority
        self.validate_authority(&request)?;

        // Step 2: Check if confirmation is required
        if request.command.requires_confirmation() {
            return self.handle_confirmable_command(request);
        }

        // Step 3: Execute non-confirmable command directly
        self.execute_command(&request)
    }

    /// Validate authority for a command.
    fn validate_authority(&self, request: &CommandRequest) -> ControlPlaneResult<()> {
        // Per PHASE7_AUTHORITY_MODEL.md §3.2:
        // Operator authority is required for all state mutation.
        if request.command.is_mutating() && !request.authority.can_mutate() {
            return Err(ControlPlaneError::insufficient_authority(
                "OPERATOR",
                &request.authority.level.to_string(),
            ));
        }
        Ok(())
    }

    /// Handle a command that requires confirmation.
    fn handle_confirmable_command(
        &mut self,
        request: CommandRequest,
    ) -> ControlPlaneResult<CommandResponse> {
        let command_name = request.command.command_name();
        let target_id = self.extract_target_id(&request.command);

        // Check if confirmation token is provided
        match request.confirmation_token {
            None => {
                // Request confirmation
                let token = self
                    .confirmation
                    .request_confirmation(command_name, target_id);
                Ok(CommandResponse::awaiting_confirmation(
                    request.request_id,
                    command_name,
                    token.id(),
                ))
            }
            Some(token_id) => {
                // Validate and consume confirmation
                match self.confirmation.confirm(token_id, command_name, target_id) {
                    ConfirmationResult::Proceed { .. } => {
                        // Execute the command
                        self.execute_command(&request)
                    }
                    ConfirmationResult::Abort { reason } => {
                        Err(ControlPlaneError::missing_confirmation(&reason))
                    }
                }
            }
        }
    }

    /// Extract target ID from a command.
    fn extract_target_id(&self, command: &ControlPlaneCommand) -> Option<Uuid> {
        match command {
            ControlPlaneCommand::Inspection(InspectionCommand::InspectNode { node_id }) => {
                Some(*node_id)
            }
            ControlPlaneCommand::Control(cmd) => Some(cmd.target_id()),
            _ => None,
        }
    }

    /// Execute a command against the kernel.
    ///
    /// Per PHASE7_INVARIANTS.md §P7-E1:
    /// Each operator command MUST correspond to exactly one kernel action.
    fn execute_command(&self, request: &CommandRequest) -> ControlPlaneResult<CommandResponse> {
        match &request.command {
            ControlPlaneCommand::Inspection(cmd) => {
                self.execute_inspection(request.request_id, cmd)
            }
            ControlPlaneCommand::Diagnostic(cmd) => {
                self.execute_diagnostic(request.request_id, cmd)
            }
            ControlPlaneCommand::Control(cmd) => self.execute_control(request.request_id, cmd),
        }
    }

    // =========================================================================
    // INSPECTION COMMANDS
    // =========================================================================

    fn execute_inspection(
        &self,
        request_id: Uuid,
        cmd: &InspectionCommand,
    ) -> ControlPlaneResult<CommandResponse> {
        match cmd {
            InspectionCommand::InspectClusterState => {
                // TODO: Call actual kernel to get cluster state
                let state = ClusterState {
                    cluster_id: None,
                    primary_id: None,
                    replicas: Vec::new(),
                    snapshot_time: SystemTime::now(),
                };
                Ok(CommandResponse::success(
                    request_id,
                    cmd.command_name(),
                    CommandResponseData::ClusterState(state),
                ))
            }
            InspectionCommand::InspectNode { node_id } => {
                // TODO: Call actual kernel to get node state
                let state = NodeState {
                    node_id: *node_id,
                    role: NodeRole::Unknown,
                    wal_position: 0,
                    health: NodeHealth::Unknown,
                    snapshot_time: SystemTime::now(),
                };
                Ok(CommandResponse::success(
                    request_id,
                    cmd.command_name(),
                    CommandResponseData::NodeState(state),
                ))
            }
            InspectionCommand::InspectReplicationStatus => {
                // TODO: Call actual kernel to get replication status
                let status = ReplicationStatus {
                    primary_id: None,
                    replicas: Vec::new(),
                    snapshot_time: SystemTime::now(),
                };
                Ok(CommandResponse::success(
                    request_id,
                    cmd.command_name(),
                    CommandResponseData::ReplicationStatus(status),
                ))
            }
            InspectionCommand::InspectPromotionState => {
                // TODO: Call actual kernel (promotion controller) to get state
                let state = PromotionStateView {
                    state: "Steady".to_string(),
                    pending_replica: None,
                    last_promotion: None,
                    snapshot_time: SystemTime::now(),
                };
                Ok(CommandResponse::success(
                    request_id,
                    cmd.command_name(),
                    CommandResponseData::PromotionState(state),
                ))
            }
        }
    }

    // =========================================================================
    // DIAGNOSTIC COMMANDS
    // =========================================================================

    fn execute_diagnostic(
        &self,
        request_id: Uuid,
        cmd: &DiagnosticCommand,
    ) -> ControlPlaneResult<CommandResponse> {
        match cmd {
            DiagnosticCommand::RunDiagnostics => {
                // TODO: Collect actual diagnostics from kernel
                let result = DiagnosticResult {
                    sections: Vec::new(),
                    collected_at: SystemTime::now(),
                };
                Ok(CommandResponse::success(
                    request_id,
                    cmd.command_name(),
                    CommandResponseData::Diagnostics(result),
                ))
            }
            DiagnosticCommand::InspectWal => {
                // TODO: Call actual WAL subsystem
                let info = WalInfo {
                    current_position: 0,
                    oldest_position: 0,
                    size_bytes: 0,
                    snapshot_time: SystemTime::now(),
                };
                Ok(CommandResponse::success(
                    request_id,
                    cmd.command_name(),
                    CommandResponseData::WalInfo(info),
                ))
            }
            DiagnosticCommand::InspectSnapshots => {
                // TODO: Call actual snapshot subsystem
                let info = SnapshotInfo {
                    snapshots: Vec::new(),
                    checkpoints: Vec::new(),
                    snapshot_time: SystemTime::now(),
                };
                Ok(CommandResponse::success(
                    request_id,
                    cmd.command_name(),
                    CommandResponseData::SnapshtoInfo(info),
                ))
            }
        }
    }

    // =========================================================================
    // CONTROL COMMANDS
    // =========================================================================

    fn execute_control(
        &self,
        request_id: Uuid,
        cmd: &ControlCommand,
    ) -> ControlPlaneResult<CommandResponse> {
        match cmd {
            ControlCommand::RequestPromotion { replica_id, reason } => {
                // TODO: Call actual promotion controller
                // Per PHASE7_COMMAND_MODEL.md §6.1:
                // Maps to promotion state machine
                let result = PromotionResultData {
                    replica_id: *replica_id,
                    success: false,
                    new_role: None,
                    explanation: "Promotion controller integration pending".to_string(),
                };
                Ok(CommandResponse::success(
                    request_id,
                    cmd.command_name(),
                    CommandResponseData::PromotionResult(result),
                ))
            }
            ControlCommand::RequestDemotion { node_id, reason } => {
                // TODO: Call actual demotion logic
                let result = PromotionResultData {
                    replica_id: *node_id,
                    success: false,
                    new_role: None,
                    explanation: "Demotion integration pending".to_string(),
                };
                Ok(CommandResponse::success(
                    request_id,
                    cmd.command_name(),
                    CommandResponseData::PromotionResult(result),
                ))
            }
            ControlCommand::ForcePromotion {
                replica_id,
                reason,
                acknowledged_risks,
            } => {
                // TODO: Call actual force promotion with override flag
                let result = PromotionResultData {
                    replica_id: *replica_id,
                    success: false,
                    new_role: None,
                    explanation: "Force promotion integration pending".to_string(),
                };
                Ok(CommandResponse::success(
                    request_id,
                    cmd.command_name(),
                    CommandResponseData::PromotionResult(result),
                ))
            }
        }
    }

    /// Request confirmation for a command (without executing).
    pub fn request_confirmation(&mut self, command: &ControlPlaneCommand) -> ConfirmationToken {
        let command_name = command.command_name();
        let target_id = self.extract_target_id(command);
        self.confirmation
            .request_confirmation(command_name, target_id)
    }

    /// Reject a pending confirmation.
    pub fn reject_confirmation(&mut self, token_id: Uuid) {
        self.confirmation.reject(token_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dx::api::control_plane::authority::AuthorityContext;

    #[test]
    fn test_inspection_no_confirmation() {
        let mut handler = ControlPlaneHandler::new();
        let cmd = ControlPlaneCommand::Inspection(InspectionCommand::InspectClusterState);
        let request = CommandRequest::new(cmd, AuthorityContext::observer());

        let response = handler.handle_command(request).unwrap();
        assert_eq!(response.outcome, CommandOutcome::Success);
    }

    #[test]
    fn test_control_requires_confirmation() {
        let mut handler = ControlPlaneHandler::new();
        let cmd = ControlPlaneCommand::Control(ControlCommand::RequestPromotion {
            replica_id: Uuid::new_v4(),
            reason: None,
        });
        let request = CommandRequest::new(cmd, AuthorityContext::operator());

        let response = handler.handle_command(request).unwrap();
        assert_eq!(response.outcome, CommandOutcome::AwaitingConfirmation);
        assert!(response.confirmation_token.is_some());
    }

    #[test]
    fn test_control_with_confirmation() {
        let mut handler = ControlPlaneHandler::new();
        let replica_id = Uuid::new_v4();
        let cmd = ControlPlaneCommand::Control(ControlCommand::RequestPromotion {
            replica_id,
            reason: None,
        });

        // First request - get confirmation token
        let request1 = CommandRequest::new(cmd.clone(), AuthorityContext::operator());
        let response1 = handler.handle_command(request1).unwrap();
        let token_id = response1.confirmation_token.unwrap();

        // Second request - with confirmation token
        let request2 =
            CommandRequest::new(cmd, AuthorityContext::operator()).with_confirmation(token_id);
        let response2 = handler.handle_command(request2).unwrap();

        assert_eq!(response2.outcome, CommandOutcome::Success);
    }

    #[test]
    fn test_insufficient_authority_rejected() {
        let mut handler = ControlPlaneHandler::new();
        let cmd = ControlPlaneCommand::Control(ControlCommand::RequestPromotion {
            replica_id: Uuid::new_v4(),
            reason: None,
        });

        // Observer cannot execute mutating commands
        let request = CommandRequest::new(cmd, AuthorityContext::observer());
        let result = handler.handle_command(request);

        assert!(result.is_err());
    }
}
