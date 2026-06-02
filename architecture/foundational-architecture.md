# INWP Foundational Architecture

> Iraq National Workforce Platform — A sovereign, distributed, offline-first workforce operating system for Iraq.
> Designed for government-grade reliability, zero-trust security, and autonomous edge operation.

---

## 1. High-Level Architecture

### 1.1 System Topology

INWP operates across three physical tiers, each fully autonomous when disconnected:

```
+--------------------------------------------------------------------+
|                      NATIONAL TIER (Baghdad DC)                     |
|  +----------------+  +----------------+  +----------------------+   |
|  | Identity IdP   |  | Audit Ledger   |  | National Sync Hub   |   |
|  | (Keycloak)     |  | (Hash Chain)   |  | (Anti-Entropy)      |   |
|  +----------------+  +----------------+  +----------------------+   |
|  +----------------+  +----------------+  +----------------------+   |
|  | Analytics Svc  |  | Policy Engine  |  | Workforce State      |   |
|  | (MinIO lake)   |  | (Central Rules)|  | Engine (CQRS)       |   |
|  +----------------+  +----------------+  +----------------------+   |
|  +----------------+  +----------------+  +----------------------+   |
|  | NATS Cluster   |  | Redis Cache    |  | PostgreSQL (Patroni) |   |
|  | (Event Bus)    |  | (Session,Lock) |  | (Primary Cluster)   |   |
|  +----------------+  +----------------+  +----------------------+   |
+--------------------------------------------------------------------+
           | Encrypted WAN (mTLS 1.3)
           v
+--------------------------------------------------------------------+
|                   REGIONAL TIER (x18 Governorates)                  |
|  +----------------+  +----------------+  +----------------------+   |
|  | Regional Sync  |  | Regional Audit |  | Regional Identity    |   |
|  | Relay          |  | Buffer         |  | Cache (Redis)        |   |
|  +----------------+  +----------------+  +----------------------+   |
|  +----------------+  +----------------+  +----------------------+   |
|  | Attendance Agg |  | Leave Service  |  | Device Gateway       |   |
|  | (Regional)     |  | (Regional)     |  | (Device Mgmt)        |   |
|  +----------------+  +----------------+  +----------------------+   |
|  | PostgreSQL (Streaming Replica) | NATS Leaf Node | MinIO Cache | |
+--------------------------------------------------------------------+
           | LAN / Mesh / Encrypted Radio
           v
+--------------------------------------------------------------------+
|                     EDGE TIER (1000+ Sites)                         |
|  +----------------+  +----------------+  +----------------------+   |
|  | Local PG       |  | Sync Engine    |  | Attendance Service   |   |
|  | (Offline Store)|  | (Merkle Tree)  |  | (Offline-first)      |   |
|  +----------------+  +----------------+  +----------------------+   |
|  +----------------+  +----------------+  +----------------------+   |
|  | Leave Approver |  | Device Gateway |  | Biometric Devices    |   |
|  | (Offline)      |  | (Edge Proxy)   |  | (ZKteco, Suprema)    |   |
|  +----------------+  +----------------+  +----------------------+   |
|  | Redis (Cache)  |  | MinIO (Edge)   |  | pg_notify (Events)   |  |
+--------------------------------------------------------------------+
```

### 1.2 Architectural Principles

| Principle | Application |
|---|---|
| **Offline-first** | Every node functions fully disconnected. Local PG is source of truth until sync. |
| **Event-driven** | All state changes produce CloudEvents 1.0 events. Services communicate through events exclusively. |
| **Local-first** | Data owned by creating node. National DC is a convergent replica, not authoritative source. |
| **Anti-entropy** | Sync uses merkle-tree reconciliation with delta compression. No single-master topology. |
| **Zero-trust** | mTLS at transport, JWT at application, Ed25519 at data layer, TPM at device layer. |
| **Tenant isolation** | Each ministry is a tenant with isolated schema, encryption keys, and authentication realms. |
| **No SPOF** | Every service runs min 2 replicas at each tier. Edge nodes operate independently. |
| **Anti-fragile** | System strengthens under stress: chaos engineering, automatic healing, graceful degradation. |

### 1.3 Communication Patterns

| Pattern | Protocol | Use Case |
|---|---|---|
| Service-to-service (internal) | gRPC + mTLS | Query, command, internal RPC |
| Service-to-service (events) | NATS (national), pg_notify (edge) | Event publish-subscribe |
| Edge → Hub | HTTPS + mTLS + zstd | Sync upload, heartbeat |
| Hub → Edge | WebSocket + mTLS | Sync push, commands |
| LAN peer-to-peer | mDNS + gRPC | Office mesh sync |
| Mobile → Edge/Cloud | HTTPS + Certificate Pinning | Employee operations |
| Admin → Web Portal | HTTPS + OAuth2 + OIDC | Dashboard, reports |
| Device → Gateway | gRPC/HTTP + mTLS | Biometric device integration |
| Cache queries | Redis RESP (TLS) | Session cache, rate limits, locks |

---

## 2. Suggested Folder Structure

```
inwp/
├── .github/
│   └── workflows/
│       ├── ci.yml                    # Build + test all services
│       ├── cd-national.yml           # Deploy to national DC
│       └── cd-edge.yml               # Deploy to edge nodes
│
├── platform-core/                    # Shared kernel (Go module)
│   ├── pkg/
│   │   ├── domain/                   # Base types, value objects
│   │   │   ├── identifiers.go        # MinistryID, SiteID, EmployeeID, DeviceID
│   │   │   ├── base_event.go         # DomainEvent interface
│   │   │   ├── sync_metadata.go      # SyncMetadata, SyncStatus
│   │   │   ├── crypto.go             # Signing, verification, encryption
│   │   │   └── validation.go         # ValidationRule, ValidationResult
│   │   ├── schemas/                  # JSON Schema registry (CloudEvents)
│   │   │   ├── v1/
│   │   │   │   ├── attendance/
│   │   │   │   ├── identity/
│   │   │   │   ├── leave/
│   │   │   │   ├── audit/
│   │   │   │   ├── sync/
│   │   │   │   ├── policy/
│   │   │   │   ├── workforce/
│   │   │   │   └── analytics/
│   │   │   └── schema-registry.json
│   │   └── proto/                    # Shared protobuf definitions
│   │       ├── inwp/
│   │       │   ├── common/
│   │       │   ├── attendance/
│   │       │   ├── identity/
│   │       │   ├── leave/
│   │       │   ├── sync/
│   │       │   ├── audit/
│   │       │   ├── policy/
│   │       │   ├── workforce/
│   │       │   └── analytics/
│   │       └── google/               # Well-known types
│   ├── go.mod
│   └── Makefile
│
├── identity-service/
│   ├── cmd/
│   │   └── identity-service/
│   │       └── main.go
│   ├── internal/
│   │   ├── domain/                   # User, Device, Realm, Session aggregates
│   │   ├── application/              # Use cases
│   │   ├── infrastructure/
│   │   │   ├── postgres/             # UserRepo, DeviceRepo, RealmRepo
│   │   │   ├── keycloak/             # Keycloak admin client adapter
│   │   │   ├── redis/                # Session cache, rate limiter
│   │   │   └── eventbus/             # NATS publisher + consumer
│   │   └── interfaces/
│   │       ├── rest/                 # OAuth2/OIDC endpoints
│   │       ├── grpc/                 # Internal gRPC
│   │       └── scim/                 # SCIM provisioning
│   ├── migrations/
│   ├── Dockerfile
│   ├── go.mod
│   └── Makefile
│
├── attendance-service/               # (already implemented)
│   ├── cmd/
│   ├── internal/
│   │   ├── domain/                   # ClockEvent, Shift, Policy, Exception
│   │   ├── application/
│   │   ├── infrastructure/
│   │   │   ├── postgres/
│   │   │   ├── eventbus/
│   │   │   └── sync/
│   │   └── interfaces/
│   │       ├── rest/
│   │       └── grpc/
│   ├── migrations/
│   ├── Dockerfile
│   └── go.mod
│
├── leave-service/
│   ├── cmd/
│   ├── internal/
│   │   ├── domain/                   # LeaveRequest, LeaveBalance, AccrualPolicy
│   │   ├── application/
│   │   ├── infrastructure/
│   │   │   ├── postgres/
│   │   │   └── eventbus/
│   │   └── interfaces/
│   │       ├── rest/
│   │       └── grpc/
│   ├── migrations/
│   ├── Dockerfile
│   └── go.mod
│
├── workforce-state-engine/           # CQRS state projection service
│   ├── cmd/
│   ├── internal/
│   │   ├── domain/                   # EmployeeState, WorkHistory, Timesheet
│   │   ├── application/
│   │   ├── infrastructure/
│   │   │   ├── postgres/             # Materialized views
│   │   │   ├── redis/                # Real-time state cache
│   │   │   └── eventbus/             # Event consumers
│   │   └── interfaces/
│   │       ├── rest/                 # Query API for computed state
│   │       └── grpc/
│   ├── migrations/
│   ├── Dockerfile
│   └── go.mod
│
├── sync-engine/                      # Rust sync engine
│   ├── src/
│   │   ├── domain/                   # MerkleTree, SyncBatch, SyncNode
│   │   ├── application/
│   │   ├── infrastructure/
│   │   │   ├── postgres/
│   │   │   └── transport/            # HTTPS, WS, mDNS
│   │   └── interfaces/
│   │       └── grpc/                 # Sync protocol gRPC
│   ├── migrations/
│   ├── Cargo.toml
│   └── Dockerfile
│
├── audit-ledger/
│   ├── cmd/
│   ├── internal/
│   │   ├── domain/                   # LedgerEntry, Seal, IntegrityProof
│   │   ├── application/
│   │   ├── infrastructure/
│   │   │   ├── postgres/             # Append-only, hash chain
│   │   │   └── minio/                # Archive cold storage
│   │   └── interfaces/
│   │       └── grpc/
│   ├── migrations/
│   ├── Dockerfile
│   └── go.mod
│
├── notification-service/
│   ├── cmd/
│   ├── internal/
│   │   ├── domain/                   # Notification, Template, Delivery
│   │   ├── application/
│   │   ├── infrastructure/
│   │   │   ├── postgres/
│   │   │   └── channels/             # Push (FCM), SMS, Email, On-screen
│   │   └── interfaces/
│   │       └── grpc/
│   ├── migrations/
│   ├── Dockerfile
│   └── go.mod
│
├── policy-engine/                    # Centralized policy definition & evaluation
│   ├── cmd/
│   ├── internal/
│   │   ├── domain/                   # Policy, Rule, Condition, Action
│   │   ├── application/
│   │   ├── infrastructure/
│   │   │   ├── postgres/
│   │   │   └── redis/                # Policy cache (hot path)
│   │   └── interfaces/
│   │       ├── rest/                 # Policy CRUD dashboard
│   │       └── grpc/                 # Policy evaluation
│   ├── migrations/
│   ├── Dockerfile
│   └── go.mod
│
├── device-gateway/                   # Device abstraction & protocol bridging
│   ├── cmd/
│   ├── internal/
│   │   ├── domain/                   # Device, ProtocolAdapter, Firmware
│   │   ├── application/
│   │   ├── infrastructure/
│   │   │   ├── postgres/
│   │   │   ├── redis/                # Device session cache
│   │   │   └── adapters/             # ZKteco, Suprema, HID, RS485
│   │   └── interfaces/
│   │       ├── rest/                 # Device-facing API
│   │       └── grpc/                 # Service-facing API
│   ├── Dockerfile
│   └── go.mod
│
├── analytics-service/                # Aggregation, reporting, BI
│   ├── cmd/
│   ├── internal/
│   │   ├── domain/                   # Report, Dashboard, Metric, Alert
│   │   ├── application/
│   │   ├── infrastructure/
│   │   │   ├── postgres/             # Aggregated tables
│   │   │   ├── minio/                # Data lake (Parquet files)
│   │   │   └── eventbus/             # Event consumers
│   │   └── interfaces/
│   │       └── rest/                 # Dashboard API
│   ├── Dockerfile
│   └── go.mod
│
├── deployment/
│   ├── national/                     # Kubernetes manifests
│   │   ├── kustomization.yaml
│   │   ├── namespaces/
│   │   ├── deployments/
│   │   ├── services/
│   │   ├── configmaps/
│   │   └── secrets/
│   ├── regional/                     # Docker Compose per hub
│   │   ├── docker-compose.yml
│   │   ├── nginx/
│   │   └── prometheus/
│   └── edge/                         # Docker Compose per site
│       ├── docker-compose.yml
│       ├── init-scripts/
│       └── .env.template
│
├── infrastructure/                   # Shared infra config
│   ├── postgres/
│   │   ├── patroni.yml
│   │   ├── pg_hba.conf
│   │   └── postgresql.conf
│   ├── nats/
│   │   └── nats.conf
│   ├── redis/
│   │   └── redis.conf
│   ├── minio/
│   │   └── minio.env
│   └── keycloak/
│       ├── realm-import/
│       └── Dockerfile
│
├── scripts/
│   ├── bootstrap-node.sh             # Edge node provisioning
│   ├── bootstrap-region.sh           # Regional hub provisioning
│   ├── generate-certs.sh             # Certificate generation
│   └── chaos-test.sh                 # Chaos engineering scenarios
│
├── docs/
│   ├── adr/                          # Architecture Decision Records
│   ├── api/                          # OpenAPI + AsyncAPI specs
│   ├── event-models/                 # Event catalog
│   ├── security/                     # Threat models, pentest reports
│   ├── sync-protocol/                # Protocol specification
│   └── governance/                   # Operational runbooks
│
├── security/
│   ├── ca/                           # Certificate authority scripts
│   ├── policies/                     # OPA/Rego policies
│   └── vault/                        # Vault configuration
│
├── governance/
│   ├── rbac-models/                  # Role definitions per ministry
│   └── compliance/                   # Audit compliance templates
│
├── .github/
│   ├── CODEOWNERS
│   └── SECURITY.md
│
├── AGENTS.md                         # Engineering rules
├── LICENSE                           # Apache 2.0
├── README.md
└── Makefile                          # Top-level build orchestration
```

---

## 3. Service Boundaries

### 3.1 Service Map & Ownership

```
                    +--------------------------------------+
                    |          platform-core                |
                    | (Shared Kernel: types, schemas,      |
                    |  proto, crypto, validation)          |
                    +--------------------------------------+
                               | depends on
                               v
+----------+  +----------+ +----------+ +----------+ +----------+
| identity |  |   sync   | |  audit   | | notifica | |  policy  |
| -service |  | -engine  | | -ledger  | | -tion    | | -engine  |
+----------+  +----------+ +----------+ +----------+ +----------+
                               |
         +---------------------+---------------------+
         v                                           v
+------------------+                        +------------------+
| attendance-      |                        | leave-service    |
| service          |                        |                  |
+------------------+                        +------------------+
         |                                           |
         v                                           v
+------------------+                        +------------------+
| device-gateway   |                        | workforce-state  |
| (biometric,hw)   |                        | -engine (CQRS)   |
+------------------+                        +------------------+
                                                     |
                                                     v
                                            +------------------+
                                            | analytics-       |
                                            | service          |
                                            +------------------+
```

### 3.2 Service Specifications

#### `platform-core` (Shared Kernel)

| Attribute | Value |
|---|---|
| Language | Go |
| Responsibility | Domain types, event schemas, protobuf, crypto, validation |
| Output | Shared library (Go module), JSON Schema catalog, .proto files |
| Dependencies | Zero external deps beyond Go stdlib |
| Deployment | Embedded dependency; not independently deployed |

#### `identity-service`

| Attribute | Value |
|---|---|
| Language | Go |
| Database | PostgreSQL (`idm_*`), Redis (session cache) |
| External IdP | Keycloak (federation bridge, OIDC provider) |
| Responsibilities | User registration, auth (password, biometric, cert), RBAC/ABAC, ministry realms, session mgmt, device enrollment, SCIM provisioning |
| APIs | gRPC (internal), REST OAuth2/OIDC (external), SCIM |
| Key Features | Ministry-scoped realms, federated IdP bridging, offline auth sessions, hardware-backed keys |
| Sync | User/role/group changes → outbound events |

#### `attendance-service`

| Attribute | Value |
|---|---|
| Language | Go |
| Database | PostgreSQL (`att_*`, partitioned by month) |
| Cache | Redis (policy cache, rate limits) |
| Responsibilities | Clock-in/out, shift management, overtime calc, policy enforcement, biometric verification delegation, exception handling |
| APIs | gRPC (internal), REST (mobile/web client) |
| Key Features | Full offline operation, batch sync, policy-as-config, exception detection, duplicate prevention |
| Sync Priority | HIGH (payroll-critical) |

#### `leave-service`

| Attribute | Value |
|---|---|
| Language | Go |
| Database | PostgreSQL (`lv_*`, partitioned by ministry) |
| Responsibilities | Leave requests, approval workflows, balance tracking, accrual policies, ministry-specific types (Hajj, Umrah, etc.) |
| APIs | gRPC + REST |
| Key Features | Hierarchical approval chains, policy-driven accrual, offline request/approve, fiscal year management |
| Sync Priority | MEDIUM |

#### `workforce-state-engine`

| Attribute | Value |
|---|---|
| Language | Go |
| Database | PostgreSQL (materialized views), Redis (real-time state) |
| Pattern | CQRS — consumes events, projects state |
| Responsibilities | Employee state projection, work history, timesheet computation, eligibility checks, real-time dashboards |
| APIs | gRPC (queries), REST (dashboards) |
| Key Features | Event-sourced state, real-time Redis pub/sub, aggregated views, historical snapshots |
| Sync Priority | LOW (read model, eventual consistency) |

#### `sync-engine`

| Attribute | Value |
|---|---|
| Language | Rust |
| Database | PostgreSQL (`sync_*`: merkle trees, checkpoints, batch logs) |
| Responsibilities | Merkle tree reconciliation, delta computation, conflict resolution (CRDT/LWW/merge), compression (zstd), bandwidth management, LAN discovery (mDNS) |
| Protocols | HTTPS + WebSocket (WAN), gRPC + mDNS (LAN) |
| Key Features | Differential sync, checkpoint/resume, bandwidth throttling, ministry-level sync policies, offline queue, schema negotiation |
| Sync | Self-synchronizing (meta-sync protocol) |

#### `audit-ledger`

| Attribute | Value |
|---|---|
| Language | Go |
| Database | PostgreSQL (`ledger_*`, append-only hash chain) |
| Archive | MinIO (cold storage for aged-out entries) |
| Responsibilities | Immutable event ingestion, hash-chain verification, tamper-evident seal generation, compliance queries, retention management, periodic merkle root publication |
| APIs | gRPC (ingest + query), REST (read-only compliance) |
| Key Features | Cryptographic chaining (prev_hash), periodic seals, WAL-based replication, no UPDATE/DELETE |

#### `notification-service`

| Attribute | Value |
|---|---|
| Language | Go |
| Database | PostgreSQL (`notif_*`: delivery tracking, templates) |
| Responsibilities | Multi-channel notification (push/FCM, SMS, email, on-screen, in-app), template engine, delivery guarantees, ministry notification policies |
| APIs | gRPC (dispatch), REST (admin) |
| Key Features | Offline-capable delivery queue, exponential backoff, read receipts, Arabic/Kurdish localization, ministry branding |

#### `policy-engine`

| Attribute | Value |
|---|---|
| Language | Go |
| Database | PostgreSQL (policy definitions), Redis (hot cache) |
| Responsibilities | Centralized policy definition, storage, evaluation, and distribution; supports attendance rules, leave policies, overtime rules, security policies |
| Evaluation Model | Rego/OPA-compatible rule language with JSON-based conditions |
| APIs | gRPC (evaluation), REST (CRUD) |
| Key Features | Versioned policies, dry-run evaluation, policy inheritance (national → ministry → site), audit trail for changes |
| Sync | Policies synced to edge as config objects |

#### `device-gateway`

| Attribute | Value |
|---|---|
| Language | Go |
| Database | PostgreSQL (device registry), Redis (device sessions) |
| Responsibilities | Device abstraction layer, protocol bridging (ZKteco, Suprema, HID, RS485, MQTT), firmware management, health monitoring, trust scoring |
| APIs | gRPC (service-facing), REST/HTTP (device-facing) |
| Key Features | Device protocol adapters, attestation verification, firmware OTA, health heartbeat, trust decay computation |
| Sync | Clock events buffered and forwarded; device config synced from edge |

#### `analytics-service`

| Attribute | Value |
|---|---|
| Language | Go |
| Database | PostgreSQL (aggregated tables) |
| Data Lake | MinIO (Parquet files, raw event archive) |
| Responsibilities | Attendance summaries, leave utilization, workforce KPIs, ministry dashboards, trend analysis, export (PDF, CSV, Excel) |
| APIs | REST (dashboard API, export) |
| Key Features | Async report generation, scheduled snapshots, pre-aggregated materialized views, role-based dashboard scoping |

### 3.3 Service Dependency Graph

```
identity-service:       [platform-core, postgres, redis, keycloak]
sync-engine:            [platform-core, postgres, nats]
audit-ledger:           [platform-core, postgres, nats, minio]
attendance-service:     [platform-core, postgres, redis, identity-service, sync-engine, policy-engine]
leave-service:          [platform-core, postgres, identity-service, sync-engine, policy-engine]
workforce-state-engine: [platform-core, postgres, redis, nats (consumer)]
notification-service:   [platform-core, postgres, identity-service]
policy-engine:          [platform-core, postgres, redis]
device-gateway:         [platform-core, postgres, redis, identity-service]
analytics-service:      [platform-core, postgres, minio, nats (consumer)]
```

---

## 4. Database Strategy

### 4.1 PostgreSQL-Only Mandate with Strategic Adjuncts

| Role | Primary | Secondary | Rationale |
|---|---|---|---|
| Persistent data | PostgreSQL 16 | — | All transactional data, events, sync metadata, audit |
| Session cache | Redis | PostgreSQL (fallback) | Sub-millisecond reads at edge; TTL-based expiry |
| Rate limiting | Redis | — | Sliding window counters, atomic increments |
| Distributed locks | Redis (Redlock) | PostgreSQL advisory locks | Cross-service coordination |
| Real-time state | Redis pub/sub | pg_notify | Workforce state engine needs sub-100ms broadcasts |
| Object storage | MinIO | PostgreSQL (metadata) | Audit archive, document attachments, Parquet data lake |
| IdP/federation | Keycloak | PostgreSQL (user store) | OIDC/OAuth2, SAML, SCIM; Keycloak backs to PG |
| Search | PostgreSQL tsvector | — | Full-text search within PG, no Elasticsearch dependency |
| Message queue | NATS (national) | pg_notify (edge) | Durable, at-least-once, cluster-mode event bus |
| Policy cache | Redis | — | Hot-path policy evaluation, <5ms p99 |

### 4.2 Schema Strategy

#### Database-per-Service

| Service | Database | Schema Prefix |
|---|---|---|
| identity-service | `inwp_identity` | `idm_` |
| sync-engine | `inwp_sync` | `sync_` |
| audit-ledger | `inwp_audit` | `ledger_` |
| attendance-service | `inwp_attendance` | `att_` |
| leave-service | `inwp_leave` | `lv_` |
| workforce-state-engine | `inwp_workforce` | `wf_` |
| notification-service | `inwp_notification` | `notif_` |
| policy-engine | `inwp_policy` | `pol_` |
| device-gateway | `inwp_device` | `dev_` |
| analytics-service | `inwp_analytics` | `an_` |

#### Multi-Tenancy via Row-Level Security

```sql
-- Every table includes ministry_id and site_id
ALTER TABLE attendance.clock_events ENABLE ROW LEVEL SECURITY;

CREATE POLICY ministry_isolation ON attendance.clock_events
    USING (ministry_id = current_setting('app.current_ministry_id')::UUID);

CREATE POLICY site_operator_access ON attendance.clock_events
    USING (
        ministry_id = current_setting('app.current_ministry_id')::UUID
        AND site_id = current_setting('app.current_site_id')::UUID
    );
```

### 4.3 Partitioning Strategy

```sql
-- Time-based partitioning for high-volume tables
CREATE TABLE attendance.clock_events (
    id UUID NOT NULL,
    employee_id UUID NOT NULL,
    ministry_id UUID NOT NULL,
    site_id UUID NOT NULL,
    event_type attendance.clock_event_type NOT NULL,
    event_time TIMESTAMPTZ NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sync_id UUID,
    sync_status attendance.sync_status DEFAULT 'local_only',
    PRIMARY KEY (id, recorded_at)
) PARTITION BY RANGE (recorded_at);

-- Monthly partitions
CREATE TABLE attendance.clock_events_2026_04
    PARTITION OF attendance.clock_events
    FOR VALUES FROM ('2026-04-01') TO ('2026-05-01');

-- Automated partition management via pg_partman
SELECT partman.create_parent(
    p_parent_table := 'attendance.clock_events',
    p_control := 'recorded_at',
    p_interval := '1 month',
    p_premake := 3
);
```

### 4.4 Redis Usage

```go
// Session cache (identity-service)
type SessionCache interface {
    SetSession(ctx, sessionID, claimsJSON, ttl) error
    GetSession(ctx, sessionID) (claims, error)
    DeleteSession(ctx, sessionID) error
}

// Rate limiter (per-ministry, per-endpoint)
type RateLimiter interface {
    Allow(ctx, key, limit, window) (bool, int, error)  // sliding window
}

// Distributed lock (leave balance, financial operations)
type DistributedLock interface {
    Acquire(ctx, lockKey, ttl) (lockID, error)
    Release(ctx, lockKey, lockID) error
}

// Real-time state (workforce-state-engine)
type StatePublisher interface {
    PublishState(ctx, channel, employeeID, stateJSON) error
    Subscribe(ctx, channel) (<-chan StateUpdate, error)
}

// Policy cache (policy-engine)
type PolicyCache interface {
    GetPolicy(ctx, ministryID, policyType) (*Policy, error)
    SetPolicy(ctx, ministryID, policyType, policy, ttl) error
    Invalidate(ctx, ministryID, policyType) error
}
```

### 4.5 MinIO Usage

| Bucket | Purpose | Retention | Encryption |
|---|---|---|---|
| `audit-archive` | Cold storage for aged-out ledger entries | 10 years | SSE-S3 (AES-256) |
| `documents` | Employee documents (certificates, IDs) | Per ministry policy | SSE-KMS (per-tenant key) |
| `analytics-lake` | Parquet exports, BI snapshots | 90 days raw, 5 years aggregated | SSE-S3 |
| `device-firmware` | OTA firmware packages | Latest 3 versions | SSE-S3 |
| `reports` | Generated PDF/CSV/Excel reports | 30 days | SSE-S3 |
| `backups` | PG WAL archives, config backups | 30 days | SSE-KMS |

### 4.6 Keycloak Integration

```yaml
# keycloak realm template
keycloak:
  realm: inwp
  ministry-realms:
    - name: mohe.inwp.iq        # Ministry of Education
      display: Ministry of Education
      scim-enabled: true
      auth-flows:
        - password + totp
        - biometric + pin
        - smart-card
      session-ttl: 15m
      refresh-ttl: 24h
    - name: moh.inwp.iq          # Ministry of Health
      display: Ministry of Health
      scim-enabled: true
      auth-flows:
        - password + totp
        - certificate
      session-ttl: 30m
      refresh-ttl: 12h

  identity-service-bridge:
    mode: "write-through"        # identity-service is authorative for user data
    sync-direction: "bidirectional"
    user-attributes:
      - national_id
      - employee_code
      - site_id
      - department
      - position
```

---

## 5. Event Taxonomy

### 5.1 Event Schema Standard

All events follow **CloudEvents 1.0** with mandatory INWP extensions:

```json
{
  "specversion": "1.0",
  "id": "01J8X2Y3Z4A5B6C7D8E9F0G1H2",
  "source": "/ministries/{ministry_id}/sites/{site_id}/services/{service_name}",
  "type": "inwp.{domain}.v1.{entity}.{action}",
  "datacontenttype": "application/json",
  "subject": "{entity_type}:{entity_id}",
  "time": "2026-06-01T10:00:00.000Z",
  "dataschema": "inwp:{domain}:{entity}:{action}:v1",
  "ministry_id": "b1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "site_id": "c2c3d4e5-f6a7-8901-bcde-f12345678901",
  "device_id": "d3d4e5f6-a7b8-9012-cdef-123456789012",
  "user_id": "e4e5f6a7-b8c9-0123-defa-234567890123",
  "offline_generated": true,
  "local_timestamp": "2026-06-01T13:00:00.000+03:00",
  "sync_id": "f5f6a7b8-c9d0-1234-efab-345678901234",
  "trace_id": "a6a7b8c9-d0e1-2345-fabc-456789012345"
}
```

### 5.2 Complete Event Taxonomy

```
inwp
├── attendance.v1
│   ├── clock-in.created
│   ├── clock-out.created
│   ├── break.started
│   ├── break.ended
│   ├── attendance.corrected
│   ├── attendance.disputed
│   ├── attendance.exception.created
│   ├── attendance.exception.justified
│   ├── attendance.exception.resolved
│   ├── attendance.exception.escalated
│   ├── shift.created
│   ├── shift.modified
│   ├── shift.deactivated
│   ├── policy.created
│   ├── policy.activated
│   └── policy.superseded
│
├── leave.v1
│   ├── request.created
│   ├── request.submitted
│   ├── request.approved
│   ├── request.rejected
│   ├── request.cancelled
│   ├── request.expired
│   ├── request.completed
│   ├── balance.adjusted
│   ├── balance.accrued
│   ├── balance.deducted
│   ├── balance.expired
│   ├── balance.warning
│   ├── accrual.processed
│   ├── accrual.policy.created
│   └── accrual.policy.updated
│
├── identity.v1
│   ├── user.registered
│   ├── user.verified
│   ├── user.deactivated
│   ├── user.reactivated
│   ├── credential.added
│   ├── credential.changed
│   ├── credential.revoked
│   ├── role.assigned
│   ├── role.revoked
│   ├── device.enrolled
│   ├── device.activated
│   ├── device.suspended
│   ├── device.revoked
│   ├── device.trust.changed
│   ├── device.heartbeat
│   ├── realm.created
│   ├── realm.deactivated
│   ├── auth.policy.updated
│   └── session.created
│
├── workforce.v1
│   ├── employee.state.changed
│   ├── timesheet.generated
│   ├── work.history.updated
│   ├── eligibility.changed
│   └── snapshot.computed
│
├── policy.v1
│   ├── policy.created
│   ├── policy.updated
│   ├── policy.activated
│   ├── policy.deactivated
│   ├── policy.evaluated
│   └── policy.violation.detected
│
├── device.v1
│   ├── device.connected
│   ├── device.disconnected
│   ├── device.firmware.available
│   ├── device.firmware.updated
│   ├── device.diagnostic.reported
│   ├── device.config.updated
│   └── device.alert.generated
│
├── sync.v1
│   ├── batch.committed
│   ├── conflict.detected
│   ├── conflict.resolved
│   ├── heartbeat.sent
│   ├── schema.negotiated
│   └── node.state.changed
│
├── audit.v1
│   ├── entry.appended
│   ├── seal.generated
│   ├── seal.verified
│   └── integrity.failure
│
├── notification.v1
│   ├── notification.created
│   ├── notification.sent
│   ├── notification.delivered
│   ├── notification.read
│   ├── notification.failed
│   └── notification.expired
│
├── analytics.v1
│   ├── report.generated
│   ├── report.scheduled
│   ├── metric.computed
│   └── anomaly.detected
│
└── system.v1
    ├── node.online
    ├── node.offline
    ├── service.healthy
    ├── service.degraded
    ├── service.unhealthy
    └── configuration.changed
```

### 5.3 Event Lifecycle

```
[Producer Service]
    |
    |---(1) Create domain event in aggregate
    |---(2) Validate against JSON Schema
    |---(3) Sign with Ed25519 (producer private key)
    |---(4) Append to local event_outbox (PostgreSQL)
    |---(5) Publish to event bus (NATS national / pg_notify edge)
    |
    v
[Event Bus]
    |
    |---(6) Route by type + tenant (content-based routing)
    |---(7) Persistent store in NATS JetStream (national)
    |---(8) Deliver to subscribers (at-least-once)
    |
    v
[Consumer Service]
    |
    |---(9) Verify Ed25519 signature
    |---(10) Validate schema version
    |---(11) Idempotent processing (dedup by event_id)
    |---(12) Acknowledge or dead-letter
    |
    v
[Audit Ledger]
    |
    |---(13) Append all events to immutable hash chain
    |---(14) Generate periodic seals for integrity verification
```

### 5.4 Event Versioning

| Strategy | Detail |
|---|---|
| **Breaking changes** | New major version (`attendance.v2`) — co-exist until old consumers migrate |
| **Additive changes** | Backward-compatible — `dataschema` URL reflects minor version |
| **Deprecation** | Events carry `deprecated: true`; removed after 2 migration cycles |
| **Schema registry** | `platform-core` maintains canonical schemas; validated at produce/consume time |

---

## 6. Security Architecture

### 6.1 Zero-Trust Model

```
+-----------------------+       +-----------------------+
| Request               | ----> | Never trust, always    |
| (service, device,     |       | verify                |
|  user, admin)         |       |                       |
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

### 6.2 Security Layers

| Layer | Mechanism | Scope |
|---|---|---|
| **Transport** | mTLS 1.3 — all service-to-service; certificate pinning on mobile | Wire |
| **Authentication** | OAuth2 + OIDC (Keycloak for external); JWT with signed assertions (internal) | Identity |
| **Authorization** | RBAC (role-based) + ABAC (attribute-based: ministry/site/region/device context) | Access |
| **Data at rest** | AES-256-GCM per-tenant envelope encryption; HSM-backed key hierarchy | Storage |
| **Data in transit** | TLS 1.3 + payload-level Ed25519 signing for audit events | Data |
| **Device trust** | TPM/TrustZone attestation, device certificate enrollment, trust score decay | Device |
| **Audit** | Immutable hash chain, cryptographic seals, periodic external verification | Accountability |
| **Secrets** | HashiCorp Vault (national), Docker secrets (edge), no secrets in config | Operations |
| **API security** | Rate limiting (Redis), idempotency keys, input validation, CORS per ministry | API |

### 6.3 Certificate Authority Hierarchy

```
            +-----------------------------+
            | INWP Root CA (offline)      |
            | Self-signed, HSM-protected  |
            | Air-gapped, powered on only |
            | for CRL/CA updates          |
            +-----------------------------+
                         |
          +--------------+--------------+
          |              |              |
+---------v----+  +------v------+  +---v----------+
| National CA  |  | Ministry CA |  | Device CA    |
| (online,     |  | (per        |  | (per tenant, |
| HSM-backed)  |  |  ministry)  |  |  short TTL)  |
+--------------+  +-------------+  +--------------+
     |                  |                |
     v                  v                v
+---------+      +-------------+  +-------------+
| Service  |      | Ministry    |  | Biometric   |
| Certs    |      | Admin Certs |  | Terminal    |
| (gRPC)   |      | (Web Portal)|  | Certificates|
+---------+      +-------------+  +-------------+
```

### 6.4 Authentication Flows

#### User Authentication (External)

```
User -> Web Portal / Mobile App
    |--(1) POST /auth/authorize { username, password, ministry_id, device_attestation }
    |
Keycloak / Identity Service
    |--(2) Verify credentials against ministry realm
    |--(3) Verify device attestation (TPM nonce challenge)
    |--(4) Evaluate MFA policy (password + TOTP/biometric as configured)
    |--(5) Issue tokens:
    |       access_token (15 min JWT) + refresh_token (24h)
    |       JWT claims: sub, ministry, site, roles, device_id, auth_method
    |
User -> API Gateway
    |--(6) Present JWT; gateway validates signature, expiry, device trust
    |--(7) Forward to target service with JWT claims as gRPC metadata
```

#### Service-to-Service Authentication

```
Service A
    |--(1) mTLS 1.3 handshake with service certificate (issued by National CA)
    |--(2) Present service JWT (5 min TTL):
    |       { sub: "service:attendance-service", aud: "service:leave-service",
    |         ministries: ["mohe", "moh"], iat, exp }
    |
Service B
    |--(3) Verify mTLS client cert (CRL check, validity, issuer chain)
    |--(4) Verify JWT signature (service account key from Vault)
    |--(5) Evaluate ABAC policy (can attendance-service read leave records for mohe?)
    |--(6) Authorize or reject with audit event
```

### 6.5 RBAC Structure

```
/national
  ├── national_admin         Full system access
  ├── national_auditor       Read-only audit logs, all ministries
  └── system_operator        Infrastructure management

/ministries/{ministry_id}
  ├── ministry_admin         Full ministry access
  ├── ministry_hr            HR operations, employee management
  ├── ministry_auditor       Read-only ministry data
  └── ministry_viewer        Dashboard read-only

/sites/{site_id}
  ├── site_admin             Site operations management
  ├── hr_operator            Attendance, leave operations
  ├── attendance_operator    Clock event management, exceptions
  ├── leave_approver         Leave approval authority
  └── viewer                 Site data read-only
```

### 6.6 Cryptographic Inventory

| Algorithm | Usage | Key Length |
|---|---|---|
| Ed25519 | Event signing, sync batch signatures | 256-bit |
| X25519 | Key exchange (E2EE for sensitive data) | 256-bit |
| AES-256-GCM | Data at rest (per-tenant envelope) | 256-bit |
| SHA-256 | Merkle tree, hash chains, payload hashing | 256-bit |
| TLS 1.3 | Transport security | X.509 (ECC P-384) |
| Argon2id | Password hashing | (memory=cost, time=cost) |
| HMAC-SHA256 | Idempotency keys, API tokens | 256-bit |

---

## 7. Synchronization Architecture

### 7.1 Design Principles

- **No single source of truth** — truth is eventually converged across all nodes
- **Anti-entropy over consensus** — merkle tree reconciliation; no RAFT/Paxos across WAN
- **Peer-to-peer on LAN** — direct sync between edge nodes without hub mediation
- **Hub-and-spoke on WAN** — regional relays optimize bandwidth
- **CRDT-inspired** — LWW, ministry-author-wins, service-merge, additive merge per entity
- **Delta-only** — only changed data transmitted; full snapshot on first sync

### 7.2 Sync Topology

```
                +------------------+
                |  National Sync   |
                |  Hub (NAT)       |
                |  - Global routing |
                |  - Cross-ministry |
                |  - Merkle root    |
                |    publication    |
                +--------+---------+
                         |
           +-------------+-------------+
           |             |             |
     +-----v---+   +-----v---+   +-----v---+
     | Region1 |   | Region2 |   | Region3 |
     | Relay   |   | Relay   |   | Relay   |
     | - zstd  |   | - zstd  |   | - zstd  |
     | - Store-|   | - Store-|   | - Store-|
     |  &-Frwd |   |  &-Frwd |   |  &-Frwd |
     +-----+---+   +-----+---+   +-----+---+
           |             |             |
     +-----v---+   +-----v---+   +-----v---+
     | Edge 1  |   | Edge 2  |   | Edge 3  |
     | (LAN    |<->| (LAN    |<->| (LAN    |
     | Mesh)   |   | Mesh)   |   | Mesh)   |
     +---------+   +---------+   +---------+
```

### 7.3 Sync Protocol — 5 Phases

```
Phase 1: DISCOVERY
  Edge Node A broadcasts presence via mDNS (LAN) or connects to assigned Regional Relay (WAN).
  Exchange: capabilities, supported schema versions, last sync checkpoint, node certificate.

Phase 2: MERKLE EXCHANGE
  Each node maintains a merkle tree per partition (ministry + entity + time bucket).
  A sends merkle root hashes for each partition.
  B compares with its own, identifies differing branches recursively.
  Result: specific divergent record IDs identified.

Phase 3: DELTA TRANSFER
  For each differing record, owning node sends full record + event chain.
  Payload compressed via zstd (level 19 for cold data, level 3 for hot data).
  Large transfers chunked at 1MB with resume capability.

Phase 4: RECONCILIATION
  Both nodes apply mutual changes.
  Conflicts resolved per per-entity matrix:
    - LWW (default) — highest local_timestamp wins
    - Ministry author wins — hub overrides edge
    - Service-side merge — domain-correct merge (e.g., accrual math)
    - Manual — escalated to admin dashboard

Phase 5: COMMITMENT
  Both nodes sign the sync batch (sync_id, merkle root after apply, timestamp).
  Receipt appended to local audit log.
  Both advance checkpoint.
  Batch committed event published.
```

### 7.4 Per-Entity Conflict Resolution

| Entity | Strategy | Rationale |
|---|---|---|
| ClockEvent | LWW + Dedup (append-only) | Immutable; dedup by event_id |
| Shift | Ministry Author Wins | HR authoritative |
| AttendancePolicy | Ministry Author Wins | Only one active per site |
| AttendanceException | LWW (justification) | Latest justification valid |
| LeaveRequest | Service Merge + Manual | State machine + human judgment |
| LeaveBalance | Service Merge (PN-Counter) | Financial accuracy required |
| AccrualPolicy | LWW | Latest policy wins |
| User profile | LWW by field | Per-field independent resolution |
| Role assignment | Additive Merge (G-Set) | Union of assignments |
| Device trust | LWW | Computed value |
| LedgerEntry | None (append-only) | Immutable by design |

### 7.5 Offline Operation

| Capability | Detail |
|---|---|
| **Local writes** | All CRUD succeed locally; event_outbox stores pending events |
| **Local reads** | Complete local PostgreSQL; zero dependency on remote data |
| **Queue depth** | Unlimited (disk-based); oldest purged only after confirmed sync |
| **Reconnection** | Exponential backoff: 1s → 5s → 30s → 5min → 30min; immediate on network change |
| **Conflict queue** | Conflicting writes held for admin review; no data loss |
| **Heartbeat** | Edge nodes emit heartbeat every 60s via NATS/pg_notify |
| **Delayed sync** | Admin-configurable sync windows (e.g., 02:00-05:00 for bulk) |
| **Regional sync** | Edge → Regional Hub on reconnect; Regional → National on schedule |
| **Event replay** | Full event log replay for new nodes or recovery |
| **Event versioning** | Schema negotiated during discovery; backward-compatible |

### 7.6 Bandwidth Management

| Technique | Implementation |
|---|---|
| **Delta compression** | zstd at payload level; only send diffs after initial sync |
| **Priority queuing** | Attendance = HIGH (real-time), Leave = MEDIUM, Analytics = LOW |
| **Bandwidth budgeting** | Admin-configurable daily bandwidth cap per node (Redis counters) |
| **Sync scheduling** | Configurable windows per entity type |
| **Chunking** | 1MB chunks with checkpoint/resume on interrupt |
| **Image compression** | Biometric images resized to 640x480, JPEG quality 70 |
| **Dedup** | Event-level dedup before transmission |

---

## 8. Deployment Architecture

### 8.1 Tier Architecture

```
+---------------------+     +---------------------+     +---------------------+
| NATIONAL DC         |     | REGIONAL HUB (x18)  |     | EDGE NODE (1000+)   |
| (Kubernetes)        |     | (Docker Compose)    |     | (Docker Compose)    |
|                     |     |                     |     |                     |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| | NATS Cluster  |   |     | | NATS Leaf     |   |     | | Sync Engine   |   |
| | (3 nodes)     |   |     | | (1 node)      |   |     | | (1 instance)  |   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| | Patroni PG    |   |     | | PG Replica    |   |     | | PostgreSQL    |   |
| | (3 nodes)     |   |     | | (streaming)   |   |     | | (local)       |   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| | Redis Cluster |   |     | | Redis Cache   |   |     | | Redis Cache   |   |
| | (3 nodes)     |   |     | | (standalone)  |   |     | | (standalone)  |   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| | MinIO         |   |     | | MinIO Cache   |   |     | | Attendance    |   |
| | (distributed) |   |     | | (gateway)     |   |     | | Service       |   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| | Keycloak      |   |     | | Identity      |   |     | | Leave Service |   |
| | (cluster)     |   |     | | Cache         |   |     | | (offline)     |   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
| | All Services  |   |     | | Attendance Agg|   |     | | Device Gateway|   |
| | (3 replicas)  |   |     | | (2 replicas)  |   |     | | (edge proxy)  |   |
| +---------------+   |     | +---------------+   |     | +---------------+   |
+---------------------+     +---------------------+     +---------------------+
```

### 8.2 Infrastructure Requirements

| Tier | CPU | RAM | Storage | Network |
|---|---|---|---|---|
| National DC (K8s node) | 32+ cores | 128+ GB | SSD/NVMe 4TB+ | 10 Gbps redundant |
| Regional Hub | 8 cores | 32 GB | SSD 1TB | 1 Gbps |
| Edge Node (large site) | 4 cores | 16 GB | SSD 500GB | 100 Mbps LAN |
| Edge Node (small site) | 2 cores | 8 GB | SSD 250GB | 10 Mbps LAN / 4G |
| Mobile device | ARM64 | 4+ GB | 64GB+ flash | WiFi / 4G |

### 8.3 Docker Compose (Edge Node)

```yaml
version: "3.8"
services:
  postgres:
    image: postgis/postgres:16-3.4
    volumes:
      - pgdata:/var/lib/postgresql/data
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    deploy:
      resources:
        limits:
          memory: 512M
    networks:
      - inwp-lan

  redis:
    image: redis:7-alpine
    volumes:
      - redis-data:/data
    command: redis-server --requirepass ${REDIS_PASSWORD} --lfu-decay-time 1
    deploy:
      resources:
        limits:
          memory: 128M
    networks:
      - inwp-lan

  minio:
    image: minio/minio:latest
    volumes:
      - minio-data:/data
    command: server /data --console-address ":9001"
    environment:
      MINIO_ROOT_USER: ${MINIO_ACCESS_KEY}
      MINIO_ROOT_PASSWORD: ${MINIO_SECRET_KEY}
    deploy:
      resources:
        limits:
          memory: 256M
    networks:
      - inwp-lan

  sync-engine:
    image: registry.inwp.iq/sync-engine:latest
    depends_on: [postgres, nats]
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
      NATS_URL: "nats://nats:4222"

  attendance-service:
    image: registry.inwp.iq/attendance-service:latest
    depends_on: [postgres, redis, sync-engine]
    ports:
      - "8080:8080"
    networks:
      - inwp-lan
      - biometric-vlan
    environment:
      DB_DSN: "postgres://inwp:${DB_PASSWORD}@postgres:5432/inwp_attendance"
      REDIS_URL: "redis://:${REDIS_PASSWORD}@redis:6379"
      NATS_URL: "nats://nats:4222"
      BIOMETRIC_DEVICES: "192.168.50.0/24"

  device-gateway:
    image: registry.inwp.iq/device-gateway:latest
    depends_on: [postgres, redis, nats]
    networks:
      - inwp-lan
      - biometric-vlan
    environment:
      DB_DSN: "postgres://inwp:${DB_PASSWORD}@postgres:5432/inwp_device"
      REDIS_URL: "redis://:${REDIS_PASSWORD}@redis:6379"
      NATS_URL: "nats://nats:4222"

  nats:
    image: nats:2-alpine
    command: -c /etc/nats/nats.conf
    volumes:
      - ./nats.conf:/etc/nats/nats.conf:ro
    networks:
      - inwp-lan
    deploy:
      resources:
        limits:
          memory: 64M

volumes:
  pgdata:
  redis-data:
  minio-data:

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

### 8.4 NATS Configuration (Edge Leaf Node)

```yaml
# nats.conf — edge leaf node
port: 4222

leafnode:
  listen: "0.0.0.0:7422"
  remotes:
    - url: "tls://nats-relay.baghdad.inwp.iq:7422"
      account: "inwp-edge"

jetstream:
  store_dir: /data/jetstream
  max_memory_store: 64MB
  max_file_store: 1GB

authorization:
  token: ${NATS_TOKEN}

websocket:
  port: 8443
  tls: true
```

### 8.5 Disaster Recovery

| Tier | RPO | RTO | Mechanism |
|---|---|---|---|
| National DC | 0-5s | < 1min | Patroni cluster with synchronous replication |
| Regional Hub | < 5min | < 15min | Streaming PG replication + NATS JetStream mirror |
| Edge Node | < 24h | < 1h | WAL archive to MinIO; pgBackRest restore |
| Cross-region DR | < 1h | < 4h | Logical replication to DR site in different governorate |
| Backup (full) | Daily | < 2h | pg_dump to MinIO, encrypted with KMS |
| Backup (WAL) | Continuous | < 15min | WAL archiving to MinIO |

---

## 9. Phase-1 Implementation Roadmap

### 9.1 Phases Overview

```
Phase 1 (Months 1-3): Core Foundation
  Identity Service + Attendance Service + Platform Core + Sync Engine MVP

Phase 2 (Months 4-6): Core Services
  Leave Service + Audit Ledger + Notification Service + Policy Engine

Phase 3 (Months 7-9): Advanced Services
  Workforce State Engine + Device Gateway + Analytics Service + Regional Hub

Phase 4 (Months 10-12): Production Hardening
  National DC (K8s) + Chaos Engineering + Performance Tuning + Security Audit
```

### 9.2 Phase 1 Detailed Plan

#### Sprint 1-2: Platform Core & Infrastructure

| Task | Deliverable | Owner |
|---|---|---|
| Set up monorepo structure | Repository with Go modules, proto, CI | Platform |
| Implement platform-core types | Domain types, base event, crypto, validation | Platform |
| Define protobuf schemas | .proto files for all services | Platform |
| JSON Schema registry | v1 event schemas (attendance, identity) | Platform |
| Postgres schema strategy | Migration framework, RLS templates, partitioning | Platform |
| Docker Compose for national | Postgres, NATS, Redis, MinIO, Keycloak | Infra |
| Docker Compose for edge | Postgres, NATS leaf, sync engine | Infra |
| CI/CD pipeline | Build, test, lint, container image push | Platform |

#### Sprint 3-4: Identity Service

| Task | Deliverable | Owner |
|---|---|---|
| User aggregate + repository | Register, verify, deactivate, reactivate | Identity |
| Device aggregate + repository | Enroll, trust scoring, certificate mgmt | Identity |
| Realm aggregate + repository | Ministry realm, auth policies, role catalog | Identity |
| Authentication service | Password, TOTP, biometric, certificate auth | Identity |
| Keycloak bridge | User sync, realm import, OIDC federation | Identity |
| Session management | JWT issue/refresh/revoke, Redis cache | Identity |
| gRPC API | Internal service interface | Identity |
| REST OAuth2/OIDC API | External auth endpoints | Identity |
| SCIM provisioning | User/group provisioning API | Identity |
| RBAC implementation | Role definitions, assignment, ABAC evaluation | Identity |

#### Sprint 5-6: Attendance Service

| Task | Deliverable | Owner |
|---|---|---|
| ClockEvent aggregate + repository | Immutable clock-in/out, partitioning, dedup | Attendance |
| Shift aggregate + repository | Create, modify, deactivate, site binding | Attendance |
| AttendancePolicy aggregate + repo | Versioned policies, supersede logic | Attendance |
| AttendanceException + repo | Auto-detect, justify, resolve, escalate | Attendance |
| ClockIn/ClockOut handlers | Command → domain → event → persist → publish | Attendance |
| Duplicate detection | 30s window dedup | Attendance |
| Biometric verification delegation | BiometricResult API | Attendance |
| Event publishing (NATS + pg_notify) | CloudEvents 1.0 via NATS, fallback to pg_notify | Attendance |
| Sync outbox integration | Pending events for sync engine | Attendance |
| REST API | Full set of 11 endpoints | Attendance |
| gRPC API | Internal service API | Attendance |
| PostgreSQL migrations | Partitioned tables, RLS, indexes, triggers | Attendance |

#### Sprint 7-8: Sync Engine MVP

| Task | Deliverable | Owner |
|---|---|---|
| Merkle tree implementation | Partitioned merkle trees, leaf/node hashing | Sync |
| Merkle exchange protocol | Diff identification, recursive branching | Sync |
| Delta transfer | zstd compression, chunking, resume | Sync |
| Conflict detection + LWW resolution | Version vectors, LWW, dedup | Sync |
| Sync batch commitment | Dual signature, audit receipt | Sync |
| LAN discovery (mDNS) | Peer discovery for mesh sync | Sync |
| HTTPS transport | Edge → Regional Hub sync upload/download | Sync |
| WebSocket transport | Hub → Edge push notifications | Sync |
| Heartbeat mechanism | 60s heartbeat, stale node detection | Sync |
| Bandwidth tracking | Usage counters, throttling | Sync |
| Sync outbox consumer | Fetch pending events from services | Sync |

### 9.3 Phase 1 Architecture Decision Records

| ADR | Decision | Rationale |
|---|---|---|
| ADR-P1-001 | Go for all Phase 1 services | Single binary, fast compile, ARM64 support |
| ADR-P1-002 | Rust deferred to Phase 2 for sync engine | Go MVP sufficient for initial merkle exchange |
| ADR-P1-003 | Keycloak as external IdP with identity-service bridge | Battle-tested OIDC; custom code for offline flows |
| ADR-P1-004 | Redis for session cache + rate limits from day 1 | Critical for offline session handling |
| ADR-P1-005 | MinIO deferred to Phase 2 | File storage not needed for MVP |
| ADR-P1-006 | NATS for all event bus (national + regional) | Consistent event fabric; pg_notify only at edge |
| ADR-P1-007 | Docker Compose for all tiers in Phase 1 | K8s complexity deferred to Phase 4 |
| ADR-P1-008 | Phase 1 sync uses Go (not Rust) | Faster iteration; Rust rewrite in Phase 2 if perf requires |

---

## 10. Anti-Fragility Recommendations

### 10.1 Anti-Fragility Principles

Anti-fragility means the system **strengthens under stress** — it doesn't just survive failures, it learns and improves from them.

```
┌─────────────────────────────────────────────────────────────────┐
│  FRAGILE         ROBUST          RESILIENT        ANTIFRAGILE   │
│  Breaks under    Withstands      Recovers from    Strengthens   │
│  stress          stress          stress           from stress   │
│                                                      ↑         │
│                                              INWP TARGET       │
└─────────────────────────────────────────────────────────────────┘
```

### 10.2 Chaos Engineering

| Scenario | Frequency | Mechanism | Success Criterion |
|---|---|---|---|
| Node isolation (network partition) | Weekly | Block port 8443 on edge node | Full local operation, queue grows, sync resumes on reconnect |
| Database failover | Bi-weekly | Kill primary Patroni node | < 30s failover, no data loss |
| NATS cluster partition | Bi-weekly | Split NATS cluster into 2+1 | Leaf nodes reconnect to available routes |
| Resource starvation | Weekly | Limit CPU/memory on edge to 50% | Graceful degradation, no crash |
| Disk full simulation | Monthly | Fill disk to 95% | Monitoring alert, admin notification, read-only fallback |
| Certificate expiry | Monthly | Expire device/service cert | Device enters DEGRADED, admin alerted, auto-renew attempted |
| Clock skew > 300s | Monthly | Set device clock 10min ahead | HLC ordering prevents incorrect LWW resolution |
| Regional hub offline (extended) | Quarterly | Take regional hub offline for 72h | Edge nodes operate independently, batch sync on return |
| National DC offline | Bi-annual | Full national DC outage for 24h | Regional hubs operate autonomously, DR site activated |

### 10.3 Graceful Degradation Paths

| Component Failure | Degraded Behavior | Recovery |
|---|---|---|
| **PostgreSQL (local)** | Service enters read-only mode; events buffered in memory with disk spill | Auto-restart via Docker healthcheck; WAL replay |
| **NATS (local)** | Switch to pg_notify for local events; queue grows in outbox | Reconnect with exponential backoff; replay outbox |
| **NATS (regional)** | Edge continues on pg_notify only; no cross-site events | Reconnect to alternate relay or direct to national |
| **Redis (local)** | Session validation falls back to PostgreSQL (slower) | Redis auto-restart; cache repopulated on demand |
| **Keycloak (national)** | Identity service uses local user cache + offline sessions | Keycloak cluster failover; cache TTL extended |
| **Sync engine** | Events queue in outbox; merkle tree persists locally | Resume sync from last checkpoint |
| **MinIO** | Audit archive writes fail; new data stays in PG hot storage | MinIO restore from replica; catch-up archive |
| **Device gateway** | Devices buffer events locally (most devices have 10k+ buffer) | Gateway restart; drain device buffer |
| **Power failure (edge)** | PG crash recovery on restart; WAL replay; sync re-initiated | Automated via Docker restart policy: always |

### 10.4 Automatic Healing Mechanisms

| Mechanism | Implementation | Trigger |
|---|---|---|
| **Auto-restart** | Docker `restart: always` + healthcheck | Process crash, unhealthy probe |
| **Circuit breaker** | Per-service circuit breaker (hystrix-go) | 50% error rate over 30s window |
| **Bulkhead isolation** | Separate connection pools per tenant/operation | Pool exhaustion threshold |
| **Retry with backoff** | Exponential backoff with jitter | Transient failures (network, DB timeout) |
| **Dead letter queue** | Failed events routed to DLQ for analysis | Max retries exceeded |
| **Leader election** | PostgreSQL advisory locks for singleton tasks | Cron job collision |
| **Auto-scaling** | Docker Compose replicas (edge), HPA (national) | CPU > 70% or queue depth > 1000 |
| **Connection pooling** | pgx pool with min/max config per service | Connection spike |
| **Cache refresh** | Background goroutine refreshes Redis cache | TTL expiry or cache miss threshold |
| **Schema migration** | Automated migration on startup; backward-compatible only | New binary version |

### 10.5 Monitoring & Observability

```yaml
# Prometheus metrics exposed by every service
metrics:
  - inwp_requests_total{service, method, status, ministry}
  - inwp_request_duration_ms{service, method, quantile}
  - inwp_events_produced_total{service, event_type}
  - inwp_events_consumed_total{service, event_type}
  - inwp_sync_batch_duration_ms{source, target, partition}
  - inwp_sync_bytes_transferred{source, target}
  - inwp_sync_conflicts_total{partition, resolution}
  - inwp_db_connections{service, state}
  - inwp_queue_depth{service, queue_name}
  - inwp_device_trust_level{device_id, level}

# Alerting rules
alerts:
  - name: NodeOffline
    condition: inwp_node_status == 0 for > 120s
    severity: critical
    action: Notify site admin + regional ops

  - name: SyncStale
    condition: inwp_sync_last_success > 24h
    severity: warning
    action: Notify regional ops

  - name: QueueBacklog
    condition: inwp_queue_depth > 10000
    severity: warning
    action: Notify service owner

  - name: HighConflictRate
    condition: rate(inwp_sync_conflicts_total[1h]) > 50
    severity: warning
    action: Notify sync team + generate report

  - name: CertificateExpiring
    condition: inwp_cert_days_remaining < 14
    severity: warning
    action: Auto-renew or notify admin

  - name: AuditIntegrityFailure
    condition: inwp_audit_integrity_failures > 0
    severity: critical
    action: Immediate security incident response
```

### 10.6 Learning Loops

| Loop | Input | Output | Frequency |
|---|---|---|---|
| Incident post-mortem | Production incidents | Runbook updates, code fixes, monitoring improvements | Per incident |
| Chaos exercise review | Chaos experiment results | Architecture hardening, new chaos scenarios | Monthly |
| Sync conflict analysis | Conflict log | Resolution strategy tuning, entity matrix updates | Weekly |
| Performance benchmark | Load test results | Capacity planning, config tuning, schema optimization | Sprint review |
| Security audit | Penetration test, vulnerability scan | Security fixes, policy updates, threat model refresh | Quarterly |
| Operational health review | SLO/SLI dashboard | Budget adjustment, reliability investments | Monthly |
| Edge node feedback | Field reports from site admins | UX improvements, offline flow enhancements | Bi-weekly |

---

## Appendix: Technology Stack Summary

| Category | Technology | Phase |
|---|---|---|
| **Core language** | Go 1.22+ | P1 |
| **Sync engine** | Rust (Go MVP in P1) | P2 (P1: Go) |
| **Database** | PostgreSQL 16 | P1 |
| **Cache** | Redis 7 | P1 |
| **Object storage** | MinIO | P2 |
| **Event bus** | NATS 2 + pg_notify | P1 |
| **Identity** | Keycloak + custom IdP bridge | P1 |
| **Serialization** | Protocol Buffers + JSON | P1 |
| **Sync protocol** | Custom (merkle tree + delta) | P1 |
| **Container runtime** | Docker (edge), Docker Compose (regional), Kubernetes (national) | P1: Docker; P4: K8s |
| **Secrets** | HashiCorp Vault (national), Docker secrets (edge) | P1 |
| **Monitoring** | Prometheus + Grafana | P1 |
| **Logging** | Structured JSON → stdout → Fluentd → S3 | P1 |
| **APIs** | gRPC (internal) + REST (external) + AsyncAPI (events) | P1 |
| **Policy evaluation** | OpenPolicyAgent (Rego) embedded | P2 |
| **CI/CD** | GitHub Actions | P1 |

---

## Appendix: Port Mapping

| Service | Internal Port | External Port | Protocol |
|---|---|---|---|
| identity-service (gRPC) | 50051 | 443 (via proxy) | gRPC |
| identity-service (REST) | 8081 | 443 | HTTPS |
| attendance-service (gRPC) | 50055 | 8080 | gRPC |
| attendance-service (REST) | 8080 | 8080 | HTTPS |
| leave-service (gRPC) | 50056 | 8082 | gRPC |
| leave-service (REST) | 8082 | 8082 | HTTPS |
| workforce-state-engine | 50060 | 8084 | gRPC |
| sync-engine (HTTPS/WS) | 50052 | 8443 | HTTPS/WS |
| sync-engine (LAN mesh) | 50053 | - | gRPC |
| audit-ledger | 50054 | 443 | gRPC |
| notification-service | 50057 | 8083 | gRPC |
| policy-engine | 50058 | 8085 | gRPC |
| device-gateway | 50059 | 8086 | gRPC/HTTP |
| analytics-service | 50061 | 8087 | REST |
| PostgreSQL | 5432 | - | TCP |
| NATS | 4222 | 4222 | NATS |
| NATS WebSocket | 8443 | 8443 | WSS |
| NATS Leafnode | 7422 | 7422 | TLS |
| Redis | 6379 | - | TCP |
| MinIO (API) | 9000 | 9000 | HTTPS |
| MinIO (Console) | 9001 | 9001 | HTTPS |
| Keycloak | 8080 | 443 | HTTPS |
| Prometheus metrics | 9090 | - | HTTP |
| Health/readiness | 8080 | - | HTTP |

---

## Appendix: Key ADRs

| ADR | Decision | Rationale |
|---|---|---|
| ADR-001 | PostgreSQL for all persistence | No polyglot persistence; mandated by constraints |
| ADR-002 | Go as primary language | Single binary, fast compile, ARM64 cross-compile |
| ADR-003 | Rust for sync engine (P2) | Memory safety for correctness-critical sync code |
| ADR-004 | Custom sync protocol | Requirements-specific (offline-first, LAN mesh, air-gap) |
| ADR-005 | NATS for national event bus | Lightweight, at-least-once, cluster mode, JetStream |
| ADR-006 | pg_notify for edge event bus | Zero additional dependencies, works offline |
| ADR-007 | Event sourcing + CQRS for audit | Immutable audit trail requirement |
| ADR-008 | mTLS for all service-to-service | Zero-trust mandate |
| ADR-009 | OAuth2 + OIDC (Keycloak) for identity | Industry standard, offline-capable with local sessions |
| ADR-010 | Redis for cache/locks/sessions | Sub-millisecond reads, atomic operations, TTL-based expiry |
| ADR-011 | MinIO for object storage | S3-compatible, lightweight, distributed, encryption |
| ADR-012 | Per-ministry schema isolation | Multi-tenant without shared schema complexity |
| ADR-013 | CRDT/LWW conflict resolution | Eventual consistency without consensus overhead |
| ADR-014 | 3-tier deployment (edge/region/national) | Government-grade reliability, regional autonomy |
| ADR-015 | Anti-fragility by design | System strengthens under stress via chaos engineering |
