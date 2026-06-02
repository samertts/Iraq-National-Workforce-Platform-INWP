# INWP Sync Conflict Resolution Strategy

> Complete conflict resolution design for offline-first, distributed synchronization across edge, regional, and national tiers.

---

## Table of Contents

1. [Conflict Model](#1-conflict-model)
2. [Detection Mechanisms](#2-detection-mechanisms)
3. [Resolution Strategies](#3-resolution-strategies)
4. [Per-Entity Resolution Matrix](#4-per-entity-resolution-matrix)
5. [Resolution Lifecycle](#5-resolution-lifecycle)
6. [CRDT-Inspired Data Structures](#6-crdt-inspired-data-structures)
7. [Delete Handling](#7-delete-handling)
8. [Merge Semantics by Entity](#8-merge-semantics-by-entity)
9. [Manual Resolution Workflow](#9-manual-resolution-workflow)
10. [Edge Cases & Guarantees](#10-edge-cases--guarantees)

---

## 1. Conflict Model

### 1.1 What Constitutes a Conflict

A conflict occurs when two or more nodes independently modify the same logical record between sync cycles, and those modifications cannot be automatically reconciled without data loss.

**Formal definition:**

```
Given record R with causal history H(R):
  - Node A applies mutation M_A at time T_A
  - Node B applies mutation M_B at time T_B
  - Neither node has the other's mutation when applying its own
  - If M_A and M_B modify the same field(s) of R differently → CONFLICT
  - If M_A and M_B modify different fields of R → MERGEABLE (no conflict)
  - If M_A creates R and M_B deletes R → TOMBSTONE CONFLICT
  - If M_A deletes R and M_B modifies R → DELETE vs UPDATE CONFLICT
```

### 1.2 Conflict Classification

```
CONFLICT
├── VALUE CONFLICT
│   ├── Same-field divergent     // Both nodes changed the same field to different values
│   ├── Add-add                  // Both nodes created a record with the same logical key
│   └── Set-member               // Both nodes added/removed the same set member
│
├── DELETE CONFLICT
│   ├── Delete vs Update         // One node deleted, another modified
│   ├── Delete vs Delete         // Both deleted (technically consistent, but tombstone needed)
│   └── Recreate after delete    // One node deleted, another recreated with same ID
│
├── CONSTRAINT CONFLICT
│   ├── Referential integrity    // Node A references a record Node B deleted
│   ├── Unique constraint        // Two nodes assigned the same unique value to different records
│   └── Balance/quantity         // Leave balance updates conflict (financial accuracy)
│
└── SCHEMA CONFLICT
    ├── Field removed            // One node upgraded schema, other hasn't
    └── Field type changed       // Schema drift between nodes
```

### 1.3 Conflict Frequency Estimation

| Data Type | Expected Conflict Rate | Rationale |
|---|---|---|
| Clock events (clock-in/out) | < 0.1% | Append-only, immutable; conflicts arise only from device clock skew |
| Leave requests | < 1% | Single-actor writes; conflicts from concurrent admin edits |
| Leave balances | ~0% (design goal) | Pessimistic locking + service-side merge prevents conflicts |
| User profiles | < 2% | Multi-admin edits in disconnected ministries |
| Roles/permissions | < 0.01% | Centralized control, rarely modified offline |
| Device trust scores | ~0% | Computed value, not user-editable |
| Documents | < 5% | Long-lived, multi-editor; highest conflict potential |

---

## 2. Detection Mechanisms

### 2.1 Merkle Tree Comparison

The primary detection mechanism is merkle tree reconciliation during the sync cycle:

```
Phase 2: Merkle Exchange

Node A's Merkle Tree (partition: mohe/attendance/2026-05)
                        root: 3f7a...
                       /      \
               branch: a1c...  branch: 9b2...
               /      \        /      \
     leaf: clock-in   leaf: clock-out  leaf: correction
     hash: b4e...     hash: 7f1...    hash: 2a8...

Node B's Merkle Tree (same partition)
                        root: 8d3...  ← DIFFERENT from A
                       /      \
               branch: a1c...  branch: 4e6... ← DIFFERENT from A
               /      \        /      \
     leaf: clock-in   leaf: clock-out  leaf: correction
     hash: b4e...     hash: 9c2...    hash: 2a8...
                               ↑
                         DIVERGENT LEAF

Result: Leaf "clock-out" differs between A and B.
Both nodes request the full leaf contents to identify specific conflicting records.
```

### 2.2 Version Vectors

Each record carries a version vector — a map of `(node_id → version_counter)`:

```
Record: clock-event/uuid-123
version_vector: {
    "edge-basra-01":  5,    // This node has seen 5 mutations
    "relay-baghdad":  3,    // Relay has seen 3 mutations
    "national-hub":   2     // National hub has seen 2 mutations
}
```

**Conflict detection rule:**
```
If version_vector_A[node] != version_vector_B[node] for any node:
    AND the differing entries modify the same field:
        → CONFLICT
    AND the differing entries modify different fields:
        → MERGE (no conflict)

If version_vector_A is a descendent of version_vector_B (all entries >=):
    → No conflict, B is stale, fast-forward
```

### 2.3 Causal History Graph

For complex conflicts, each record maintains a causal history — a DAG of event IDs:

```
Record: leave-request/uuid-456

Event History (DAG):
    e1 (created by employee) ← e2 (submitted by employee)
    e1 ← e3 (edited by admin at site A)    // Concurrent with e2
    e2 ← e4 (approved by supervisor)        // Depends on e2
    e3 ← e5 (modified leave dates)          // Depends on e3

    Conflict: e4 (approved) vs e5 (modified dates after approval)
    Resolution: depends on policy (see §5)
```

### 2.4 Detection at Different Layers

| Layer | Mechanism | Triggered When |
|---|---|---|
| **Merkle tree** | Root/branch hash mismatch | Start of every sync cycle |
| **Version vector** | Concurrent increments on same field | Delta comparison phase |
| **Causal history** | DAG branch divergence | Complex conflict evaluation |
| **Constraint check** | DB constraint violation on apply | Apply phase (unique, FK) |
| **Business rule** | Balance goes negative, leave overlap | Post-apply validation |

---

## 3. Resolution Strategies

### 3.1 Strategy Catalog

| Strategy | Code | Description | Data Loss Risk |
|---|---|---|---|
| **Last-Writer-Wins (LWW)** | `lww_timestamp` | Highest `local_timestamp` wins. The entire record is replaced by the winner. | HIGH — losing write is discarded entirely |
| **LWW by Field** | `lww_field` | Per-field resolution. Each field independently takes the last-writer value. | LOW — only conflicting fields affected |
| **Ministry Author Wins** | `ministry_author` | The node designated as "authority" for the data type always wins over subordinate nodes. | MEDIUM — subordinate writes discarded |
| **Service-Side Merge** | `service_merge` | The receiving service applies domain-specific merge logic (e.g., accrual math for balances). | NONE — domain-correct merge |
| **Additive Merge** | `additive` | For set/list fields: both sides are unioned. No data discarded. | NONE |
| **CRDT Auto-Merge** | `crdt` | Data structure designed to be mergeable (e.g., grow-only set, LWW-register). Always converges. | NONE |
| **Manual** | `manual` | Escalated to admin dashboard. Human decision required. | User-defined |
| **Defer** | `defer` | Conflict recorded but not applied. Both versions preserved pending policy change. | NONE (temporary) |

### 3.2 Resolution Decision Tree

```
For each conflicting record R:

  Is R append-only (clock events, audit entries)?
    → LWW by timestamp. Immutable records do not actually conflict
      (each append is unique). Use IDEMPOTENCY check to discard duplicates.

  Is R a financial/numerical value (leave balance)?
    → SERVICE_MERGE. Domain service computes correct value from
      both operation histories. Never LWW on financial data.

  Is R a set or collection (user roles, device bindings)?
    → ADDITIVE merge. Union both sets. No data loss.

  Is there an authoritative node for this data type?
    (e.g., HR records → Ministry Hub is authority)
    → MINISTRY_AUTHOR_WINS. Subordinate node's changes are rolled back
      and re-applied on next cycle.

  Are non-conflicting fields involved?
    → LWW_BY_FIELD. Merge compatible fields, flag only true conflicts.

  Is human judgment required?
    (e.g., overlapping leave approvals, disputed attendance)
    → MANUAL. Escalate with full context.

  Is it a complex structural conflict?
    → CRDT merge if data structure supports it, otherwise MANUAL.
```

### 3.3 Strategy Selection Matrix

```
                    ┌─────────────────────────────────────────────────┐
                    │              UPDATE TYPE                        │
                    ├──────────┬──────────┬──────────┬───────────────┤
                    │  Create  │  Update  │  Delete  │  Recreate     │
         ┌─────────┼──────────┼──────────┼──────────┼───────────────┤
         │ Create  │ LWW /    │ LWW      │ Tombstone│ Last-create   │
         │         │ Merge    │          │ + Alert  │ wins          │
         ├─────────┼──────────┼──────────┼──────────┼───────────────┤
  NODE B │ Update  │ LWW      │ Per-field │ Deleted  │ Recreated     │
         │         │          │ LWW /    │ record   │ overrides     │
         │         │          │ Merge    │ loses    │ delete        │
         ├─────────┼──────────┼──────────┼──────────┼───────────────┤
         │ Delete  │ Tombstone│ Deleted  │ No-op    │ Delete wins   │
         │         │ + Alert  │ record   │ (already │ + Alert       │
         │         │          │ loses    │ deleted) │               │
         ├─────────┼──────────┼──────────┼──────────┼───────────────┤
         │Recreate │ Last-    │ Recreated│ Delete   │ Last-create   │
         │         │ create   │ overrides│ wins     │ wins          │
         │         │ wins     │ update   │ + Alert  │               │
         └─────────┴──────────┴──────────┴──────────┴───────────────┘
```

---

## 4. Per-Entity Resolution Matrix

### 4.1 Attendance Data

| Entity | Conflict Type | Resolution Strategy | Rationale |
|---|---|---|---|
| **ClockEvent** | Duplicate (same employee + device + time ± 30s) | LWW + Dedup | Immutable records; dedup by event signature |
| **ClockEvent** | Device clock skew (different times for same event) | LWW by event_time | Device time used for payroll; server time for ordering |
| **Shift** | Concurrent edit of shift times | Ministry Author Wins | HR/Admin on ministry hub is authoritative |
| **AttendancePolicy** | Concurrent policy activation | LWW + Audit Alert | Only one policy can be active; admin must verify |
| **AttendanceException** | Duplicate justification submission | LWW (last justification accepted) | Employee can re-submit; latest is valid |
| **AttendanceException** | Admin resolution vs employee justification | Ministry Author Wins | Admin resolution overrides employee justification |

### 4.2 Leave Data

| Entity | Conflict Type | Resolution Strategy | Rationale |
|---|---|---|---|
| **LeaveRequest** | Admin edits leave dates after approval | Service Merge + Manual Escalation | Changes to approved leave require re-approval |
| **LeaveRequest** | Concurrent approval/rejection | Manual | Different admins made opposite decisions |
| **LeaveRequest** | Employee cancels while admin approves | Manual | Must determine intent; escalation required |
| **LeaveBalance** | Concurrent balance deduction | Service Merge | CRDT counter: balance = SUM(all accruals) - SUM(all deductions) |
| **LeaveBalance** | Manual adjustment vs system accrual | Ministry Author Wins | Admin adjustment overrides auto-accrual for same period |
| **AccrualPolicy** | Concurrent policy update | LWW | Latest policy activated; overrides previous |
| **LeaveRequest** | Overlap with existing approved leave | Constraint rejection | Overlap detected on apply; rejected with explanation |

### 4.3 Identity Data

| Entity | Conflict Type | Resolution Strategy | Rationale |
|---|---|---|---|
| **User** | Profile field edit by two admins | LWW by Field | Each field resolved independently |
| **User** | Ministry A deactivates, Ministry B retains | Ministry Author Wins | Home ministry has authority |
| **User** | Concurrent role assignment/revocation | Additive Merge | Roles are a set; union of both operations |
| **Device** | Trust score update race | LWW | Score is a computed value; latest computation wins |
| **Device** | Enroll vs suspend at same time | LWW + Audit Alert | Both actions recorded; latest state applies |
| **Realm** | Auth policy concurrent update | LWW | Policy versioned; latest active version wins |

### 4.4 Audit Data

| Entity | Conflict Type | Resolution Strategy | Rationale |
|---|---|---|---|
| **LedgerEntry** | None (append-only) | N/A | Append-only by design; no UPDATE/DELETE |
| **Seal** | Concurrent seal generation | LWW | Both seals valid; later seal covers larger range |

### 4.5 Sync Metadata

| Entity | Conflict Type | Resolution Strategy | Rationale |
|---|---|---|---|
| **SyncBatch** | Duplicate sync batch | LWW + Dedup | Identify by sync_id; discard duplicate |
| **MerkleTree** | Tree root mismatch | Anti-entropy reconciliation | Merkle exchange resolves differences automatically |
| **SyncNode** | Concurrent heartbeat update | LWW | Latest heartbeat supersedes previous |

---

## 5. Resolution Lifecycle

### 5.1 State Machine

```
                     ┌─────────────┐
                     │  DETECTED   │
                     └──────┬──────┘
                            │
                    ┌───────┴───────┐
                    │               │
              ┌─────┴────┐   ┌─────┴────┐
              │AUTO-      │   │MANUAL    │
              │RESOLVABLE │   │REQUIRED  │
              └─────┬────┘   └─────┬────┘
                    │               │
              ┌─────┴────┐   ┌─────┴────┐
              │ RESOLVED │   │ PENDING  │
              │ (auto)   │   │ MANUAL   │
              └─────┬────┘   └─────┬────┘
                    │               │
                    │         ┌─────┴────┐
                    │         │IN REVIEW │
                    │         └─────┬────┘
                    │               │
                    │         ┌─────┴────┐
                    │         │RESOLVED  │
                    │         │(manual)  │
                    │         └─────┬────┘
                    │               │
                    └───────┬───────┘
                            │
                    ┌───────┴───────┐
                    │  COMMITTED    │
                    │  (both nodes  │
                    │   sign batch) │
                    └───────────────┘
```

### 5.2 Lifecycle Phases

```
Phase 1: DETECTION
  ├── Merkle comparison identifies divergent leaves
  ├── Version vector comparison identifies concurrent mutations
  ├── Causal history comparison identifies DAG branches
  └── Constraint check identifies integrity violations

Phase 2: CLASSIFICATION
  ├── Conflict categorized by type (value, delete, constraint, schema)
  ├── Auto-resolvable flag set based on per-entity matrix
  ├── Resolution strategy selected from catalog
  └── If MANUAL → escalate to admin queue immediately

Phase 3: RESOLUTION (AUTO)
  ├── Apply selected strategy (LWW, merge, additive, etc.)
  ├── Generate ConflictResolved event with resolution metadata
  ├── Record both versions in conflict log for audit
  └── Proceed to Phase 5

Phase 4: RESOLUTION (MANUAL)
  ├── Conflict added to admin dashboard queue
  ├── Notification sent to responsible admin (role-based)
  ├── Admin reviews both versions with diff view
  ├── Admin selects winner, merges values, or overrides
  ├── If AUTO-ESCALATE: after 48h, escalate to next admin level
  ├── After 7 days: auto-resolve via default strategy (LWW)
  └── Generate ConflictResolved event

Phase 5: COMMITMENT
  ├── Resolved version written to local store
  ├── Merkle tree updated with new leaf hash
  ├── Both nodes sign the sync batch
  ├── Conflict record sealed in audit ledger
  └── Sync checkpoint advanced
```

### 5.3 Timing & SLA

| Step | Auto-Resolution | Manual Resolution |
|---|---|---|
| Detection | Instant (during sync) | Instant (during sync) |
| Classification | < 100ms | < 100ms |
| Resolution | < 500ms | < 48h target (admin SLA) |
| Escalation | N/A | 48h → mid-level, 96h → senior, 168h → auto-resolve |
| Commitment | Included in current sync batch | Next sync cycle after resolution |
| Audit record | Current batch | When resolution applied |

---

## 6. CRDT-Inspired Data Structures

### 6.1 Design Philosophy

INWP does **not** implement full CRDTs (too complex, storage-heavy). Instead, we use **CRDT-inspired** patterns for specific data types where automatic convergence is critical:

### 6.2 Grow-Only Set (G-Set)

Used for: **Role assignments, device bindings, permissions**

```json
{
  "user_id": "uuid",
  "roles": {
    "added": {
      "role_1": "2026-05-31T08:00:00Z",
      "role_2": "2026-05-31T09:00:00Z"
    }
  }
}
```

**Merge rule:**
```
roles.merged = roles_A.added ∪ roles_B.added
```
Union of all additions. Removals handled via separate revocation list (see §6.3).

**Why not full CRDT:**
True CRDTs require tombstones for removal (2P-Set). We handle removals via a separate revocation event stream to keep the primary set small.

### 6.3 Observed-Remove Set (OR-Set)

Used for: **Device-bound employees, group membership**

```json
{
  "device_id": "uuid",
  "bound_employees": {
    "added": {
      "emp_1": "opaque-tag-a1b2",
      "emp_2": "opaque-tag-c3d4"
    },
    "removed": {
      "emp_3": "opaque-tag-e5f6"
    }
  }
}
```

**Merge rule:**
```
bound = (set_A.added ∪ set_B.added) - (set_A.removed ∪ set_B.removed)
```
Using unique opaque tags per operation prevents the "re-add after remove" problem.

### 6.4 LWW Register (per field)

Used for: **User profiles, shift definitions, policy configurations**

```json
{
  "record_id": "uuid",
  "fields": {
    "full_name": {
      "value": "Ahmed Hassan",
      "timestamp": "2026-05-31T10:00:00Z",
      "node_id": "edge-basra-01"
    },
    "phone": {
      "value": "+964780123456",
      "timestamp": "2026-05-31T11:00:00Z",
      "node_id": "relay-baghdad"
    }
  }
}
```

**Merge rule:**
```
For each field in (fields_A ∪ fields_B):
    if field exists only in one → take that value
    if field exists in both → take the one with highest timestamp
    if same timestamp → node_id lexicographic order wins
```

### 6.5 PN-Counter (Positive-Negative Counter)

Used for: **Leave balances, attendance counts**

```json
{
  "balance_id": "uuid",
  "leave_type": "annual",
  "counters": {
    "accrued": {
      "edge-basra-01": { "P": 2.5, "N": 0 },
      "relay-baghdad": { "P": 5.0, "N": 3.0 }
    },
    "deducted": {
      "edge-basra-01": { "P": 1.0, "N": 0 },
      "relay-baghdad": { "P": 0, "N": 0 }
    }
  }
}
```

**Merge rule:**
```
Accrued value for each node = P - N
Total accrued = SUM(all nodes' accrued values)
Same for deducted.
Current balance = Total accrued - Total deducted
```

**Why PN-Counter for balances:**
Simple counters cannot handle concurrent debits/credits. PN-Counters converge correctly:

```
Node A: balance was 10, deducts 2: P_accrued=10, N_deducted=2
Node B: balance was 10, deducts 3: P_accrued=10, N_deducted=3

After merge:
  P_accrued = max(10, 10) = 10  (or sum for additive counters)
  N_deducted = 2 + 3 = 5
  Balance = 10 - 5 = 5
  → CORRECT (both deductions preserved)
```

### 6.6 Event Log (Append-Only)

Used for: **Clock events, audit entries, sync batches**

No conflict resolution needed. Append-only with idempotency:

```json
{
  "log_id": "uuid",
  "entries": [
    { "id": "evt-1", "data": "...", "timestamp": "..." },
    { "id": "evt-2", "data": "...", "timestamp": "..." }
  ]
}
```

**Merge rule:**
```
merged = entries_A ∪ entries_B
deduplicated by event id
ordered by timestamp
```

---

## 7. Delete Handling

### 7.1 Tombstone Strategy

Soft deletes with tombstones are used for all records that participate in sync:

```json
{
  "record_id": "uuid-123",
  "deleted": true,
  "deleted_at": "2026-05-31T12:00:00Z",
  "deleted_by": "uuid-admin",
  "tombstone_ttl": "2026-12-31T00:00:00Z"
}
```

**Rules:**

| Scenario | Handling |
|---|---|
| Delete on one node, update on another | Tombstone preserved; conflict escalated to manual |
| Delete on both nodes | Tombstone created once; acknowledged by both |
| Recreate after delete | Tombstone removed; new record created with new ID |
| Tombstone expiry | Tombstone purged after TTL (ensures all nodes have synced the delete) |
| Tombstone TTL | Default 90 days (configurable per ministry, min 30 days) |

### 7.2 Delete Propagation

```
Node A: DELETE record R
  ├── Mark R as tombstone (deleted=true, deleted_at=now)
  ├── Update merkle tree leaf hash
  ├── Wait for sync cycle
  │
  ▼
Sync Cycle:
  ├── Node B receives DELETE event
  ├── Node B checks: does B have concurrent modifications to R?
  │   ├── NO → B marks R as tombstone
  │   └── YES → B flags DELETE vs UPDATE conflict (§7.3)
  │
  ▼
Both nodes: R is tombstoned

After tombstone TTL:
  ├── Both nodes: hard-delete R
  ├── Merkle tree prunes leaf
  └── Space reclaimed
```

### 7.3 Delete vs Update Resolution

```
Scenario: Node A deletes record R while Node B updates R.

Resolution:
  1. A and B detect conflict during merkle exchange
  2. Conflict type: DELETE vs UPDATE
  3. Resolution depends on data type:
     a. Clock events (immutable): Delete is impossible; conflict means
        the record doesn't exist on A. A receives the update as a new record.
     b. Leave requests: Ministry Author Wins. If B is the ministry hub
        and the request was approved → UPDATE wins. If B is edge and
        only draft → DELETE wins.
     c. User profiles: Manual resolution required.
        Both versions preserved for admin decision.
  4. Conflict recorded with both versions
  5. If UPDATE wins: A removes tombstone, applies update
  6. If DELETE wins: B applies tombstone, discards update
```

### 7.4 Garbage Collection

```sql
-- Periodic cleanup (daily via pg_cron)
DELETE FROM records
WHERE deleted = true
  AND tombstone_ttl < now()
  AND NOT EXISTS (
    -- Ensure no node is behind
    SELECT 1 FROM sync_checkpoint
    WHERE last_sync_at < deleted_at
  );
```

---

## 8. Merge Semantics by Entity

### 8.1 ClockEvent Merge

```python
def merge_clock_events(local: ClockEvent, remote: ClockEvent) -> ClockEvent:
    """
    Clock events are immutable + idempotent.
    No true merge needed. Identify canonical version.
    """

    # Same event delivered twice → dedup
    if local.event_id == remote.event_id:
        return local  # or remote, identical

    # Same employee, device, type, within 30s window → duplicate
    if (local.employee_id == remote.employee_id
        and local.device_id == remote.device_id
        and local.event_type == remote.event_type
        and abs(local.event_time - remote.event_time) < timedelta(seconds=30)):
        # Keep the one with higher confidence biometric
        if local.biometric_confidence > remote.biometric_confidence:
            return local
        return remote

    # Different events → both are valid, keep both
    return local  # caller will append remote separately
```

### 8.2 LeaveBalance Merge

```python
def merge_leave_balances(local: LeaveBalance, remote: LeaveBalance) -> LeaveBalance:
    """
    Leave balances use PN-Counter semantics.
    The service performs the merge, not generic sync.
    """
    merged = LeaveBalance(id=local.id, employee_id=local.employee_id)

    # Merge PN-Counters
    merged.accruals = merge_pn_counter(local.accruals, remote.accruals)
    merged.deductions = merge_pn_counter(local.deductions, remote.deductions)

    # Compute new balance
    merged.current_balance = merged.accruals.total() - merged.deductions.total()

    # Merge transaction logs (preserve all history)
    merged.transactions = merge_transaction_logs(local.transactions, remote.transactions)

    # Detect and flag constraint violations
    if merged.current_balance < 0:
        if local.ministry_allow_negative:
            merged.current_balance = merged.current_balance  # allowed
            merged.negative_balance_flag = True
        else:
            merged.current_balance = 0  # clamp to zero
            merged.clamped = True
            merged.clamp_reason = "Balance went negative; clamped to 0"

    return merged

def merge_pn_counter(local: PNCounter, remote: PNCounter) -> PNCounter:
    """
    PN-Counter merge: sum all positive increments, sum all negative increments.
    """
    merged = PNCounter()

    # Union all node contributions
    all_nodes = set(local.per_node.keys()) | set(remote.per_node.keys())
    for node in all_nodes:
        local_p = local.per_node.get(node, {}).get("P", 0)
        local_n = local.per_node.get(node, {}).get("N", 0)
        remote_p = remote.per_node.get(node, {}).get("P", 0)
        remote_n = remote.per_node.get(node, {}).get("N", 0)

        # Take max of positive and negative (standard PN-Counter)
        merged.per_node[node] = {
            "P": max(local_p, remote_p),
            "N": max(local_n, remote_n)
        }

    return merged
```

### 8.3 LeaveRequest Merge

```python
def merge_leave_requests(local: LeaveRequest, remote: LeaveRequest) -> MergeResult:
    """
    Leave requests have a defined state machine.
    Merge must respect state transitions.
    """
    STATE_TRANSITIONS = {
        "draft": ["pending_approval", "cancelled"],
        "pending_approval": ["approved", "rejected", "cancelled"],
        "approved": ["cancelled", "in_progress", "completed"],
        "rejected": ["draft"],  # re-submitted
        "cancelled": [],
        "in_progress": ["completed"],
        "completed": []
    }

    # If one is a valid successor of the other → fast-forward
    if local.status in STATE_TRANSITIONS.get(remote.status, []):
        return MergeResult(use=remote, conflict=False)
    if remote.status in STATE_TRANSITIONS.get(local.status, []):
        return MergeResult(use=local, conflict=False)

    # Concurrent transitions that conflict:
    # e.g., admin approved (local) while employee cancelled (remote)
    # → Manual resolution required
    if local.status == "approved" and remote.status == "cancelled":
        return MergeResult(
            use=None,
            conflict=True,
            conflict_type="APPROVED_VS_CANCELLED",
            resolution_strategy="manual",
            local_snapshot=local,
            remote_snapshot=remote
        )

    # Same-field edits (e.g., both changed end_date)
    if local.end_date != remote.end_date:
        return MergeResult(
            use=None,
            conflict=True,
            conflict_type="VALUE_CONFLICT",
            field="end_date",
            local_value=local.end_date,
            remote_value=remote.end_date,
            resolution_strategy="lww_timestamp",
            resolved=local if local.updated_at > remote.updated_at else remote
        )

    # No conflict: different fields modified
    return MergeResult(
        use=merge_fields(local, remote),
        conflict=False,
        merged_fields=True
    )
```

### 8.4 UserProfile Merge

```python
def merge_user_profiles(local: User, remote: User) -> User:
    """
    Per-field LWW merge with ministry authority override.
    """
    merged = User(id=local.id)

    authority_node = get_authority_node("user_profile", local.ministry_id)
    authority = authority_node == "ministry_hub"  # hub is authority

    for field in USER_PROFILE_FIELDS:
        local_val = getattr(local, field)
        remote_val = getattr(remote, field)

        if local_val == remote_val:
            setattr(merged, field, local_val)
            continue

        # Determine which node authored the change
        local_author = local.field_authors.get(field)
        remote_author = remote.field_authors.get(field)

        if authority:
            # Hub is authoritative
            if local_author == "ministry_hub":
                setattr(merged, field, local_val)
            elif remote_author == "ministry_hub":
                setattr(merged, field, remote_val)
            else:
                # Neither is hub; LWW
                winner = local if local.updated_at > remote.updated_at else remote
                setattr(merged, field, getattr(winner, field))
        else:
            # Edge nodes: LWW, edge's own field wins for locally-edited fields
            winner = local if local.updated_at > remote.updated_at else remote
            setattr(merged, field, getattr(winner, field))

    return merged
```

---

## 9. Manual Resolution Workflow

### 9.1 Admin Dashboard

```
┌─────────────────────────────────────────────────────────────────┐
│  ⚠ CONFLICT RESOLUTION REQUIRED                    2 pending   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Conflict #CF-2026-0001                        SEVERITY: │   │
│  │ Type: VALUE CONFLICT                            MEDIUM  │   │
│  │ Entity: Leave Request #LR-456                          │   │
│  │ Detected: 2026-05-31 14:30 UTC                         │   │
│  │ Age: 2h 15m                                             │   │
│  │                                                         │   │
│  │ ┌─────────────────┐  ┌─────────────────────────────┐   │   │
│  │ │ LOCAL VERSION   │  │ REMOTE VERSION              │   │   │
│  │ │ (Edge Basra 01) │  │ (Ministry Hub Baghdad)      │   │   │
│  │ ├─────────────────┤  ├─────────────────────────────┤   │   │
│  │ │ Status: APPROVED│  │ Status: CANCELLED           │   │   │
│  │ │ Approved by:    │  │ Cancelled by:               │   │   │
│  │ │   Supervisor A  │  │   HR Manager B              │   │   │
│  │ │ At: 10:30 UTC   │  │ At: 11:00 UTC               │   │   │
│  │ └─────────────────┘  └─────────────────────────────┘   │   │
│  │                                                         │   │
│  │ Actions: [KEEP LOCAL] [KEEP REMOTE] [CUSTOM MERGE]     │   │
│  │                                                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Conflict #CF-2026-0002                        SEVERITY: │   │
│  │ Type: DELETE VS UPDATE                           HIGH   │   │
│  │ Entity: Employee #EMP-789                               │   │
│  │ ...                                                     │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 9.2 Notification Chain

```
Conflict Detected
  ├── AUTO-RESOLVABLE?
  │   └── Yes → Resolve, log, continue
  │
  └── MANUAL REQUIRED?
      └── Yes:
          ├── Immediate notification to assigned resolver role
          │   (e.g., "attendance_admin" for attendance conflicts)
          ├── Dashboard entry created
          │
          ├── [48h] No resolution?
          │   └── Escalate to next-level admin
          │       (e.g., "ministry_admin")
          │
          ├── [96h] No resolution?
          │   └── Escalate to "national_admin"
          │
          └── [168h] No resolution?
              └── Auto-resolve via default strategy (LWW)
                  + mandatory audit report
```

### 9.3 API for Manual Resolution

```protobuf
service ConflictResolutionService {
    // List pending conflicts for an admin
    rpc ListConflicts(ListConflictsRequest) returns (ListConflictsResponse);

    // Get full details of a specific conflict
    rpc GetConflict(GetConflictRequest) returns (ConflictDetail);

    // Resolve a conflict manually
    rpc ResolveConflict(ResolveConflictRequest) returns (ResolveConflictResponse);

    // Batch resolve multiple conflicts with same strategy
    rpc BatchResolveConflicts(BatchResolveRequest) returns (BatchResolveResponse);
}

message ResolveConflictRequest {
    string conflict_id = 1;
    string resolution = 2;        // "local_wins", "remote_wins", "custom_merge"
    string resolved_by = 3;       // admin user ID
    string notes = 4;             // mandatory reason for audit
    optional bytes merged_payload = 5;  // for custom merges
}
```

### 9.4 Manual Resolution Audit Record

Every manual resolution produces an audit event:

```json
{
  "conflict_id": "cf-uuid",
  "entity_type": "leave_request",
  "entity_id": "lr-uuid",
  "resolution": "local_wins",
  "resolved_by": "admin-uuid",
  "resolved_at": "2026-05-31T15:00:00Z",
  "notes": "Employee confirmed they did not cancel. Hub version was in error.",
  "local_snapshot": { ... },
  "remote_snapshot": { ... },
  "merged_result": { ... }
}
```

---

## 10. Edge Cases & Guarantees

### 10.1 Convergence Guarantee

```
Theorem: After a full sync cycle between any two nodes with the
same data set, both nodes will reach the same state for all records
that were not flagged as manual conflicts.

Proof:
  - Merkle exchange identifies all differing records (complete diff)
  - Each differing record goes through deterministic resolution
  - Resolution produces the same result on both nodes
    (deterministic algorithms + signed agreements)
  - Both nodes apply the same result
  - Both nodes update their merkle trees
  - After commitment, merkle roots match → state is identical
```

### 10.2 Split-Brain Prevention

INWP prevents split-brain through:

| Mechanism | How it prevents split-brain |
|---|---|
| **Deterministic resolution** | Same algorithm on all nodes produces same result |
| **Signed batch receipts** | Both nodes sign the outcome; third party can verify |
| **Merkle root convergence** | After sync, merkle roots match → state identical |
| **National hub as tiebreaker** | If resolution splits, hub's decision is final |
| **No independent authority split** | Each data type has a defined authority node |
| **Periodic full audit** | Seal verification catches any divergence |

### 10.3 Network Partition Scenarios

```
Scenario A: Temporary partition (< 24h)
  ├── Nodes operate independently
  ├── Conflicts expected but bounded
  ├── Full reconciliation on reconnect
  └── Auto-resolve majority; manual for few high-severity

Scenario B: Extended partition (24h - 90 days)
  ├── Tombstones may expire before all nodes sync
  ├── Extended TTL: tombstone expiry extended by partition duration
  ├── Ministry hub holds authoritative state for long-term reconciliation
  └── National hub publishes "consensus snapshot" for rejoining nodes

Scenario C: Permanent partition (node never returns)
  ├── Node marked DECOMMISSIONED after 90 days offline
  ├── Other nodes prune its version vector entries
  ├── Tombstones for its data purged
  └── If node returns: full re-sync from hub, discard all local state
```

### 10.4 Clock Skew Handling

```
Problem: LWW depends on timestamps. Clock skew between devices causes
incorrect ordering.

Solution:
  1. All events carry BOTH device_local_time AND server_recorded_at
  2. Internal ordering uses hybrid logical clocks (HLC):
     - max(device_time, server_time) with tiebreaker by node_id
  3. Clock skew detection:
     - If device_time differs from server_time by > 300s → flag for admin
     - If any node has consistent > 60s skew → NTP alert
  4. For LWW resolution, the authoritative timestamp is:
     - If both events from same node: use local sequence number
     - If different nodes: use HLC timestamp
     - If HLC ties: node_id UUID comparison (consistent total order)
```

### 10.5 Idempotency Guarantees

```
Every event has a unique event_id (UUID v7).
The sync engine guarantees:

  At-least-once delivery: An event may be delivered more than once.
  Idempotent processing: Applying the same event twice produces the
                          same state as applying it once.

Implementation:
  - event_outbox table records which event_ids have been processed
  - event_id is the deduplication key (unique index)
  - Duplicate detection before any mutation
  - Sync batches carry event_id list; receiver checks each against
    processed_events table
```

### 10.6 Concurrent Schema Migration

```
Problem: Node A upgrades to schema v2 while Node B is still on v1.
Events produced by A may include fields B doesn't understand.

Solution:
  1. Schema negotiation during sync discovery phase:
     - Each node advertises supported schema versions per entity
     - Sync only transfers data using the minimum common schema version
  2. Forward compatibility:
     - Unknown fields are preserved as opaque JSONB blobs
     - When B upgrades, it can read previously preserved fields
  3. Backward compatibility:
     - Schema v2 must accept v1 events
     - Missing fields filled with defaults during deserialization
  4. Migration window:
     - Schema changes rolled out to edge nodes within 30 days
     - After 30 days, old schema nodes are isolated
```

### 10.7 Summary of Guarantees

| Property | Guarantee | Exception |
|---|---|---|
| **Eventual convergence** | All nodes reach same state | Manual conflicts pending resolution |
| **No data loss** | All mutations preserved in event log | Discarded by manual resolution |
| **Idempotent apply** | Duplicate events produce same state | N/A |
| **Causal ordering** | If A happened-before B, A applied before B | Concurrent events (no causal relation) |
| **Total order** | All nodes agree on event order | Concurrent events may order differently |
| **Integrity** | All sync batches signed by both parties | N/A |
| **Auditability** | All conflicts and resolutions recorded | N/A |
| **Bounded tombstone** | Tombstones eventually purged | All nodes must acknowledge delete first |
