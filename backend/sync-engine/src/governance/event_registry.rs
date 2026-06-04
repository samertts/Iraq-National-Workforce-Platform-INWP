use crate::error::SyncResult;
use std::collections::HashMap;
use tracing::info;

/// Sovereign event registry — all event contracts are versioned, lineage-tracked, and governance-enforced
pub struct EventRegistry {
    contracts: HashMap<String, EventContract>,
    lineage: Vec<EventLineageEntry>,
    deprecation_queue: Vec<DeprecationRequest>,
}

#[derive(Debug, Clone)]
pub struct EventContract {
    pub contract_id: String,
    pub event_name: String,
    pub domain: String,
    pub version: semver::Version,
    pub schema_fingerprint: Vec<u8>,
    pub proto_definition: String,
    pub event_type: EventType,
    pub status: ContractStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub superseded_by: Option<String>,
    pub audit_hash: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    DomainEvent,
    Command,
    SyncEvent,
    FederationEvent,
    GovernanceEvent,
    SecurityEvent,
    SovereigntyEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractStatus {
    Active,
    Deprecated,
    Sunset,
    Retired,
    Superseded,
}

#[derive(Debug, Clone)]
pub struct EventLineageEntry {
    pub event_name: String,
    pub from_version: String,
    pub to_version: String,
    pub change_type: LineageChangeType,
    pub migrated_at: chrono::DateTime<chrono::Utc>,
    pub migration_proof: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineageChangeType {
    FieldAdded,
    FieldRemoved,
    FieldRenamed,
    FieldTypeChanged,
    FieldMadeOptional,
    FieldMadeRequired,
    EventSplit,
    EventMerged,
    EventRenamed,
    SchemaRestructured,
}

#[derive(Debug, Clone)]
pub struct DeprecationRequest {
    pub contract_id: String,
    pub reason: String,
    pub sunset_date: chrono::DateTime<chrono::Utc>,
    pub replacement_contract: Option<String>,
    pub approved: bool,
    pub approved_by: Option<String>,
}

#[derive(Debug)]
pub struct ProtocolBufferValidation {
    pub contract_id: String,
    pub valid: bool,
    pub wire_format_compatible: bool,
    pub field_number_collisions: Vec<String>,
    pub reserved_fields_respected: bool,
    pub package_namespace_valid: bool,
}

impl EventRegistry {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
            lineage: Vec::new(),
            deprecation_queue: Vec::new(),
        }
    }

    pub fn register_contract(&mut self, contract: EventContract) {
        let id = contract.contract_id.clone();
        let key = format!("{}@{}", contract.event_name, contract.version);
        info!(
            contract = %id,
            event = %contract.event_name,
            version = %contract.version,
            "Event contract registered"
        );
        self.contracts.insert(key, contract);
    }

    pub fn record_lineage(&mut self, entry: EventLineageEntry) {
        info!(
            event = %entry.event_name,
            from = %entry.from_version,
            to = %entry.to_version,
            change = ?entry.change_type,
            "Event lineage recorded"
        );
        self.lineage.push(entry);
    }

    pub fn get_contract(
        &self,
        event_name: &str,
        version: &semver::Version,
    ) -> Option<&EventContract> {
        let key = format!("{}@{}", event_name, version);
        self.contracts.get(&key)
    }

    pub fn get_latest_contract(&self, event_name: &str) -> Option<&EventContract> {
        self.contracts
            .iter()
            .filter(|(k, _)| k.starts_with(event_name))
            .filter(|(_, c)| matches!(c.status, ContractStatus::Active))
            .max_by_key(|(_, c)| c.version.clone())
            .map(|(_, c)| c)
    }

    pub fn validate_protobuf(&self, contract_id: &str) -> ProtocolBufferValidation {
        let contract = self
            .contracts
            .values()
            .find(|c| c.contract_id == contract_id);
        match contract {
            Some(c) => ProtocolBufferValidation {
                contract_id: contract_id.to_string(),
                valid: true,
                wire_format_compatible: true,
                field_number_collisions: Vec::new(),
                reserved_fields_respected: true,
                package_namespace_valid: c.event_name.contains('.'),
            },
            None => ProtocolBufferValidation {
                contract_id: contract_id.to_string(),
                valid: false,
                wire_format_compatible: false,
                field_number_collisions: Vec::new(),
                reserved_fields_respected: true,
                package_namespace_valid: false,
            },
        }
    }

    pub fn submit_deprecation(&mut self, request: DeprecationRequest) {
        info!(
            contract = %request.contract_id,
            reason = %request.reason,
            "Deprecation request submitted"
        );
        self.deprecation_queue.push(request);
    }

    pub fn approve_deprecation(&mut self, contract_id: &str, approver: &str) -> SyncResult<()> {
        let request = self
            .deprecation_queue
            .iter_mut()
            .find(|r| r.contract_id == contract_id && !r.approved)
            .ok_or_else(|| {
                crate::error::SyncEngineError::Internal(format!(
                    "No pending deprecation request for '{}'",
                    contract_id
                ))
            })?;
        request.approved = true;
        request.approved_by = Some(approver.to_string());

        if let Some(contract) = self
            .contracts
            .values_mut()
            .find(|c| c.contract_id == contract_id)
        {
            contract.status = ContractStatus::Deprecated;
        }
        info!(contract = %contract_id, approved_by = %approver, "Deprecation approved");
        Ok(())
    }

    pub fn get_lineage(&self, event_name: &str) -> Vec<&EventLineageEntry> {
        self.lineage
            .iter()
            .filter(|e| e.event_name == event_name)
            .collect()
    }

    pub fn get_all_versions(&self, event_name: &str) -> Vec<&EventContract> {
        self.contracts
            .values()
            .filter(|c| c.event_name == event_name)
            .collect()
    }

    pub fn validate_replay_compatibility(&self, event_name: &str, from: &str, to: &str) -> bool {
        let from_ver = semver::Version::parse(from).ok();
        let to_ver = semver::Version::parse(to).ok();

        match (from_ver, to_ver) {
            (Some(f), Some(t)) if f <= t => {
                let from_contract = self
                    .contracts
                    .values()
                    .find(|c| c.event_name == event_name && c.version == f);
                let to_contract = self
                    .contracts
                    .values()
                    .find(|c| c.event_name == event_name && c.version == t);
                matches!((from_contract, to_contract), (Some(_), Some(_)))
            }
            _ => false,
        }
    }
}

impl Default for EventRegistry {
    fn default() -> Self {
        Self::new()
    }
}
