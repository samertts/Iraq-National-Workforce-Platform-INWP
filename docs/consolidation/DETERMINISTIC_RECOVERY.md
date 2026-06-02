# Deterministic Recovery Procedures

## Replay Recovery
1. Verify last known good checkpoint hash
2. Isolate domain from federation
3. Replay events from checkpoint (deterministic — MUST produce byte-identical state)
4. Verify state consistency hash against pre-failure snapshot
5. Re-establish federation trust via mTLS attestation
6. Execute CRDT reconciliation with federation peers
7. Verify global state convergence

## Partition Healing
1. Detect partition boundaries via version vector comparison
2. Quantify divergence: list all keys with differing version vectors
3. Execute CRDT merge for each divergent key
4. Verify unified state: all nodes MUST converge to identical state hash
5. Re-establish direct routing between previously partitioned nodes

## Topology Repair
1. Discover orphaned nodes (nodes with no parent or unreachable parent)
2. Reattach orphans to nearest available regional hub
3. Verify routing table: no cycles, no gaps, all nodes reachable
4. Propagate topology update across federation

## Federation Healing
1. Isolate unhealthy domain (prevent further divergence)
2. Verify domain state integrity via checkpoint hash
3. Execute full Merkle sync with federation quorum
4. Re-establish trust: verify credentials, replay history, validate signatures

## Sovereign Recovery Escalation
- Level 1: Automatic healing via HealingEngine
- Level 2: Governance-approved healing plan
- Level 3: Human operator intervention with governance override
- Level 4: Sovereign authority manual recovery
