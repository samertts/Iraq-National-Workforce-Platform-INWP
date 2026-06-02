# INWP Synchronization Topology

> Complete synchronization architecture: conflict resolution, event replay, delta sync, eventual consistency, retry queues, partition tolerance, and regional recovery.

---

## 1. Sync Topology Overview

### 1.1 Three-Tier Sync Mesh

```
                    +---------------------------+
                    |    NATIONAL SYNC HUB      |
                    |    (Baghdad, 3 replicas)  |
                    |                           |
                    |  - Global event routing   |
                    |  - Cross-ministry sync    |
                    |  - Merkle root publishing |
                    |  - Dead-letter queue mgmt |
                    |  - Schema registry master |
                    +----------+----------------+
                               |
           +-------------------+-------------------+
           |                   |                   |
+----------v--------+ +------v---------+ +-------v---------+
| REGIONAL RELAY 1  | | REGIONAL RELAY | | REGIONAL RELAY  |
| (Baghdad)         | | (Basra)        | | (Mosul)         |
|                   | |                | |                 |
| - Store & forward | | - Store & fwd  | | - Store & fwd   |
| - Bandwidth shape | | - BW shape     | | - BW shape      |
| - zstd compress   | | - zstd         | | - zstd          |
| - Conflict proxy  | | - Conflict     | | - Conflict      |
+----------+--------+ +-------+--------+ +--------+--------+
           |                    |                  |
     +-----+-----+       +-----+-----+       +----+-----+
     |     |     |       |     |     |       |    |     |
+----v-+ +-v---+-+   +--v---+ +-v---+-+   +-v----+ +---v--+
|Edge 1| |Edge2|...|  |Edge N| |...  |   |Edge X| |Edge Y |
|      |<----->|   |  |      |       |   |      |<------>|
|LAN   | |LAN  |   |  |LAN   |       |   |LAN   | |LAN   |
|Mesh  | |Mesh |   |  |Mesh  |       |   |Mesh  | |Mesh  |
+------+ +-----+   |  +------+       |   +------+ +------+
                    |                 |
    Peer-to-Peer sync on LAN (mDNS + gRPC)
    No hub mediation for intra-site sync
```

### 1.2 Sync Modes by Tier

| Tier | Sync Mode | Protocol | Frequency | Direction |
|---|---|---|---|---|
| Edge -> Regional | Upload (push) | HTTPS + mTLS + zstd | Continuous + batch | Bidirectional |
| Regional -> National | Relay (push/pull) | NATS Leaf + HTTPS | Near real-time | Bidirectional |
| Edge -> Edge (LAN) | Peer mesh | mDNS + gRPC + mTLS | On discovery | Bidirectional |
| Mobile -> Edge/Cloud | Device sync | HTTPS + cert pinning | On connect | Upload primarily |
| National -> DR | Replication | PostgreSQL logical | Continuous streaming | Unidirectional |

---

## 2. Conflict Resolution

### 2.1 Resolution Strategies

| Strategy | Description | Use Cases |
|---|---|---|
| **LWW (Last Writer Wins)** | Highest `local_timestamp` wins. Both records preserved in audit. | Clock events, device trust, notifications |
| **Ministry Author Wins** | Hub/ministry record overrides edge record. | Shift definitions, policies, user profiles |
| **Service-Side Merge** | Domain-aware merge logic (not blind override). | Leave balance (PN-Counter), accrual math |
| **Additive Merge (G-Set)** | Union of all values. No removal. | Role assignments, permission sets |
| **Manual Resolution** | Escalated to admin dashboard for human decision. | Leave request conflicts, data disputes |

### 2.2 Per-Entity Conflict Resolution Matrix

| Entity | Strat | Auto | Overridable | Details |
|---|---|---|---|---|
| ClockEvent | LWW + Dedup | Yes | No | Immutable; dedup by event_id |
| AttendanceException | LWW | Yes | Yes | Latest justification wins |
| Shift | Ministry Author | Yes | No | HR authoritative |
| AttendancePolicy | Ministry Author | Yes | No | One active per site |
| LeaveRequest | Service Merge + Manual | Partial | Yes | State machine + human judgment |
| LeaveBalance | PN-Counter CRDT | Yes | No | Commutative merge required |
| AccrualPolicy | LWW | Yes | Yes | Latest policy wins |
| UserProfile | LWW per field | Yes | Yes | Per-field independent |
| RoleAssignment | Additive Merge | Yes | Yes | Union of assignments |
| DeviceTrust | LWW | Yes | No | Computed score |
| LedgerEntry | Append-only | N/A | N/A | Immutable by design |
| PolicyDefinition | Ministry Author | Yes | N/A | Authoritative source |

### 2.3 Conflict Lifecycle

```
1. DETECTION
   Sync engine compares record versions
   Local.version != Remote.version AND Local.timestamp != Remote.timestamp
   -> Conflict detected

2. CLASSIFICATION
   Auto-resolvable (matches known strategy)
   OR Manual (ambiguous, needs human)

3. RESOLUTION (Auto)
   Apply strategy: LWW / Ministry Author / CRDT Merge
   Resolution event published: sync.v1.conflict.resolved
   Both nodes accept resolution

4. ESCALATION (Manual)
   Conflict persisted to sync_conflicts table
   Alert sent to admin dashboard
   Admin: choose local, remote, or custom merge
   Admin resolution published as event

5. COMMITMENT
   Resolved record applied to both nodes
   New merkle root computed
   Checkpoint advanced
```

### 2.4 Conflict Queue Model

```sql
CREATE TABLE sync_conflicts (
    conflict_id      UUID PRIMARY KEY,
    sync_id          UUID REFERENCES sync_batch_log(sync_id),
    partition_key    TEXT NOT NULL,
    record_id        TEXT NOT NULL,
    record_type      TEXT NOT NULL,
    local_version    JSONB NOT NULL,
    remote_version   JSONB NOT NULL,
    strategy         TEXT NOT NULL,
    status           TEXT DEFAULT 'open',
    auto_resolvable  BOOLEAN DEFAULT false,
    escalated_at     TIMESTAMPTZ,
    resolved_by      UUID,
    resolution       TEXT,
    resolved_at      TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

---

## 3. Event Replay

### 3.1 Replay Triggers

| Trigger | Scope | Mechanism |
|---|---|---|
| New node joining mesh | All events since epoch | Full event log replay from trusted peer |
| Node recovery after data loss | Events since last checkpoint | Partial replay from sync checkpoints |
| Data corruption detected | Events in corrupted range | Replay from last verified seal |
| Schema migration | Events requiring transformation | Replay through schema converter |
| Audit verification | All events | Full replay to verify hash chain |
| Analytics rebuild | Events in time range | Replay to rebuild materialized views |

### 3.2 Replay Protocol

```
Phase 1: Request
  Node A: "I need replay from checkpoint X to current"
  Node B: "Acknowledged. Preparing replay stream..."

Phase 2: Stream
  Node B sends events in ordered batches (1000 events/batch)
  Each batch includes: {batch_seq, events[], merkle_proof}
  Node A verifies merkle proof for each batch

Phase 3: Apply
  Node A applies each event through normal event pipeline
  Idempotent: already-applied events detected by event_id
  Events trigger standard side effects (state engine, etc.)

Phase 4: Verify
  After replay completes, compare merkle root with source
  If mismatch: re-run delta sync to identify divergence
  If match: commit checkpoint, resume normal operation
```

### 3.3 Replay Sizing

```
Full replay from epoch:
  Events: ~100M (5M employees x 20 events/year x 1 year)
  Data: ~500GB (5KB avg event size)
  Time: ~2-4 hours (on 1Gbps link with zstd compression)

Partial replay (1 month):
  Events: ~8M
  Data: ~40GB
  Time: ~10-20 minutes
```

---

## 4. Delta Sync

### 4.1 Delta Computation Algorithm

```
FUNCTION delta_sync(local_node, remote_node, partition):
    // Phase 1: Exchange merkle roots
    local_root  = local_node.get_merkle_root(partition)
    remote_root = remote_node.get_merkle_root(partition)

    IF local_root == remote_root:
        RETURN no_changes_needed

    // Phase 2: Recursive merkle diff
    diff_keys = reconcile(local_root, remote_root, depth=0)
    // Returns set of record IDs that differ

    // Phase 3: Fetch deltas
    local_deltas  = local_node.get_records(diff_keys)
    remote_deltas = remote_node.get_records(diff_keys)

    // Phase 4: Transfer + apply
    FOR key IN diff_keys:
        IF key NOT IN local_deltas:
            local_node.apply_create(remote_deltas[key])
        ELIF key NOT IN remote_deltas:
            remote_node.apply_create(local_deltas[key])
        ELSE:
            resolve_conflict(key, local_deltas[key], remote_deltas[key])

    // Phase 5: Compute new root, commit
    new_root = compute_merkle_root(partition)
    sign_commitment(partition, new_root, remote_node)
    advance_checkpoint(partition, new_root)
```

### 4.2 Delta Compression

```
- Full sync (first time):  zstd level 19 (max compression)
- Delta sync (subsequent): zstd level 3 (speed optimized)
- Chunk size: 1MB per batch with checkpoint/resume
- Dedup: skip records with matching event_id
- Bloom filters: quick "do you have record X?" check
  - Reduces round-trips by 60%+ for reconciliation
  - False positive rate: 1% (configurable)

Bandwidth savings:
  - Full sync baseline: 100% (all records)
  - After first sync: ~5% of data transferred
  - With compression: ~1-2% of raw data size
```

---

## 5. Eventual Consistency

### 5.1 Consistency Model

INWP uses **Strong Eventual Consistency (SEC)** via CRDT-inspired merge strategies:

| Property | Guarantee | Mechanism |
|---|---|---|
| **Read your writes** | Yes (local node) | Local PG is immediate source of truth |
| **Monotonic reads** | Yes (per node) | Local reads don't regress |
| **Eventual convergence** | Yes (across nodes) | CRDT merge + merkle reconciliation |
| **No conflicts** | Per strategy | CRDT for balances, LWW for events |
| **Bounded convergence** | Configurable | Sync priority + scheduling ensures timely convergence |

### 5.2 Convergence Timeline

```
Edge -> Edge (LAN mesh):
  - Same site: <1 second (mDNS + gRPC)
  - Adjacent sites: <5 seconds

Edge -> Regional Hub:
  - Connected: <30 seconds
  - 4G backup: <2 minutes
  - Scheduled bulk: configurable (e.g., 02:00-05:00)

Regional -> National Hub:
  - Near real-time: <1 minute
  - Bulk: <15 minutes

National -> DR Site:
  - Streaming: <5 seconds
```

### 5.3 Read Consistency Levels

| Level | Latency | Staleness | Use Case |
|---|---|---|---|
| LOCAL | <5ms | Up to configured TTL | Employee dashboard, clock-in validation |
| REGIONAL | <100ms | Up to 30s | Leave balance, approval status |
| NATIONAL | <500ms | Strong | Compliance queries, audit reports |
| CONSENSUS | <2s | Zero | Financial reconciliation, payroll |

---

## 6. Retry Queues

### 6.1 Retry Architecture

```
Event -> Sync Queue
    |
    +--[SUCCESS]--> Sync Batch -> Checkpoint Advanced
    |
    +--[RETRYABLE ERROR]--> Retry Queue
    |       |
    |       +-- Exponential backoff
    |       +-- Max retries: 10 (configurable per event type)
    |       +-- After max: Dead Letter Queue
    |
    +--[FATAL ERROR]--> Dead Letter Queue
            |
            +-- Admin notification
            +-- Manual investigation
            +-- Replay or discard
```

### 6.2 Retry Backoff Strategy

```
Backoff: Exponential with jitter

Formula: delay = min(base * 2^retry + random(0, jitter), max_delay)

Defaults:
  Base:       1 second
  Max delay:  30 minutes
  Jitter:     1 second
  Max retries: 10

Per-category overrides:
  Attendance:  base=1s, max_delay=5min, max_retries=20
  Identity:    base=5s, max_delay=30min, max_retries=10
  Leave:       base=10s, max_delay=1h, max_retries=10
  Sync meta:   base=500ms, max_delay=2min, max_retries=50
```

### 6.3 Dead Letter Queue

```sql
CREATE TABLE sync_dead_letter (
    dlq_id          UUID PRIMARY KEY,
    sync_id         UUID,
    event_id        UUID NOT NULL,
    partition_key   TEXT NOT NULL,
    payload         BYTEA NOT NULL,
    error_message   TEXT NOT NULL,
    error_code      TEXT NOT NULL,
    retry_count     INT DEFAULT 0,
    failed_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_retry_at   TIMESTAMPTZ,
    status          TEXT DEFAULT 'pending_review',
    reviewed_by     UUID,
    resolution      TEXT,
    resolved_at     TIMESTAMPTZ
);
```

---

## 7. Partition Tolerance

### 7.1 Partition Scenarios

| Scenario | Behavior | Data Safety | Recovery |
|---|---|---|---|
| Edge isolated from region | Fully autonomous operation | All data preserved locally | Merkle sync on reconnect |
| Region isolated from national | Region operates independently | All data preserved regionally | Catch-up sync on reconnect |
| National DC offline | Regions operate independently | All data preserved | Logical replication catch-up |
| Asymmetric partition | Both sides accept writes | CRDT merge ensures convergence | Auto-resolution on reconnect |
| Partial partition (some services down) | Degraded operation per service | Healthy services continue | Restart/recover failed services |

### 7.2 Split-Brain Prevention

```
Design features that prevent split-brain:
1. No single master -- every node is authoritative for its own writes
2. CRDT merge ensures convergence regardless of partition duration
3. All events carry local_timestamp + node_id for ordering
4. Append-only audit prevents any destructive conflict
5. Row-level version vectors track divergence per record

If both sides modified the same record:
  - Conflict detected on reconnect (version mismatch)
  - Resolution strategy applied per entity type
  - Admin notified of manual-resolution conflicts
  - No data loss -- both versions preserved in audit
```

### 7.3 Quarantine Zone

```
Records that cannot be resolved automatically enter quarantine:
- Quarantine table: sync_conflicts
- Admin must review and resolve
- System sends alert: "X conflicts pending resolution"
- Quarantined records excluded from reports until resolved
- Time limit: 30 days auto-escalate to national admin
```

---

## 8. Regional Recovery

### 8.1 Recovery State Machine

```
         +-----------+ 
         | ONLINE    | <------ Heartbeat OK
         +-----+-----+
               | 3 missed heartbeats
               v
         +-----------+ 
         | SUSPECTED | <--- Investigate connectivity
         +-----+-----+
               | 3 more missed heartbeats (total 6)
               v
         +-----------+ 
         | OFFLINE   | <--- Alert triggered, region autonomous
         +-----+-----+
               | Reconnect attempt succeeds
               v
         +-----------+ 
         | RECOVERING| <--- Catch-up sync in progress
         +-----+-----+
               | Sync complete, queues drained
               v
         +-----------+ 
         | ONLINE    | <--- Normal operation resumed
         +-----------+
```

### 8.2 Recovery Steps

```
Step 1: Detection
  - Heartbeat timeout (no event for 180s = 3 missed)
  - Alert: region-{name}-offline
  - Regional hub enters autonomous mode

Step 2: Isolation
  - All services continue with local data
  - Outbound event queue accumulates
  - Edge nodes continue syncing to regional hub (LAN)

Step 3: Reconnection Attempt
  - Exponential backoff: 1s / 5s / 30s / 2min / 10min / 30min
  - mTLS handshake on reconnect
  - Certificate validity check
  - Schema version negotiation

Step 4: Catch-up Sync
  - Exchange last checkpoint per partition
  - Merkle tree comparison
  - Delta transfer (bidirectional)
  - Conflict resolution (auto + manual queue)

Step 5: Normalization
  - Sync queues drained
  - Dead-letter queue reviewed
  - Heartbeat resumes (60s interval)
  - Alert resolved

Step 6: Verification
  - Compare merkle roots with national hub
  - Verify sync_batch_log continuity
  - Generate recovery report for admin
```

### 8.3 Recovery Timeline Estimates

```
Duration          Catch-up Time      Data Volume
 1 hour           ~2 minutes         ~10MB
 1 day            ~30 minutes        ~500MB
 1 week           ~4 hours           ~3.5GB
 1 month          ~24 hours          ~15GB
 6 months         ~1 week (manual)   ~90GB

Factors affecting recovery time:
  - WAN bandwidth (10 Mbps / 100 Mbps / 1 Gbps)
  - Conflict volume (auto vs manual)
  - Schema changes (may require transformation)
  - Available system resources during catch-up

Capacity planning:
  - Store 30+ days of events at regional hub
  - Compression target: 10:1 (raw:stored)
  - Typical daily event volume per region: ~500MB
```

---

## 9. Sync Data Model

```sql
-- Node registry (every sync participant)
CREATE TABLE sync.node_registry (
    node_id         UUID PRIMARY KEY,
    node_type       TEXT NOT NULL,      -- 'edge', 'regional_relay', 'national_hub'
    node_name       TEXT NOT NULL,
    ministry_id     UUID,
    site_id         UUID,
    region          TEXT,
    certificate_serial TEXT,
    public_key      BYTEA NOT NULL,
    address         TEXT,               -- IP/hostname for WAN, mDNS for LAN
    port            INT,
    capabilities    JSONB,              -- Schema versions, supported entities
    status          TEXT DEFAULT 'online',
    last_heartbeat  TIMESTAMPTZ,
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata        JSONB
);

-- Sync checkpoint (per partition, per node)
CREATE TABLE sync.sync_checkpoint (
    node_id         UUID NOT NULL REFERENCES sync.node_registry(node_id),
    partition_key   TEXT NOT NULL,      -- '{ministry}/{entity}/{time_bucket}'
    merkle_root     BYTEA NOT NULL,
    last_sync_at    TIMESTAMPTZ,
    synced_events   BIGINT DEFAULT 0,
    last_error      TEXT,
    PRIMARY KEY (node_id, partition_key)
);

-- Sync batch log (immutable receipts)
CREATE TABLE sync.sync_batch_log (
    sync_id          UUID PRIMARY KEY,
    source_node      UUID NOT NULL,
    target_node      UUID NOT NULL,
    partition_key    TEXT NOT NULL,
    direction        TEXT NOT NULL,     -- 'upload', 'download', 'bidirectional'
    events_count     INT NOT NULL,
    bytes_transferred BIGINT NOT NULL,
    conflict_count   INT DEFAULT 0,
    conflicts_auto   INT DEFAULT 0,
    conflicts_manual INT DEFAULT 0,
    local_merkle     BYTEA NOT NULL,
    remote_merkle    BYTEA NOT NULL,
    source_sig       BYTEA NOT NULL,
    target_sig       BYTEA NOT NULL,
    compression_ratio REAL DEFAULT 1.0,
    duration_ms      INT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Sync queue (pending events)
CREATE TABLE sync.sync_queue (
    queue_id         UUID PRIMARY KEY,
    node_id          UUID NOT NULL,
    partition_key    TEXT NOT NULL,
    event_id         UUID NOT NULL,
    event_type       TEXT NOT NULL,
    payload          BYTEA NOT NULL,
    priority         INT DEFAULT 5,
    status           TEXT DEFAULT 'pending',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    next_retry_at    TIMESTAMPTZ,
    retry_count      INT DEFAULT 0,
    last_error       TEXT
);

CREATE INDEX idx_sync_queue_priority ON sync.sync_queue(priority, created_at);
CREATE INDEX idx_sync_queue_node     ON sync.sync_queue(node_id, status);
CREATE INDEX idx_sync_queue_retry    ON sync.sync_queue(next_retry_at);
```
