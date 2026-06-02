# Sovereign Survivability Playbook

## Degradation Response
- **Normal**: All systems operational
- **Minor**: ≤2 failures — prioritize sync, maintain federation
- **Major**: ≤5 failures — preserve audit log, degrade non-critical
- **Critical**: ≤10 failures — keep identity only, isolate regions
- **Outage**: >10 failures — minimal survival mode, emergency broadcast

## Isolation Management
- Zone enters isolation → create IsolationPlan with appropriate AutonomyLevel
- Short isolation (<1hr): LimitedConnectivity — cache with TTL
- Medium isolation (<24hr): AutonomousOperation — full local replica
- Long isolation (>7d): SovereignIsolation — sovereign data only
- Emergency: EmergencyMode — minimal survival operations

## Reconnection Protocol
1. Secure connection to federation parent
2. Authenticate + verify trust credentials
3. Exchange version vectors
4. CRDT reconciliation for divergent state
5. Verify global consistency
6. Resume normal sync schedule

## Blast Radius Containment
- Identify incident epicenter
- Compute dependency impact via KnowledgeGraph
- Isolate boundary: affected domains contained, healthy domains protected
- Execute healing plan within boundary
- Verify containment before reconnecting

## Federation Collapse Recovery
1. Identify surviving quorum (≥3 sovereign nodes)
2. Select authoritative state (highest checkpoint count wins)
3. Synchronize all surviving nodes to authoritative state
4. Rebuild federation topology from surviving quorum
5. Reconnect orphaned domains incrementally
6. Verify global state convergence before restoring full operations
