# INWP Foundational Architecture

> Iraq National Workforce Platform — Sovereign, distributed, offline-first workforce operating system.

---

## Table of Contents

1. [System Architecture](#1-system-architecture)
2. [Core Services](#2-core-services)
3. [Event Model](#3-event-model)
4. [Sync Architecture](#4-sync-architecture)
5. [Security Architecture](#5-security-architecture)
6. [Database Strategy](#6-database-strategy)
7. [Deployment Topology](#7-deployment-topology)
8. [API Strategy](#8-api-strategy)
9. [Identity Architecture](#9-identity-architecture)
10. [Device Integration Model](#10-device-integration-model)

---

## 1. System Architecture

### 1.1 High-Level Topology

INWP operates across three physical tiers, each capable of independent operation:

```
+------------------------------------------------------------------+
|                      NATIONAL DATA CENTER                         |
|  +----------------+  +----------------+  +------------------+     |
|  | Identity IdP   |  | Audit Ledger   |  | National Sync Hub|     |
|  | (Federation)   |  | (Immutable)    |  | (Anti-Entropy)   |     |
|  +----------------+  +----------------+  +------------------+     |
|  +----------------+  +----------------+  +------------------+     |
|  | Platform Core  |  | Web Portal     |  | Analytics/BI     |     |
|  +----------------+  +----------------+  +------------------+     |
+------------------------------------------------------------------+
           |                     |                     |
     (Encrypted WAN)      (Encrypted WAN)      (Encrypted WAN)
           v                     v                     v
+------------------------------------------------------------------+
|                      REGIONAL HUBS (x18)                          |
|  +----------------+  +----------------+  +------------------+     |
|  | Regional Sync  |  | Regional Audit |  | Regional Identity |   |
|  | Relay          |  | Buffer         |  | Cache/Replica     |   |
|  +----------------+  +----------------+  +------------------+     |
|  +----------------+  +----------------+                           |
|  | Attendance Agg |  | Leave Service  |                           |
|  +----------------+  +----------------+                           |
+------------------------------------------------------------------+
           |                     |                     |
    (LAN / Mesh)          (LAN / Mesh)          (Encrypted Radio)
           v                     v                     v
+------------------------------------------------------------------+
|                      EDGE NODES (1000+)                            |
|  +----------------+  +----------------+  +------------------+     |
|  | Local Sync     |  | Local PG       |  | Biometric Device |     |
|  | Engine         |  | (Offline Store)|  | Integration      |     |
|  +----------------+  +----------------+  +------------------+     |
|  +----------------+  +----------------+                           |
|  | Attendance     |  | Leave Approver |                           |
|  | Terminal       |  | (Offline)      |                           |
|  +----------------+  +----------------+                           |
+------------------------------------------------------------------+
```

### 1.2 Architectural Principles

| Principle | Application |
|---|---|
| **Offline-first** | Every node functions fully when disconnected. Local PG is the source of truth until sync completes. |
| **Event-driven** | All state changes produce events. Services communicate exclusively through event channels. No synchronous RPC for business transactions. |
| **Local-first** | Data is owned by the node that creates it. The national DC is a convergent replica, not the authoritative source. |
| **Anti-entropy** | Sync uses merkle-tree based anti-entropy protocols with delta compression. No single-master topology. |
| **Defense in depth** | mTLS at transport, JWT at application, cryptographic signatures at data layer, hardware attestation at device layer. |
| **Tenant isolation** | Each ministry is a tenant with isolated schema, encryption keys, and authentication realms. |
| **No SPOF** | Every service runs with minimum 2 replicas at every tier. Edge nodes are independently operational. |

### 1.3 Communication Patterns

| Pattern | Protocol | Use Case |
|---|---|---|
| Service-to-service (internal) | gRPC + mTLS | Query, command, internal RPC |
| Service-to-service (events) | NATS / Pulsar (national), local pg_notify (edge) | Event publish-subscribe |
| Edge → Hub | HTTPS + mTLS | Sync upload, heartbeat |
| Hub → Edge | WebSocket + mTLS | Sync push, commands |
| LAN peer-to-peer | mDNS + gRPC | Office mesh sync |
| Mobile → Edge/Cloud | HTTPS + Certificate Pinning | Employee operations |
| Admin → Web Portal | HTTPS + OAuth2 + OIDC | Dashboard, reports |

---

## 2. Core Services

### 2.1 Service Map

```
                    +---------------------------+
                    |     platform-core          |
                    | (Domain kernel, schemas,   |
                    |  shared libs, base events) |
                    +---------------------------+
                            |  depends
                            v
+----------+  +----------+ +----------+ +----------+ +----------+
| identity |  |   sync   | |  audit   | | notifica | |   doc    |
| -service |  | -engine  | | -ledger  | | -tion    | | -service |
+----------+  +----------+ +----------+ +----------+ +----------+
                            |
          +-----------------+-----------------+
          v                                   v
+------------------+                 +------------------+
| attendance-      |                 | leave-service    |
| service          |                 |                  |
+------------------+                 +------------------+
          |
          v
+------------------+
| biometric-        |
| integration      |
+------------------+
```

### 2.2 Service Specifications

#### `platform-core` (Kernel)

| Attribute | Value |
|---|---|
| **Language** | Go (performance, single binary, cross-compile) |
| **Responsibility** | Domain models, event schema registry, shared validation, base crypto, common utilities |
| **Output** | Shared library (Go module), protobuf schemas, JSON Schema catalog |
| **Dependencies** | None (zero external deps beyond stdlib) |
| **Deployment** | Embedded as dependency; not independently deployed |

#### `identity-service`

| Attribute | Value |
|---|---|
| **Language** | Go |
| **Database** | PostgreSQL (dedicated schema `idm_*`) |
| **Responsibilities** | User registration, authentication (password, biometric, certificate), RBAC/ABAC, ministry realm management, session management, device enrollment, SCIM provisioning |
| **APIs** | gRPC (internal), REST / OAuth2 / OIDC (external) |
| **Key Features** | Ministry-scoped realms, federated IdP bridging, self-registration with approval flow, hardware-backed keys |
| **Sync** | User/role/group changes → outbound events |

#### `sync-engine`

| Attribute | Value |
|---|---|
| **Language** | Rust (performance-critical sync, memory safety) |
| **Database** | PostgreSQL (sync metadata, merkle trees, checkpoint state) |
| **Responsibilities** | Merkle tree reconciliation, delta computation, conflict resolution (CRDT / LWW / custom), data compression, bandwidth management, sync scheduling, LAN discovery |
| **Protocols** | Custom sync protocol over HTTPS + WebSocket; mDNS for LAN peer discovery |
| **Key Features** | Differential sync, checkpoint/resume, bandwidth throttling, ministry-level sync policies, offline queue |

#### `audit-ledger`

| Attribute | Value |
|---|---|
| **Language** | Go |
| **Database** | PostgreSQL (append-only `ledger_*` tables, hash chains) |
| **Responsibilities** | Immutable event ingestion, hash-chain verification, tamper-evident seal generation, compliance queries, retention management |
| **Key Features** | Cryptographic linking (prev_hash chain), periodic merkle root publication, read-only API for queries, append-only (no UPDATE/DELETE), WAL-based replication to national DC |
| **Data Model** | `(event_id, prev_hash, payload_hash, signature, timestamp, metadata)` — chained via `prev_hash` |

#### `attendance-service`

| Attribute | Value |
|---|---|
| **Language** | Go |
| **Database** | PostgreSQL (local; partitioned by month & ministry) |
| **Responsibilities** | Clock-in/out, shift management, overtime calculation, attendance policy enforcement, biometric verification delegation |
| **Key Features** | Full offline operation, batch sync, policy-as-config (not hardcoded), exception handling |
| **Sync Priority** | HIGH — attendance data is latency-sensitive for payroll |

#### `leave-service`

| Attribute | Value |
|---|---|
| **Language** | Go |
| **Database** | PostgreSQL (local; partitioned by ministry) |
| **Responsibilities** | Leave requests, approval workflows, balance tracking, ministry leave policies, calendar integration |
| **Key Features** | Hierarchical approval chains, policy-driven accrual, offline request/approve, sync on reconnect |
| **Sync Priority** | MEDIUM |

#### `notification-service`

| Attribute | Value |
|---|---|
| **Language** | Go |
| **Database** | PostgreSQL (delivery tracking, templates) |
| **Responsibilities** | Multi-channel notification (push, SMS, email, on-screen), template engine, delivery guarantees, ministry notification policies |
| **Key Features** | Offline-capable delivery queue, exponential backoff, read receipts, ministry branding |

#### `document-service` (future)

| Attribute | Value |
|---|---|
| **Language** | Go |
| **Database** | PostgreSQL + S3-compatible blob store |
| **Responsibilities** | Document management, digital signatures, attachment handling, ministry document workflows |
| **Key Features** | Versioning, audit trail, format conversion |

### 2.3 Service Dependencies Graph

```
identity-service:  [platform-core, postgres]
sync-engine:       [platform-core, postgres]
audit-ledger:      [platform-core, postgres, sync-engine (event feed)]
attendance-service:[platform-core, postgres, identity-service, sync-engine]
leave-service:     [platform-core, postgres, identity-service, sync-engine]
notification-service:[platform-core, postgres, identity-service]
```

---

## 3. Event Model

### 3.1 Event Schema Standard

INWP adopts **CloudEvents 1.0** with mandatory extensions:

```json
{
  "specversion": "1.0",
  "id": "uuid-v7",
  "source": "/ministries/{ministry_id}/sites/{site_id}/services/{service}",
  "type": "inwp.attendance.v1.clock-in.created",
  "datacontenttype": "application/json",
  "subject": "employee:uuid",
  "time": "2026-05-31T10:00:00Z",
  "dataschema": "inwp:event:attendance:clock-in:created:v1",
  "ministry_id": "mohe",
  "site_id": "basra-univ-01",
  "device_id": "bio-scanner-42",
  "offline_generated": true,
  "local_timestamp": "2026-05-31T13:00:00+03:00",
  "sync_id": "sync-batch-7e9a",
  "data": { ... }
}
```

### 3.2 Event Taxonomy

```
inwp
├── attendance.v1
│   ├── clock-in.created
│   ├── clock-out.created
│   ├── break.started
│   ├── break.ended
│   ├── attendance.corrected
│   └── attendance.disputed
├── leave.v1
│   ├── request.created
│   ├── request.approved
│   ├── request.rejected
│   ├── request.cancelled
│   ├── balance.adjusted
│   └── accrual.processed
├── identity.v1
│   ├── user.registered
│   ├── user.verified
│   ├── user.deactivated
│   ├── role.assigned
│   ├── role.revoked
│   ├── device.enrolled
│   ├── device.suspended
│   └── credential.changed
├── sync.v1
│   ├── batch.committed
│   ├── conflict.detected
│   ├── conflict.resolved
│   └── heartbeat.sent
├── audit.v1
│   ├── seal.generated
│   ├── seal.verified
│   └── integrity.failure
├── notification.v1
│   ├── notification.sent
│   ├── notification.delivered
│   └── notification.failed
└── system.v1
    ├── node.online
    ├── node.offline
    ├── service.healthy
    └── service.degraded
```

### 3.3 Event Lifecycle

```
[Producer]
    |
    |---(1) Validate against schema (JSON Schema / Protobuf)
    |
    |---(2) Sign with producer private key (Ed25519)
    |
    |---(3) Persist locally to event_store table (PostgreSQL)
    |
    |---(4) Publish to local event bus (pg_notify / NATS)
    |
    v
[Event Bus]
    |
    |---(5) Route to subscribers (content-based + tenant-aware)
    |
    |---(6) Persist to audit-ledger (immutable chain)
    |
    v
[Consumers]
    |
    |---(7) Verify signature
    |
    |---(8) Validate schema version
    |
    |---(9) Idempotent processing (dedup by event_id)
    |
    |---(10) Acknowledge / dead-letter on failure
```

### 3.4 Event Versioning

| Strategy | Detail |
|---|---|
| **Breaking changes** | New major version (`attendance.v2`) — co-exist until old consumers migrate |
| **Additive changes** | Backward-compatible — `dataschema` URL reflects minor version |
| **Deprecation** | Events carry `deprecated: true` metadata; removed after 2 migration cycles |
| **Schema registry** | `platform-core` maintains canonical schemas; services validate at production time |

### 3.5 Event Storage

At the database level, each service stores outgoing events in an `event_outbox` table:

```sql
CREATE TABLE event_outbox (
    event_id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type      TEXT NOT NULL,          -- "inwp.attendance.v1.clock-in.created"
    source          TEXT NOT NULL,
    data            JSONB NOT NULL,
    data_schema     TEXT NOT NULL,
    signature       BYTEA NOT NULL,
    signing_key_id  TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',  -- pending, published, acknowledged
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at    TIMESTAMPTZ,
    sync_batch_id   UUID
);

CREATE INDEX idx_event_outbox_status ON event_outbox(status);
CREATE INDEX idx_event_outbox_created ON event_outbox(created_at);
```

Events remain `pending` until acknowledged by the event bus or sync engine.

---

## 4. Sync Architecture

### 4.1 Design Principles

- **No single source of truth** — the truth is eventually converged across all nodes
- **Anti-entropy over consensus** — merkle tree reconciliation; no RAFT/Paxos across WAN
- **Peer-to-peer on LAN** — direct sync between edge nodes without hub mediation
- **Hub-and-spoke on WAN** — regional relays optimize bandwidth
- **CRDT-inspired** — conflict resolution via LWW (last-writer-wins) with ministry-configurable policies
- **Delta-only** — only changed data transmitted; full snapshot on first sync

### 4.2 Sync Topology

```
                +------------------+
                |  National Sync   |
                |  Hub (NAT)       |
                +--------+---------+
                         |
           +-------------+-------------+
           |             |             |
     +-----v---+   +-----v---+   +-----v---+
     | Region1 |   | Region2 |   | Region3 |
     | Relay   |   | Relay   |   | Relay   |
     +-----+---+   +-----+---+   +-----+---+
           |             |             |
     +-----v---+   +-----v---+   +-----v---+
     | Edge 1  |   | Edge 2  |   | Edge 3  |
     | (LAN    |<->| (LAN    |<->| (LAN    |
     | Mesh)   |   | Mesh)   |   | Mesh)   |
     +---------+   +---------+   +---------+
```

- **National Hub** — global event routing, cross-ministry sync coordination, merkle root publication
- **Regional Relay** — compression/decompression, bandwidth shaping, store-and-forward for disconnected regions
- **Edge Nodes** — local source of truth, LAN mesh peering, periodic regional sync

### 4.3 Sync Protocol

#### 4.3.1 Phases of a Sync Cycle

```
Phase 1: Discovery
  Edge Node A broadcasts presence via mDNS (LAN) or connects to assigned Regional Relay (WAN).
  Exchange capabilities, supported schema versions, last sync checkpoint.

Phase 2: Merkle Exchange
  Each node maintains a merkle tree of all synced data keys, partitioned by:
    - Ministry
    - Entity type (attendance, leave, user, etc.)
    - Time bucket (hourly buckets for high-churn data)

  Node A sends merkle root hashes for each partition.
  Node B compares with its own, identifies differing branches.
  Both request missing leaf hashes recursively until the differing keys are identified.

Phase 3: Delta Transfer
  For each differing key, the owning node sends the full record + event chain.
  Payload is compressed (zstd) and optionally chunked for large transfers.

Phase 4: Reconciliation
  Both nodes apply mutual changes.
  Conflicts are resolved according to policy:
    - LWW (default) — highest `local_timestamp` wins
    - Ministry author wins — HR records override terminal records
    - Manual resolution — conflicts escalated to admin dashboard

Phase 5: Commitment
  Both nodes sign the sync batch (sync_id, merkle root after apply, timestamp).
  Sync receipt appended to local audit log.
  Both advance their checkpoint.
```

#### 4.3.2 Conflict Resolution Matrix

| Data Type | Default Strategy | Ministry Overridable |
|---|---|---|
| Attendance clock events | LWW (timestamp) | No |
| Leave requests | LWW + author precedence | Yes |
| User profile | Ministry author wins | N/A |
| Roles/Permissions | Ministry author wins | N/A |
| Leave balances | Service-side merge (accrual math) | No |
| Audit events | Append-only (no conflict) | N/A |
| Documents | LWW + version vector | Yes |

### 4.4 Bandwidth Management

| Technique | Implementation |
|---|---|
| **Delta compression** | zstd at the payload level; only send diffs after initial sync |
| **Priority queuing** | Attendance = high (real-time), Leave = medium, Documents = low |
| **Bandwidth budgeting** | Admin-configurable daily bandwidth cap per node |
| **Sync scheduling** | Configurable windows (e.g., 02:00-05:00 for bulk, immediate for high-priority) |
| **Chunking** | Records batched in 1MB chunks; resume on interrupt |
| **Image compression** | Biometric images resized to 640x480, JPEG quality 70 |

### 4.5 Offline Operation

| Capability | Detail |
|---|---|
| **Local writes** | All CRUD operations succeed locally; event_outbox stores pending events |
| **Local reads** | Complete local PostgreSQL; no dependency on remote data |
| **Queue depth** | Unlimited (disk-based); oldest events purged only after confirmed sync |
| **Reconnection** | Exponential backoff (1s → 5s → 30s → 5min → 30min); immediate retry on network change |
| **Conflict queue** | Conflicting writes held for admin review; no data loss |
| **Heartbeat** | Edge nodes emit heartbeat every 60s; hub detects stale nodes |

### 4.6 Sync Data Model

```sql
-- Sync metadata table (present on every node)
CREATE TABLE sync_checkpoint (
    node_id         UUID NOT NULL,
    partition_key   TEXT NOT NULL,     -- "mohe/attendance/2026-05"
    merkle_root     BYTEA NOT NULL,
    last_sync_at    TIMESTAMPTZ,
    synced_events   BIGINT DEFAULT 0,
    PRIMARY KEY (node_id, partition_key)
);

-- Sync batch log (immutable receipts)
CREATE TABLE sync_batch_log (
    sync_id         UUID PRIMARY KEY,
    source_node     UUID NOT NULL,
    target_node     UUID NOT NULL,
    partition_key   TEXT NOT NULL,
    events_count    INT NOT NULL,
    bytes_transfered BIGINT NOT NULL,
    conflict_count  INT DEFAULT 0,
    local_merkle    BYTEA NOT NULL,
    remote_merkle   BYTEA NOT NULL,
    source_sig      BYTEA NOT NULL,
    target_sig      BYTEA NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

---

## 5. Security Architecture

### 5.1 Zero-Trust Model

```
+-----------------------+       +-----------------------+
| Request               | ----> | Never trust, always    |
| (any source: service, |       | verify                |
|  device, user, admin) |       |                       |
+-----------------------+       +----------+------------+
                                            |
                           +----------------+----------------+
                           |                |                |
                           v                v                v
                     +---------+      +---------+      +----------+
                     | mTLS    |      | JWT     |      | Policy   |
                     | Verify  |      | Verify  |      | Evaluate |
                     | Cert    |      | (OAuth2)|      | (RBAC +  |
                     +---------+      +---------+      | ABAC)    |
                                                        +----------+
```

### 5.2 Security Layers

| Layer | Mechanism | Scope |
|---|---|---|
| **Transport** | mTLS 1.3 — all service-to-service communication; certificate pinning on mobile | Wire |
| **Authentication** | OAuth2 + OIDC (external users); JWT with signed assertions (internal services) | Identity |
| **Authorization** | RBAC (role-based) + ABAC (attribute-based, ministry/site/region context) | Access |
| **Data at rest** | AES-256-GCM per-tenant encryption keys; envelope encryption with HSM backing | Storage |
| **Data in transit** | TLS 1.3; payload-level signing (Ed25519) for audit events | Data |
| **Device trust** | Hardware attestation (TPM/TrustZone), device certificate enrollment, trust score | Device |
| **Audit** | Immutable hash chain, cryptographic seals, periodic verification | Accountability |
| **Secrets** | Vault (HashiCorp) or equivalent; no secrets in config files, env vars, or images | Operations |

### 5.3 Certificate Authority Hierarchy

```
            +-----------------------------+
            | INWP Root CA (offline)      |
            | Self-signed, HSM-protected  |
            +-----------------------------+
                         |
          +--------------+--------------+
          |              |              |
+---------v----+  +------v------+  +---v----------+
| National CA  |  | Ministry CA |  | Device CA    |
| (online,     |  | (per        |  | (per tenant, |
| HSM-backed)  |  |  ministry)  |  |  short TTL)  |
+--------------+  +-------------+  +--------------+
```

- **Root CA** — air-gapped, powered on only for CRL updates
- **National CA** — issues service certificates, regional relay certificates
- **Ministry CA** — issues ministry-admin certificates, site-level certs
- **Device CA** — issues device identity certificates (enrolled via attestation)

### 5.4 Authentication Flows

#### User Authentication (External)

```
User -> Web Portal / Mobile App
    |--(1) POST /auth/authorize { username, password, ministry_id, device_attestation }
    |
Identity Service
    |--(2) Verify credentials against ministry realm
    |--(3) Verify device attestation (TPM nonce challenge)
    |--(4) Issue short-lived access_token (15 min) + refresh_token (24h)
    |--(5) Issue session JWT with claims:
    |       {
    |         "sub": "user:uuid",
    |         "ministry": "mohe",
    |         "site": "basra-univ",
    |         "roles": ["hr_admin", "attendance_operator"],
    |         "device_id": "bio-scanner-42",
    |         "auth_method": "password+biometric",
    |         "iat": ...,
    |         "exp": ...
    |       }
    |
User -> API Gateway
    |--(6) Present JWT; gateway validates signature, expiry, device trust score
    |--(7) Forward to target service with JWT claims as gRPC metadata
```

#### Service-to-Service Authentication

```
Service A
  |--(1) mTLS handshake with service certificate (issued by National CA)
  |--(2) Present service JWT (short-lived, 5 min):
  |       {
  |         "sub": "service:attendance-service",
  |         "aud": "service:leave-service",
  |         "ministries": ["mohe", "moh"],
  |         "iat": ...,
  |         "exp": ...
  |       }
  |
Service B
  |--(3) Verify mTLS client cert (check against CRL, validity period)
  |--(4) Verify JWT signature (service account key)
  |--(5) Evaluate ABAC policy (can attendance-service read leave records for mohe?)
  |--(6) Authorize or reject
```

### 5.5 Cryptographic Inventory

| Algorithm | Usage | Key Length |
|---|---|---|
| Ed25519 | Event signing, sync batch signatures | 256-bit |
| X25519 | Key exchange (E2EE for sensitive data) | 256-bit |
| AES-256-GCM | Data at rest (per-tenant envelope) | 256-bit |
| SHA-256 | Merkle tree, hash chains | 256-bit |
| TLS 1.3 | Transport security | X.509 (ECC P-384) |
| Argon2id | Password hashing | (memory=cost, time=cost) |

### 5.6 Security Event Monitoring

All security-relevant events (auth failures, policy violations, crypto errors, device trust degradation) produce `inwp.security.v1.*` events that feed into a centralized security information and event management (SIEM) pipeline.

---

## 6. Database Strategy

### 6.1 PostgreSQL-Only Mandate

No other database engines. PostgreSQL serves every role:

| Role | PostgreSQL Feature |
|---|---|
| Primary data store | Tables, indexes, partitioning |
| Event store | `event_outbox` table with append-only semantics |
| Sync metadata | Merkle trees, checkpoints, batch logs |
| Audit ledger | Hash-chain tables, cryptographic seals |
| Message queue | `pg_notify` / `LISTEN`/`NOTIFY` for local event bus |
| Cache | Materialized views, `pg_stat_statements` for hot data |
| Search | `tsvector` full-text search |
| Time-series | `pg_partman` for partition management |
| Replication | Streaming replication, logical replication for cross-DC |
| Encryption | `pgcrypto` for column-level encryption |
| Scheduling | `pg_cron` for maintenance tasks |

### 6.2 Schema Strategy

#### Database-per-Service

Each service owns its database/schema namespace:

| Service | Database | Schema Prefix |
|---|---|---|
| identity-service | `inwp_identity` | `idm_` |
| sync-engine | `inwp_sync` | `sync_` |
| audit-ledger | `inwp_audit` | `ledger_` |
| attendance-service | `inwp_attendance` | `att_` |
| leave-service | `inwp_leave` | `lv_` |
| notification-service | `inwp_notification` | `notif_` |

#### Multi-Tenancy via Row-Level Security

```sql
-- Every table includes ministry_id and site_id
ALTER TABLE attendance.clock_events ENABLE ROW LEVEL SECURITY;

CREATE POLICY ministry_isolation ON attendance.clock_events
    USING (ministry_id = current_setting('app.current_ministry')::UUID);

CREATE POLICY site_operator_access ON attendance.clock_events
    USING (
        ministry_id = current_setting('app.current_ministry')::UUID
        AND site_id = current_setting('app.current_site')::UUID
    );
```

### 6.3 Partitioning Strategy

```sql
-- Time-based partitioning for high-volume tables
CREATE TABLE attendance.clock_events (
    id UUID NOT NULL,
    employee_id UUID NOT NULL,
    ministry_id UUID NOT NULL,
    site_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    event_time TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    sync_id UUID,
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Monthly partitions via pg_partman
SELECT partman.create_parent(
    p_parent_table := 'attendance.clock_events',
    p_control := 'created_at',
    p_interval := '1 month',
    p_premake := 3
);
```

### 6.4 Offline Database Configuration

Edge nodes run PostgreSQL with reduced resource configuration:

```ini
# postgresql.conf (edge node)
max_connections = 20
shared_buffers = 256MB
effective_cache_size = 512MB
work_mem = 16MB
maintenance_work_mem = 64MB
wal_level = logical           # Required for sync engine
max_replication_slots = 5
max_wal_senders = 5
listen_addresses = 'localhost,192.168.0.0/16'  # LAN only
```

### 6.5 Disaster Recovery

| Tier | RPO | RTO | Mechanism |
|---|---|---|---|
| National DC | 0-5s | < 1min | Synchronous replication (3-node Patroni cluster) |
| Regional Hub | < 5min | < 15min | Asynchronous streaming replication |
| Edge Node | < 24h | < 1h | WAL archive + pgBackRest to regional hub |
| Cross-region DR | < 1h | < 4h | Logical replication to DR site |

### 6.6 Migration Strategy

- All schema changes via versioned migration files (Go `golang-migrate` or equivalent)
- Migrations are part of the service binary (embedded via `embed`)
- Zero-downtime migrations: backward-compatible schema changes only
- Long-running migrations use `pg_cron` in maintenance windows
- Rollback via revert migration (restore from pre-migration snapshot if revert unavailable)

---

## 7. Deployment Topology

### 7.1 Tier Architecture

```
+---------------------+     +---------------------+     +---------------------+
| NATIONAL DC         |     | REGIONAL HUB (x18)  |     | EDGE NODE (1000+)   |
| (Kubernetes)        |     | (Docker Compose)    |     | (Docker Compose)    |
|                     |     |                     |     |                     |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| | Identity IdP  |   |     | | Sync Relay    |   |     | | Sync Engine   |   |
| | (3 replicas)  |   |     | | (2 replicas)  |   |     | | (1 instance)  |   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| | Sync Hub      |   |     | | PG Replica    |   |     | | Postgres      |   |
| | (3 replicas)  |   |     | | (read-only)   |   |     | | (local)       |   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| | Audit Ledger  |   |     | | Audit Cache   |   |     | | Attendance    |   |
| | (3 replicas)  |   |     | | (buffer)      |   |     | | Service       |   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| +---------------+   |     |                     |     | +---------------+   |
| | PG Cluster    |   |     |                     |     | | Leave Service |   |
| | (Patroni)     |   |     |                     |     | +---------------+   |
| +---------------+   |     |                     |     |                     |
+---------------------+     +---------------------+     +---------------------+
```

### 7.2 Docker Image Strategy

```dockerfile
# Multi-stage build pattern (all services)
FROM golang:1.22-alpine AS builder
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -o /app/service ./cmd/service

FROM alpine:3.19
RUN apk add --no-cache ca-certificates tzdata curl
COPY --from=builder /app/service /service
COPY --from=builder /src/migrations /migrations
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1
USER 1001:1001
ENTRYPOINT ["/service"]
```

### 7.3 Container Registry

- Private registry per ministry (Harbor or equivalent)
- Images signed with Cosign
- Vulnerability scanning (Trivy) in CI pipeline
- Immutable tags (git SHA)

### 7.4 Kubernetes (National DC)

```yaml
# Deployment manifest pattern
apiVersion: apps/v1
kind: Deployment
metadata:
  name: attendance-service
  namespace: inwp-mohe
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    maxUnavailable: 1
  template:
    spec:
      containers:
      - name: attendance-service
        image: registry.inwp.iq/mohe/attendance-service:abc123
        ports:
        - containerPort: 8080
        - containerPort: 9090  # metrics
        env:
        - name: DB_DSN
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: attendance-dsn
        - name: SYNC_ENDPOINT
          value: "https://sync.inwp.iq/relay/baghdad"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "1"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
```

### 7.5 Docker Compose (Edge Node)

```yaml
version: "3.8"
services:
  postgres:
    image: postgres:16-alpine
    volumes:
      - pgdata:/var/lib/postgresql/data
      - ./init-scripts:/docker-entrypoint-initdb.d
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    networks:
      - inwp-lan
    deploy:
      resources:
        limits:
          memory: 512M

  sync-engine:
    image: registry.inwp.iq/sync-engine:latest
    depends_on: [postgres]
    ports:
      - "8443:8443"
    secrets:
      - sync_cert
      - sync_key
    networks:
      - inwp-lan
    environment:
      DB_DSN: "postgres://inwp:${DB_PASSWORD}@postgres:5432/inwp_sync"
      SYNC_MODE: "edge"
      REGIONAL_HUB: "https://sync.mohe.baghdad.inwp.iq"

  attendance-service:
    image: registry.inwp.iq/attendance-service:latest
    depends_on: [postgres]
    ports:
      - "8080:8080"
    networks:
      - inwp-lan
      - biometric-vlan
    environment:
      DB_DSN: "postgres://inwp:${DB_PASSWORD}@postgres:5432/inwp_attendance"
      BIOMETRIC_DEVICES: "192.168.50.0/24"

volumes:
  pgdata:

networks:
  inwp-lan:
    driver: bridge
    ipam:
      config:
        - subnet: "172.20.0.0/16"
  biometric-vlan:
    driver: macvlan
    ipam:
      config:
        - subnet: "192.168.50.0/24"
```

### 7.6 Infrastructure Requirements

| Tier | CPU | RAM | Storage | Network |
|---|---|---|---|---|
| National DC (K8s node) | 32+ cores | 128+ GB | SSD/NVMe 4TB+ | 10 Gbps redundant |
| Regional Hub | 8 cores | 32 GB | SSD 1TB | 1 Gbps |
| Edge Node (large site) | 4 cores | 16 GB | SSD 500GB | 100 Mbps LAN |
| Edge Node (small site) | 2 cores | 8 GB | SSD 250GB | 10 Mbps LAN / 4G |
| Mobile device | ARM64 | 4+ GB | 64GB+ flash | WiFi / 4G |

---

## 8. API Strategy

### 8.1 API Surface Overview

| Interface | Protocol | Audience | Authentication |
|---|---|---|---|
| **Service APIs** | gRPC (protobuf) | Internal services | mTLS + service JWT |
| **Employee APIs** | REST (JSON) | Mobile app | OAuth2 + OIDC (user JWT) |
| **Admin APIs** | REST (JSON) | Web portal | OAuth2 + OIDC (user JWT) |
| **Sync APIs** | HTTPS + WS | Edge ↔ Hub | mTLS + sync JWT |
| **Device APIs** | REST (JSON) | Biometric devices | mTLS (device cert) |
| **Event APIs** | NATS / Pulsar | Service ↔ Service | mTLS + event JWT |

### 8.2 API Versioning

- All REST APIs versioned via URL prefix (`/api/v1/`, `/api/v2/`)
- gRPC services versioned via package name (`inwp.attendance.v1`)
- Minimum 6-month support for old versions
- Deprecation header on responses: `Sunset: Sat, 30 Nov 2026 23:59:59 GMT`

### 8.3 REST API Conventions

```
GET    /api/v1/ministries/{ministry_id}/employees
GET    /api/v1/ministries/{ministry_id}/employees/{id}
POST   /api/v1/ministries/{ministry_id}/employees
PATCH  /api/v1/ministries/{ministry_id}/employees/{id}
DELETE /api/v1/ministries/{ministry_id}/employees/{id}

GET    /api/v1/ministries/{ministry_id}/attendance/clock-events
POST   /api/v1/ministries/{ministry_id}/attendance/clock-in
POST   /api/v1/ministries/{ministry_id}/attendance/clock-out

GET    /api/v1/ministries/{ministry_id}/leaves
POST   /api/v1/ministries/{ministry_id}/leaves
PATCH   /api/v1/ministries/{ministry_id}/leaves/{id}/approve
PATCH   /api/v1/ministries/{ministry_id}/leaves/{id}/reject
```

### 8.4 API Design Rules

| Rule | Specification |
|---|---|
| **Idempotency** | All mutating endpoints support `Idempotency-Key` header (idempotency key stored for 24h) |
| **Pagination** | Cursor-based (`cursor` + `limit`); no offset pagination for consistency |
| **Error format** | [RFC 7807](https://tools.ietf.org/html/rfc7807) Problem Details |
| **Rate limiting** | `X-RateLimit-*` headers; per-ministry + per-user quotas |
| **Request ID** | Every response includes `X-Request-ID` for tracing |
| **Conditional requests** | `ETag` + `If-None-Match` for read endpoints |
| **Bulk operations** | JSON array POST for batch endpoints (max 1000 items) |
| **Async operations** | `202 Accepted` + `Location: /operations/{id}` for long-running tasks |

### 8.5 gRPC Service Definitions

```protobuf
// Example: Attendance Service
package inwp.attendance.v1;

service AttendanceService {
    rpc ClockIn(ClockInRequest) returns (ClockInResponse);
    rpc ClockOut(ClockOutRequest) returns (ClockOutResponse);
    rpc GetAttendanceRecord(GetAttendanceRequest) returns (AttendanceRecord);
    rpc ListAttendanceRecords(ListAttendanceRequest) returns (stream AttendanceRecord);
    rpc SyncAttendance(SyncAttendanceRequest) returns (SyncAttendanceResponse);
}

message ClockInRequest {
    string employee_id = 1;
    string ministry_id = 2;
    string site_id = 3;
    string device_id = 4;
    google.protobuf.Timestamp event_time = 5;
    optional BiometricVerification biometric = 6;
    string idempotency_key = 7;
}

message ClockInResponse {
    string event_id = 1;
    google.protobuf.Timestamp recorded_at = 2;
    string sync_status = 3;  // "local" | "synced"
}
```

### 8.6 Event API (AsyncAPI)

Event contracts are documented using AsyncAPI 2.0 and stored in `docs/api/asyncapi/`.

```yaml
asyncapi: 2.6.0
info:
  title: INWP Attendance Events
  version: 1.0.0
channels:
  attendance/clock-in:
    publish:
      message:
        $ref: '#/components/messages/ClockInCreated'
    subscribe:
      message:
        $ref: '#/components/messages/ClockInCreated'
components:
  messages:
    ClockInCreated:
      payload:
        type: object
        properties:
          event_id:
            type: string
            format: uuid
          employee_id:
            type: string
            format: uuid
          event_time:
            type: string
            format: date-time
```

---

## 9. Identity Architecture

### 9.1 Identity Model

```
+------------------------------------------------------------------+
|                        FEDERATED IDENTITY                         |
|                                                                   |
|  +------------------+  +------------------+  +------------------+ |
|  | Ministry of      |  | Ministry of      |  | Ministry of      | |
|  | Education Realm  |  | Health Realm     |  | Oil Realm        | |
|  |                  |  |                  |  |                  | |
|  | mohe.inwp.iq     |  | moh.inwp.iq      |  | moo.inwp.iq      | |
|  | - Employees      |  | - Employees      |  | - Employees      | |
|  | - HR Admins      |  | - Doctors        |  | - Engineers      | |
|  | - Site Managers  |  | - Nurses         |  | - Field Workers  | |
|  | - Devices        |  | - Patients (lim) |  | - Contractors    | |
|  +------------------+  +------------------+  +------------------+ |
+------------------------------------------------------------------+
                               |
                    Identity Service (IdP)
                    +-------------------------------+
                    | OAuth2 / OIDC Provider        |
                    | SCIM Provisioning             |
                    | SAML Bridge (legacy)          |
                    | User Federation               |
                    | Realm Management              |
                    | Device Enrollment             |
                    | Session Management            |
                    +-------------------------------+
```

### 9.2 Realm Hierarchy

```
/ministries
  /{ministry_id}
    /sites
      /{site_id}
        /roles
          - site_admin
          - hr_operator
          - attendance_operator
          - leave_approver
          - viewer
    /roles
      - ministry_admin
      - ministry_hr
      - ministry_auditor
    /groups
      /{group_id}
        - payroll_team
        - inspection_team
/national
  /roles
    - national_admin
    - national_auditor
    - system_operator
```

### 9.3 Authentication Methods

| Method | MFA Support | Offline Capable | Use Case |
|---|---|---|---|
| Password + TOTP | Yes | No | Ministry admins, web portal |
| Smart card (PKCS#11) | Inherent | Yes (local cert validation) | National admins, auditors |
| Biometric (fingerprint) | Yes (with PIN) | Yes (local template match) | Field workers, attendance |
| Biometric (face) | Yes (with PIN) | Yes (on-device) | Mobile workers |
| Certificate (device) | Inherent | Yes (local validation) | IoT devices, biometric terminals |
| SMS OTP | Second factor | No | Password reset, recovery |

### 9.4 Session Model

```sql
CREATE TABLE idm.sessions (
    session_id      UUID PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES idm.users(id),
    ministry_id     UUID NOT NULL,
    device_id       UUID REFERENCES idm.devices(id),
    auth_method     TEXT NOT NULL,          -- 'password+totp', 'biometric', 'certificate'
    access_token    TEXT NOT NULL,
    access_expires  TIMESTAMPTZ NOT NULL,
    refresh_token   TEXT NOT NULL,
    refresh_expires TIMESTAMPTZ NOT NULL,
    claims          JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    terminated_at   TIMESTAMPTZ,
    termination_reason TEXT
);

-- Offline session cache (synced to edge)
CREATE TABLE idm.offline_sessions (
    user_id         UUID NOT NULL,
    ministry_id     UUID NOT NULL,
    device_id       UUID NOT NULL,
    cached_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL,
    cached_claims   JSONB NOT NULL,
    PRIMARY KEY (user_id, device_id)
);
```

### 9.5 Device Enrollment

```
Device (e.g., biometric terminal)
    |
    |---(1) Generate device keypair (TPM/secure element)
    |
    |---(2) POST /api/v1/devices/enroll
    |       {
    |         "device_id": "bio-001",
    |         "device_type": "fingerprint_scanner",
    |         "manufacturer": "zkteco",
    |         "model": "MB460",
    |         "site_id": "...",
    |         "attestation": {
    |           "public_key": "...",
    |           "attestation_blob": "...",   // TPM/nonce signed by manufacturer cert
    |           "certificate_chain": ["..."]
    |         }
    |       }
    |
Identity Service
    |---(3) Verify device attestation (verify cert chain, challenge nonce)
    |---(4) Generate device identity certificate (signed by Device CA)
    |---(5) Register device in idm.devices
    |---(6) Return device certificate + CA bundle + sync token
    |
Device
    |---(7) Store certificate in secure element
    |---(8) Begin operation with mTLS
```

### 9.6 Identity Sync

- User/role/group changes are events (`inwp.identity.v1.*`)
- Events flow to all nodes via sync engine
- Edge nodes maintain local cache of active users for offline auth
- Offline sessions cached for configurable duration (default 24h)
- Device trust scores synced via dedicated `sync.v1.device.trust_updated` events

---

## 10. Device Integration Model

### 10.1 Device Classes

| Class | Examples | Connectivity | Offline Capability |
|---|---|---|---|
| **Biometric terminals** | Fingerprint scanner, face recognition terminal | LAN / RS485 | Full (local template match, store-and-forward) |
| **Attendance terminals** | Card readers, PIN pads | LAN / USB | Full (local event buffer) |
| **Mobile devices** | Employee smartphones, tablets | WiFi / 4G | Full (on-device DB, sync engine) |
| **IoT sensors** | Badge readers, door sensors | Zigbee / WiFi | Partial (gateway buffered) |
| **Printers/labelers** | ID card printers | LAN | Minimal (print queue only) |
| **Servers (edge)** | Local site server | LAN | Full (entire service stack) |

### 10.2 Device Communication Model

```
+-------------------+         +-------------------+
| Biometric Device  |         | Edge Node         |
| (ZKteco MB460)    |         | (Docker Host)     |
|                   |  gRPC   |                   |
| /attendance/clock | <-----> | attendance-service|
| -in               |  mTLS   |                   |
| -out              |         +-------------------+
| -verify           |                 |
+-------------------+                 | LAN
                                      |
                      +-------------------+
                      | sync-engine       |
                      | (peer mesh)       |
                      +-------------------+
```

### 10.3 Device Abstraction Layer

```go
// platform-core device abstraction
type Device interface {
    ID() string
    Type() DeviceType
    Status() DeviceStatus
    Connect(ctx context.Context) error
    Disconnect() error
    Health() (*DeviceHealth, error)

    // Attendance-specific
    ClockIn(ctx context.Context, employeeID string, opts ...ClockOption) (*ClockEvent, error)
    ClockOut(ctx context.Context, employeeID string, opts ...ClockOption) (*ClockEvent, error)
    VerifyBiometric(ctx context.Context, employeeID string) (*BiometricResult, error)

    // Management
    FirmwareVersion() string
    UpdateFirmware(ctx context.Context, version string) error
    Diagnostics() (*DeviceDiagnostics, error)
}
```

### 10.4 Device Protocol Adapters

- **ZKteco SDK** — wrapped in gRPC adapter service for fingerprint terminals
- **Suprema SDK** — wrapped for face recognition terminals
- **HID Global** — card reader integration via USB/Serial bridge
- **Custom RS485** — legacy device support via serial-to-Ethernet gateway
- **Generic HTTP** — modern devices with REST APIs
- **MQTT** — IoT sensor class devices

### 10.5 Device Trust Model

| Trust Level | Criteria | Permissions |
|---|---|---|
| **TRUSTED** | Attestation verified, firmware current, no anomalies | Full read/write |
| **PROVISIONAL** | Enrolled but attestation incomplete | Read-only, limited write |
| **DEGRADED** | Expired certificate, outdated firmware | Read-only, alert generated |
| **SUSPENDED** | Known vulnerability, anomaly detected | Blocked, admin notification |
| **REVOKED** | Compromised, decommissioned | All access blocked, audit event |

### 10.6 Device-Specific Sync

- Biometric templates are NEVER synced across the WAN
- Templates remain local to the enrolling device/site
- Employee-to-template mappings are synced (no template binary data)
- Device config (timeouts, sensitivity, policies) syncs from edge node to device
- Clock events sync from device → edge node → regional hub → national DC

### 10.7 Device Inventory Database

```sql
CREATE TABLE idm.devices (
    device_id       UUID PRIMARY KEY,
    device_type     TEXT NOT NULL,
    manufacturer    TEXT NOT NULL,
    model           TEXT NOT NULL,
    serial_number   TEXT NOT NULL UNIQUE,
    ministry_id     UUID NOT NULL,
    site_id         UUID NOT NULL,
    firmware_version TEXT,
    certificate_serial TEXT,
    trust_level     TEXT NOT NULL DEFAULT 'provisional',
    trust_score     REAL DEFAULT 0.0,
    last_seen_at    TIMESTAMPTZ,
    enrolled_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    retired_at      TIMESTAMPTZ,
    metadata        JSONB
);

CREATE TABLE device.clock_events (
    event_id        UUID PRIMARY KEY,
    device_id       UUID NOT NULL REFERENCES idm.devices(device_id),
    employee_id     UUID NOT NULL,
    event_type      TEXT NOT NULL,     -- 'clock_in', 'clock_out', 'break_start', 'break_end'
    event_time      TIMESTAMPTZ NOT NULL,
    biometric_match REAL,             -- confidence score 0.0-1.0
    biometric_template_hash BYTEA,    -- hash only, NOT the template
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    synced_at       TIMESTAMPTZ
);
```

---

## Appendix A: Technology Stack Summary

| Category | Technology | Justification |
|---|---|---|
| **Core language** | Go | Single binary, fast compilation, excellent concurrency, easy cross-compile for ARM64 edge |
| **Sync engine** | Rust | Memory safety critical for sync correctness, zero-cost abstractions, precise control over serialization |
| **Database** | PostgreSQL 16 | Mandated; logical replication, RLS, partitioning, pg_notify, pg_cron |
| **Service mesh** | mTLS direct (no mesh in MVP) | Reduced complexity for air-gapped deployments |
| **Event bus (national)** | NATS | Lightweight, at-least-once delivery, exactly-once semantics available, cluster mode |
| **Event bus (edge)** | pg_notify | Zero additional dependencies, works offline |
| **Serialization** | Protocol Buffers (binary), JSON (human) | Protobuf for gRPC, JSON for REST |
| **Sync protocol** | Custom (merkle tree + delta) | Requirements-specific; no off-the-shelf solution meets all constraints |
| **Container runtime** | Docker (edge), Kubernetes (national) | Docker mandated for edge simplicity; K8s for national DC orchestration |
| **Identity** | OAuth2 + OIDC (custom IdP) | No external IdP dependency; must work fully offline |
| **Secrets** | HashiCorp Vault (national), Docker secrets (edge) | Tier-appropriate secret management |
| **Monitoring** | Prometheus + Grafana (national), local metrics (edge) | Pull model for DC, push model for edge |
| **Logging** | Structured JSON to stdout, Fluentd aggregation | Cloud-native logging pattern |
| **CI/CD** | GitLab CI / GitHub Actions | Based on repo hosting |

## Appendix B: Port Mapping

| Service | Internal Port | External Port | Protocol |
|---|---|---|---|
| identity-service | 50051 | 443 (via proxy) | gRPC |
| identity-service (REST) | 8081 | 443 | HTTPS |
| sync-engine | 50052 | 8443 | HTTPS/WS |
| sync-engine (LAN mesh) | 50053 | - | gRPC |
| audit-ledger | 50054 | 443 | gRPC |
| attendance-service | 50055 | 8080 | gRPC |
| leave-service | 50056 | 8082 | gRPC |
| notification-service | 50057 | 8083 | gRPC |
| PostgreSQL | 5432 | - | TCP |
| Prometheus metrics | 9090 | - | HTTP |
| Health/readiness | 8080 | - | HTTP |

## Appendix C: Key Architectural Decisions (ADRs)

See `docs/adr/` for individual Architecture Decision Records. Key decisions summarized:

| ADR | Decision | Rationale |
|---|---|---|
| ADR-001 | PostgreSQL for all persistence | Mandated; no polyglot persistence |
| ADR-002 | Go as primary service language | Single binary, fast compilation, ARM64 support |
| ADR-003 | Rust for sync engine | Memory safety for correctness-critical sync code |
| ADR-004 | Custom sync protocol | Requirements-specific (offline-first, LAN mesh, air-gap) |
| ADR-005 | NATS for national event bus | Lightweight, at-least-once, cluster mode |
| ADR-006 | pg_notify for edge event bus | Zero additional dependencies, works offline |
| ADR-007 | Event sourcing + CQRS for audit | Immutable audit trail requirement |
| ADR-008 | mTLS for all service-to-service | Zero-trust mandate |
| ADR-009 | OAuth2 + OIDC for identity | Industry standard, offline-capable with local sessions |
| ADR-010 | Per-ministry schema isolation | Multi-tenant requirement without shared schema complexity |
| ADR-011 | CRDT/LWW conflict resolution | Eventual consistency without consensus overhead |
| ADR-012 | 3-tier deployment (edge/region/national) | Government-grade reliability, regional autonomy |
