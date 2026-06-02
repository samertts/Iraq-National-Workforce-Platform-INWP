# INWP Sovereign Architecture Standards

## 1. Repository Architecture

- Every bounded context owns exactly one crate or module
- Cross-context communication occurs ONLY through defined interfaces
- Anti-corruption layers are mandatory between all bounded contexts
- No crate may depend on another crate's internal implementation
- All public APIs must be versioned and governed by the schema registry

## 2. Bounded Contexts

- Each context must register in the ArchitectureRegistry
- Context ownership must be documented in the ownership registry
- Context maps must define relationships (upstream/downstream, partnership, ACL)
- Shared kernels require explicit governance approval
- Context isolation must be validated by BoundedContextValidator

## 3. Protobuf Contracts

- All event schemas must be registered in the EventRegistry
- Schema changes must be validated for backward/forward compatibility
- Field numbers are permanent — never reuse a retired field number
- Reserved fields must be declared in the proto definition
- Package names must follow the pattern: `inwp.{domain}.{context}.{version}`

## 4. Event Naming

- Events follow past-tense verb-noun convention: `EmployeeHired`, `PayrollProcessed`
- Event names must be unique across the entire federation
- Event schemas must include: event_id, timestamp, version, domain_id, partition_key
- All events must carry a cryptographic signature from the originating node
- Event lineage must be recorded in the EventRegistry lineage tracker

## 5. Replay Safety

- All event handlers must be deterministic (same input = same output every time)
- Non-deterministic operations (time, random, external calls) are FORBIDDEN in replay paths
- Every event stream must have a checksum for replay verification
- Replay must produce byte-identical state across all runs
- Replay governance validates determinism before allowing any stream registration

## 6. Synchronization Contracts

- All sync operations follow the 5-phase protocol: Discovery → Merkle Exchange → Delta Transfer → Reconciliation → Commitment
- No node may bypass the sync protocol for state exchange
- Sync contracts must be registered in the sync-engine protocol layer
- Merkle tree roots must be exchanged before any data transfer
- All sync operations produce signed receipts

## 7. Federation Protocols

- Federation boundaries require explicit SovereigntyAgreement registration
- Cross-domain event routing requires federated domain registration and boundary policy
- No event may cross a sovereignty boundary without audit logging
- Federation topology must be validated by TopologyManager
- Federation governance checks all cross-domain interactions

## 8. Observability

- All components must emit structured JSON logs
- All components must expose Prometheus metrics
- All distributed operations must have OpenTelemetry tracing spans
- Health probes must be exposed at /health and /ready
- Metrics: batch counts, conflict rates, queue depth, recovery time, trust scores

## 9. Migration Safety

- All migrations must be registered in the MigrationRegistry
- Migrations must be replay-safe (produce same state on replay)
- Rollback scripts are mandatory for all migrations
- Migration checksums must be verified before execution
- No more than 3 migrations per deployment without governance override

## 10. Recovery Procedures

- Every domain must maintain continuity checkpoints via SurvivabilityEngine
- Recovery follows the orchestrated plan: Isolate → Verify Checkpoint → Replay → Verify Consistency → Reconnect
- Split-brain resolution requires federation governance approval
- Trust re-establishment is the final step in any recovery procedure
- Recovery tests must be executed via RecoveryTestEngine before any major deployment

## 11. Testing Strategy

- Unit tests: all core domain logic (Merkle tree, CRDTs, reconciliation)
- Integration tests: sync roundtrips with PostgreSQL + NATS
- Chaos tests: partition, corruption, Byzantine behavior via ChaosEngine
- Recovery tests: disaster scenarios via RecoveryTestEngine
- Replay tests: determinism verification via ReplaySimulator
- All chaos experiments require governance approval

## 12. Deployment Topology

- Deployments follow the topology: Sovereign → National Hub → Regional Hub → Ministry Relay → Institution → Edge
- All deployments require governance policy validation
- Canary deployments are mandatory for all infrastructure changes
- Deployment policies are enforced by DeploymentGovernance
- Rollback plans must exist for every deployment

## 13. Infrastructure Provisioning

- Terraform is the sole infrastructure provisioning tool
- All infrastructure changes must be validated by deployment governance
- Docker images must be signed with Cosign
- SBOMs must be generated for every build (Syft)
- Vulnerability scanning is mandatory (Grype, Trivy)

## 14. Security Policy

- Zero-trust network architecture — no implicit trust between any nodes
- All gRPC connections require mTLS
- All events require Ed25519 signatures
- Trust scores are maintained per node — scores below 0.3 trigger automatic isolation
- Security events are logged in the immutable security audit chain
- HSM integration is required for sovereign-level cryptographic operations

## 15. Operational Governance

- ControlPlane maintains the operational topology and deployment queue
- GovernanceEngine evaluates all policy rules before any infrastructure change
- Enforcement modes: LogOnly → WarnOnly → Enforce → StrictEnforce
- All governance decisions produce auditable entries
- DeploymentOrchestrator manages rollout strategy and cooldown periods
