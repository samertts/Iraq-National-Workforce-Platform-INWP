# INWP Complete Architecture Reference

> Iraq National Workforce Platform — Sovereign, distributed, offline-first workforce operating system for Iraq.
> Government-grade reliability, zero-trust security, autonomous edge operation, national scale.

---

## Table of Contents

1. [Repository Structure](#1-repository-structure)
2. [Architecture Principles](#2-architecture-principles)
3. [System Topology](#3-system-topology)
4. [Core Services](#4-core-services)
5. [Service Boundaries](#5-service-boundaries)
6. [Event Architecture](#6-event-architecture)
7. [Synchronization Architecture](#7-synchronization-architecture)
8. [Security Architecture](#8-security-architecture)
9. [Deployment Topology](#9-deployment-topology)
10. [Infrastructure Architecture](#10-infrastructure-architecture)
11. [Observability & Monitoring](#11-observability--monitoring)
12. [CI/CD Architecture](#12-cicd-architecture)
13. [Testing Strategy](#13-testing-strategy)
14. [Disaster Recovery](#14-disaster-recovery)
15. [Edge-Node Architecture](#15-edge-node-architecture)
16. [Regional Synchronization](#16-regional-synchronization)
17. [Policy Engine Architecture](#17-policy-engine-architecture)
18. [Device Integration Architecture](#18-device-integration-architecture)
19. [Audit Ledger Architecture](#19-audit-ledger-architecture)
20. [Architecture Decision Records](#20-architecture-decision-records)
21. [Contracts Structure](#21-contracts-structure)
22. [Frontend Architecture](#22-frontend-architecture)
23. [Gateway Architecture](#23-gateway-architecture)
24. [Identity Architecture](#24-identity-architecture)
25. [Multi-Tenancy Architecture](#25-multi-tenancy-architecture)

---

## 1. Repository Structure

### 1.1 Root Layout

```
inwp/
├── backend/                          # All backend microservices
├── frontend/                         # Frontend applications
├── infrastructure/                   # Shared infrastructure config
├── deployment/                       # Deployment manifests
├── docs/                             # Documentation
├── architecture/                     # Architecture documentation
├── services/                         # Cross-cutting service definitions
├── gateway/                          # API and device gateway configs
├── identity/                         # Identity realm configuration
├── sync/                             # Sync protocol specification
├── audit/                            # Audit ledger specification
├── events/                           # Event schema registry
├── monitoring/                       # Monitoring configuration
├── observability/                    # Observability stack
├── security/                         # Security artifacts
├── policies/                         # Policy definitions
├── analytics/                        # Analytics configuration
├── scripts/                          # Operational scripts
├── docker/                           # Dockerfiles
├── kubernetes/                       # Kubernetes manifests
├── testing/                          # Test frameworks
├── edge/                             # Edge-node configuration
├── regional/                         # Regional hub configuration
├── disaster-recovery/                # DR playbooks and config
├── .github/                          # GitHub workflows
├── AGENTS.md                         # Engineering rules
├── LICENSE                           # Apache 2.0
├── README.md                         # Project overview
└── Makefile                          # Top-level build orchestration
```

### 1.2 Backend Services

```
backend/
├── identity-service/                 # Identity & access management
├── authentication-service/           # Authentication (separated from IdP)
├── authorization-service/            # Authorization (RBAC/ABAC evaluation)
├── attendance-service/               # Attendance tracking (implemented)
├── leave-service/                    # Leave management
├── workforce-state-engine/           # CQRS state projection
├── sync-engine/                      # Merkle tree sync (Rust)
├── audit-ledger/                     # Immutable audit trail
├── notification-service/             # Multi-channel notifications
├── policy-engine/                    # Centralized policy evaluation
├── analytics-service/                # Reporting & BI
├── device-gateway/                   # Device protocol abstraction
```

### 1.3 Frontend Applications

```
frontend/
├── web-portal/                       # React/Next.js admin portal
│   ├── src/
│   │   ├── components/               # Reusable UI components
│   │   │   ├── common/               # Buttons, inputs, modals, tables
│   │   │   ├── layout/               # Sidebar, header, shell
│   │   │   ├── attendance/           # Clock-event views, shift manage
│   │   │   ├── leave/                # Leave request, approval forms
│   │   │   ├── identity/             # User management, roles
│   │   │   ├── workforce/            # State explorer, timesheets
│   │   │   ├── analytics/            # Charts, dashboards
│   │   │   ├── admin/                # System administration
│   │   │   ├── reports/              # Report builder, exports
│   │   │   ├── settings/             # Ministry/site settings
│   │   │   ├── sync/                 # Sync status, conflicts
│   │   │   └── audit/                # Audit trail viewer
│   │   ├── pages/                    # Route pages
│   │   ├── hooks/                    # Custom React hooks
│   │   ├── services/                 # API clients
│   │   ├── stores/                   # State management
│   │   ├── utils/                    # Utilities
│   │   ├── types/                    # TypeScript types
│   │   ├── styles/                   # Global styles
│   │   ├── assets/                   # Static assets
│   │   └── locales/                  # i18n (ar, ku, en)
│   ├── public/                       # Static files
│   ├── configs/                      # App configuration
│   ├── tests/                        # Unit + E2E tests
│   └── deploy/                       # Build manifests
│
├── mobile-app/                       # React Native employee app
│   ├── src/
│   │   ├── components/               # Reusable mobile components
│   │   ├── screens/
│   │   │   ├── attendance/           # Clock-in/out, history
│   │   │   ├── leave/                # Request, status
│   │   │   ├── profile/              # User profile
│   │   │   ├── approvals/            # Manager approvals
│   │   │   ├── dashboard/            # Employee dashboard
│   │   │   ├── settings/             # App settings
│   │   │   └── sync/                 # Offline sync status
│   │   ├── hooks/                    # Custom hooks
│   │   ├── services/                 # API + offline clients
│   │   ├── stores/                   # State management
│   │   ├── utils/                    # Mobile utilities
│   │   ├── types/                    # TypeScript types
│   │   ├── assets/                   # Images, fonts
│   │   └── locales/                  # i18n (ar, ku, en)
│   ├── android/                      # Android native config
│   ├── ios/                          # iOS native config
│   ├── tests/                        # Unit + E2E tests
│   └── deploy/                       # Build manifests
│
└── biometric-terminal/               # Embedded device interface
    ├── src/                          # Device-side application
    ├── firmware/                     # Firmware update packages
    ├── protocols/                    # Communication protocols
    └── tests/                        # Device test suite
```

### 1.4 Service-Internal Structure

Every microservice follows this internal hexagonal architecture:

```
{service-name}/
├── cmd/
│   └── {service-name}/
│       └── main.go                   # Entrypoint, dependency injection
├── internal/
│   ├── domain/                       # Aggregates, entities, value objects
│   │   ├── types.go                  # Core domain types
│   │   ├── events.go                 # Domain event definitions
│   │   └── errors.go                 # Domain error types
│   ├── application/                  # Use cases, application services
│   │   ├── {usecase1}.go            # Use case orchestration
│   │   └── interfaces.go            # Port interfaces
│   ├── infrastructure/              # Adapters, external dependencies
│   │   ├── postgres/                 # Repository implementations
│   │   ├── redis/                    # Cache implementations
│   │   ├── eventbus/                 # NATS/pg_notify adapters
│   │   └── {other-adapters}/        # Service-specific adapters
│   └── interfaces/                  # Inbound adapters
│       ├── rest/                     # HTTP handlers
│       └── grpc/                     # gRPC service implementations
├── migrations/                       # Database migrations
├── configs/                          # Service configuration files
├── security/                         # Service-specific security config
├── tests/                            # Service test suites
│   ├── unit/                         # Unit tests
│   ├── integration/                  # Integration tests
│   └── fixtures/                     # Test fixtures
├── deploy/                           # Service-specific deploy config
├── Dockerfile                        # Container build
├── go.mod / Cargo.toml               # Dependency manifest
└── Makefile                          # Build targets
```

### 1.5 Infrastructure Structure

```
infrastructure/
├── postgres/                     # PostgreSQL configuration
│   ├── patroni/                  # Patroni HA cluster config
│   ├── scripts/                  # Init, backup, maintenance
│   └── templates/                # Config templates per tier
├── nats/                         # NATS event bus
│   ├── config/                   # NATS server config
│   └── leaf-nodes/               # Leaf node config per region
├── redis/                        # Redis configuration
│   ├── config/                   # Redis server config
│   └── scripts/                  # Sentinel, cluster scripts
├── minio/                        # MinIO object storage
│   ├── config/                   # MinIO server config
│   ├── buckets/                  # Bucket policies
│   └── lifecycle/                # Lifecycle rules
├── vault/                        # HashiCorp Vault
│   ├── config/                   # Vault server config
│   ├── policies/                 # ACL policies
│   └── scripts/                  # Init, unseal scripts
├── keycloak/                     # Keycloak IdP
│   ├── realm-import/             # Realm JSON exports
│   ├── themes/                   # Custom themes
│   └── scripts/                  # Realm management
├── nginx/                        # Reverse proxy
├── traefik/                      # Traefik proxy
├── dns/                          # DNS configuration
├── loadbalancer/                 # Load balancer config
├── certificates/                 # Certificate management
└── monitoring/                   # Monitoring infra config
    ├── prometheus/               # Prometheus config
    ├── grafana/                  # Grafana config
    ├── loki/                     # Loki log aggregation
    ├── tempo/                    # Tempo tracing
    ├── alertmanager/             # Alertmanager config
    └── vector/                   # Vector data pipeline
```

### 1.6 Documentation Structure

```
docs/
├── architecture/                 # Architecture documentation
│   ├── decisions/                # ADR records
│   ├── patterns/                 # Architectural patterns
│   └── principles/               # Design principles
├── api/                          # API specifications
│   ├── openapi/                  # OpenAPI 3.0 specs
│   ├── grpc/                     # gRPC service definitions
│   ├── asyncapi/                 # AsyncAPI event specs
│   └── rest/                     # REST API conventions
├── events/                       # Event documentation
│   ├── catalog/                  # Event type catalog
│   ├── schemas/                  # Event schema docs
│   └── versioning/               # Version strategy
├── sync/                         # Sync protocol docs
│   ├── protocol/                 # Protocol specification
│   ├── specification/            # Wire format spec
│   └── configuration/            # Sync configuration guide
├── security/                     # Security documentation
│   ├── threat-models/            # STRIDE threat models
│   ├── pentest/                  # Penetration test reports
│   └── compliance/               # Compliance checklists
├── deployment/                   # Deployment guides
│   ├── national/                 # National DC deployment
│   ├── regional/                 # Regional hub deployment
│   ├── edge/                     # Edge node deployment
│   └── disconnected/             # Disconnected mode
├── operations/                   # Operations runbooks
│   ├── runbooks/                 # Incident runbooks
│   ├── incidents/                # Incident reports
│   └── maintenance/              # Scheduled maintenance
├── development/                  # Developer guides
│   ├── setup/                    # Development setup
│   ├── guidelines/               # Coding standards
│   └── standards/                # API, event, interface standards
├── governance/                   # Governance docs
│   ├── rbac/                     # Role definitions
│   ├── compliance/               # Compliance frameworks
│   └── audit/                    # Audit procedures
└── training/                     # Training materials
    ├── admin/                    # Admin training
    ├── operator/                 # Operator training
    ├── developer/                # Developer training
    └── user/                     # End-user training
```

### 1.7 Security Structure

```
security/
├── certificates/                    # Certificate authority artifacts
│   ├── root-ca/                     # Root CA (offline, air-gapped)
│   │   ├── ca.key (encrypted)       # Root CA private key (HSM)
│   │   ├── ca.crt                   # Root CA certificate
│   │   └── config/                  # Root CA openssl config
│   ├── national-ca/                 # National CA (online, HSM)
│   │   ├── ca.key                   # National CA private key
│   │   ├── ca.crt                   # National CA certificate
│   │   └── issued/                  # Issued service certificates
│   ├── ministry-ca/                 # Per-ministry issuing CAs
│   │   └── {ministry}/
│   ├── device-ca/                   # Device identity CA
│   ├── crl/                         # Certificate revocation lists
│   └── scripts/                     # Certificate lifecycle scripts
├── policies/                        # Policy as Code
│   ├── opa-rego/                    # OPA/Rego policies
│   ├── kyverno/                     # Kyverno K8s policies
│   └── gatekeeper/                  # OPA Gatekeeper constraints
├── vault/                           # Vault configuration
│   ├── transit/                     # Transit engine
│   ├── pki/                         # PKI engine
│   ├── identity/                    # Identity engine
│   └── audit/                       # Audit device logging
├── hardening/                       # System hardening
├── siem/                            # Security information & event management
├── incident-response/               # IR procedures
│   ├── playbooks/                   # Incident response playbooks
│   └── forensics/                   # Forensics tooling
├── audit/                           # Security audit artifacts
├── secrets/                         # Secret management
└── identity/                        # Identity security
```

### 1.8 CI/CD Structure

```
.github/
├── workflows/
│   ├── ci.yml                       # Build, lint, test all services
│   ├── cd-national.yml              # Deploy to national DC
│   ├── cd-regional.yml              # Deploy to regional hubs
│   ├── cd-edge.yml                  # Deploy to edge nodes
│   ├── security-scan.yml            # SAST, DAST, dependency scan
│   ├── release.yml                  # Release & versioning
│   ├── sync-test.yml                # Sync protocol conformance
│   └── chaos-weekly.yml             # Weekly chaos engineering
├── actions/                         # Reusable composite actions
│   ├── setup-go/                    # Go setup with cache
│   ├── setup-rust/                  # Rust setup with cache
│   ├── build-service/               # Build and push service
│   ├── security-scan/               # Trivy, Semgrep, Gitleaks
│   └── deploy-k8s/                  # Deploy to Kubernetes
├── dependabot.yml                   # Dependency automation
├── CODEOWNERS                       # Ownership assignments
├── SECURITY.md                      # Security policy
└── CONTRIBUTING.md                  # Contribution guidelines
```

---

## 2. Architecture Principles

### 2.1 Foundational Principles

| Principle | Application |
|---|---|
| **Offline-first** | Every node functions fully disconnected. Local PG is source of truth until sync converges. |
| **Event-driven** | All state changes produce CloudEvents 1.0. Services communicate exclusively through events. |
| **Local-first** | Data owned by creating node. National DC is convergent replica, not authoritative source. |
| **Anti-entropy** | Sync uses merkle-tree reconciliation with delta compression. No single-master. No RAFT/Paxos across WAN. |
| **Zero-trust** | mTLS at transport, JWT at application, Ed25519 at data layer, TPM at device layer. |
| **Tenant isolation** | Each ministry = tenant with isolated schema, encryption keys, auth realms, and RLS. |
| **No SPOF** | Every service runs min 2 replicas at every tier. Edge nodes operate fully independently. |
| **Anti-fragile** | System strengthens under stress: chaos engineering, automatic healing, graceful degradation. |
| **PostgreSQL-only** | One database engine for all persistence roles. No polyglot persistence. |
| **Immutable audit** | All events appended to hash-chain ledger. No UPDATE/DELETE on audit data. |
| **Fault isolation** | Service failures contained within bounded context. Bulkhead pattern for resource isolation. |
| **Anti-corruption** | Bounded contexts with well-defined translation layers between domains. |

### 2.2 Design Constraints

| Constraint | Rationale |
|---|---|
| Go primary, Rust for sync engine | Single binary deployment, ARM64 edge support, memory safety for correctness-critical code |
| No external IdP dependency | Must work fully offline; custom OAuth2/OIDC IdP built into identity-service |
| Docker-only at edge (no K8s) | Minimal footprint, reduced operational complexity for 1000+ sites |
| K8s at national DC only | Orchestration benefits for multi-service deployment |
| mTLS for all wire communication | Zero-trust mandate; no network boundary trust |
| Events always signed | Non-repudiation for government audit requirements |
| All timestamps in UTC + local | Offline devices may have clock skew; both timestamps preserved |
| UUID v7 for all IDs | Time-ordered UUIDs for efficient B-tree index performance |
| No hardcoded secrets | All secrets injected at runtime via Vault, Docker secrets, or K8s secrets |

---

## 3. System Topology

### 3.1 Three-Tier Architecture

```
+--------------------------------------------------------------------+
|                      NATIONAL TIER (Baghdad DC)                     |
|  Kubernetes Cluster                                                 |
|  +----------------+  +----------------+  +----------------------+   |
|  | Identity IdP   |  | Audit Ledger   |  | National Sync Hub   |   |
|  | (3 replicas)   |  | (3 replicas)   |  | (3 replicas)        |   |
|  +----------------+  +----------------+  +----------------------+   |
|  +----------------+  +----------------+  +----------------------+   |
|  | Analytics Svc  |  | Policy Engine  |  | Workforce State      |   |
|  | (2 replicas)   |  | (3 replicas)   |  | Engine (2 replicas)  |   |
|  +----------------+  +----------------+  +----------------------+   |
|  +----------------+  +----------------+  +----------------------+   |
|  | NATS Cluster   |  | Redis Cluster  |  | Patroni PG Cluster   |   |
|  | (3 nodes)      |  | (3 nodes)      |  | (3 nodes + replicas) |   |
|  +----------------+  +----------------+  +----------------------+   |
|  +----------------+  +----------------+                             |
|  | MinIO          |  | Keycloak       |                             |
|  | (distributed)  |  | (cluster)      |                             |
|  +----------------+  +----------------+                             |
+--------------------------------------------------------------------+
           | Encrypted WAN (mTLS 1.3, IPsec tunnels)
           v
+--------------------------------------------------------------------+
|                   REGIONAL TIER (x18 Governorates)                   |
|  Docker Compose / Nomad                                             |
|  +----------------+  +----------------+  +----------------------+   |
|  | Regional Sync  |  | Regional Audit |  | Identity Cache       |   |
|  | Relay (x2)     |  | Buffer         |  | (Redis)              |   |
|  +----------------+  +----------------+  +----------------------+   |
|  +----------------+  +----------------+  +----------------------+   |
|  | Attendance Agg |  | Leave Service  |  | Device Gateway       |   |
|  | (2 replicas)   |  | (2 replicas)   |  | (regional proxy)     |   |
|  +----------------+  +----------------+  +----------------------+   |
|  | PG Streaming Replica | NATS Leaf Node | MinIO Gateway | Redis  |
+--------------------------------------------------------------------+
           | LAN / Mesh / Encrypted Radio / 4G Backup
           v
+--------------------------------------------------------------------+
|                     EDGE TIER (1000+ Sites)                          |
|  Docker Compose                                                     |
|  +----------------+  +----------------+  +----------------------+   |
|  | Local PG       |  | Sync Engine    |  | Attendance Service   |   |
|  | (offline store) |  | (merkle tree)  |  | (offline-first)      |   |
|  +----------------+  +----------------+  +----------------------+   |
|  +----------------+  +----------------+  +----------------------+   |
|  | Leave Approver |  | Device Gateway |  | Biometric Devices    |   |
|  | (offline)      |  | (edge proxy)   |  | (LAN-connected)      |   |
|  +----------------+  +----------------+  +----------------------+   |
|  | Redis Cache    |  | pg_notify      |  | Local Monitoring     |   |
+--------------------------------------------------------------------+
```

### 3.2 Communication Matrix

| Source | Target | Protocol | Authentication | Purpose |
|---|---|---|---|---|
| Service | Service (same node) | gRPC | mTLS + JWT | Internal RPC |
| Service | Service (same node) | NATS/pg_notify | mTLS | Event pub/sub |
| Edge Node | Regional Hub | HTTPS + WS | mTLS | Sync upload/download |
| Regional Hub | National DC | HTTPS + WS | mTLS | Sync relay |
| Edge Node | Edge Node (LAN) | mDNS + gRPC | mTLS | Mesh sync |
| Mobile App | Edge/Cloud | HTTPS | Cert pinning + JWT | Employee ops |
| Web Portal | National DC | HTTPS | OAuth2 + OIDC | Admin dashboard |
| Biometric Device | Edge Gateway | gRPC/HTTP | mTLS (device cert) | Clock events |
| Admin | Any node | HTTPS | OAuth2 + MFA | Management |

---

## 4. Core Services

### 4.1 Service Map

```
                    +--------------------------------------+
                    |          platform-core                |
                    | (Shared Kernel: types, schemas,      |
                    |  proto, crypto, validation)          |
                    +--------------------------------------+
                               | depends on
                               v
+----------+  +----------+ +----------+ +----------+ +----------+
| identity |  |   auth   | |   authz  | |   sync   | |  audit   |
| -service |  | -service | | -service | | -engine  | | -ledger  |
+----------+  +----------+ +----------+ +----------+ +----------+
                               |
         +---------------------+---------------------+
         |                                           |
+--------v--------+                         +--------v--------+
| attendance-      |                         | leave-service   |
| service          |                         |                 |
+--------+--------+                         +--------+--------+
         |                                           |
         v                                           v
+--------+--------+                         +--------+--------+
| device-gateway   |                         | workforce-state |
| (biometric,hw)   |                         | -engine (CQRS)  |
+------------------+                         +--------+--------+
                                                            |
                                                            v
                                                   +--------+--------+
                                                   | analytics-       |
                                                   | service          |
                                                   +------------------+
```

### 4.2 Technology Stack

| Category | Technology | Justification |
|---|---|---|
| Core language | Go 1.22+ | Single binary, fast compile, ARM64 cross-compile, excellent concurrency |
| Sync engine | Rust | Memory safety for correctness-critical sync, zero-cost abstractions |
| Database | PostgreSQL 16 | Logical replication, RLS, partitioning, pg_notify, pg_cron, JSONB |
| Event bus (national) | NATS JetStream | Lightweight, at-least-once, exactly-once, cluster mode, leaf nodes |
| Event bus (edge) | pg_notify | Zero additional deps, works offline, no separate infrastructure |
| Message serialization | Protocol Buffers + JSON | Protobuf for gRPC/sync, JSON for REST/events |
| Sync protocol | Custom (merkle tree + delta) | Requirements-specific; no off-the-shelf meets all constraints |
| Container runtime | Docker (edge), K8s (national) | Docker simplicity at edge, K8s orchestration at national DC |
| Identity | Custom OAuth2/OIDC IdP | Must work fully offline; no external IdP dependency |
| Secrets | HashiCorp Vault (national), Docker secrets (edge) | Tier-appropriate secret management |
| Monitoring | Prometheus + Grafana (pull at DC, push at edge) | Pull model for DC, push-based edge agents |
| Logging | Structured JSON stdout -> Vector/Fluentd -> Loki | Cloud-native logging pattern |
| Tracing | OpenTelemetry -> Tempo | Distributed traces across services |
| Policy evaluation | OPA/Rego | Policy-as-code, versioned, testable rules |
| Object storage | MinIO | S3-compatible, self-hosted, no cloud dependency |

### 4.3 Service Dependencies

| Service | Depends On |
|---|---|
| identity-service | platform-core, postgres, redis, vault |
| authentication-service | platform-core, postgres, redis, identity-service |
| authorization-service | platform-core, postgres, redis, policy-engine |
| sync-engine | platform-core, postgres, nats |
| audit-ledger | platform-core, postgres, nats, minio |
| attendance-service | platform-core, postgres, redis, identity-service, sync-engine, policy-engine, device-gateway |
| leave-service | platform-core, postgres, identity-service, sync-engine, policy-engine |
| workforce-state-engine | platform-core, postgres, redis, nats (consumer) |
| notification-service | platform-core, postgres, identity-service |
| policy-engine | platform-core, postgres, redis |
| analytics-service | platform-core, postgres, minio, nats (consumer) |
| device-gateway | platform-core, postgres, redis, identity-service |

---

## 5. Service Boundaries

### 5.1 identity-service

| Attribute | Specification |
|---|---|
| **Responsibility** | User registration, authentication (password, biometric, certificate), ministry realm management, session management, device enrollment, SCIM provisioning, federation (SAML/LDAP bridge) |
| **Storage Model** | PostgreSQL (`inwp_identity`, schema `idm_*`): users, devices, sessions, realms, credentials. Redis: session cache, rate limit counters. Vault: credential hashes, encryption keys. |
| **API Boundaries** | gRPC (internal): `IdentityService` RPCs for user CRUD, auth, device mgmt. REST (external): OAuth2/OIDC endpoints (`/auth/authorize`, `/auth/token`, `/auth/refresh`, `/auth/revoke`, `/.well-known/openid-configuration`). SCIM: `/scim/v2/Users`, `/scim/v2/Groups` |
| **Event Contracts** | Produces: `inwp.identity.v1.*` (user.registered, user.verified, user.deactivated, user.reactivated, credential.changed, credential.revoked, role.assigned, role.revoked, device.enrolled, device.suspended, device.revoked, device.trust.changed, session.created). Consumes: `inwp.system.v1.*`, `inwp.sync.v1.*` |
| **Sync Behavior** | User/role/group/device changes -> events published -> sync engine distributes to all nodes. Edge nodes maintain local cache of active users for offline auth. Offline sessions cached for configurable duration (default 24h). Device trust scores synced via dedicated sync channel. |
| **Scaling Strategy** | Horizontal scaling via stateless auth endpoints. Session data in Redis (externalized). Read replicas for user queries. Write master handles registrations. Target: 1000 auth requests/sec per replica. |
| **Fault Tolerance** | 3+ replicas behind load balancer. Redis Sentinel for session cache HA. Offline auth via local session cache when national DC unreachable. Circuit breaker on LDAP/SAML federation calls. |
| **Security Boundaries** | mTLS required for all gRPC. Rate limiting per IP/user/ministry. Argon2id for password hashing. Biometric templates never stored -- only hash + confidence. Session tokens short-lived (15min access, 24h refresh). MFA enforced per realm policy. |
| **Deployment Strategy** | Kubernetes Deployments (national DC) with HPA (CPU > 70%). Docker Compose at regional hubs (identity cache). Edge nodes: read-only replica of active user directory. |

### 5.2 authentication-service

| Attribute | Specification |
|---|---|
| **Responsibility** | Dedicated authentication provider: password verification, TOTP validation, WebAuthn/FIDO2, biometric authentication, smart card (PKCS#11) verification, certificate chain validation, MFA orchestration, authentication policy enforcement |
| **Storage Model** | PostgreSQL (`inwp_auth`, schema `auth_*`): credential records, MFA devices, authentication policies, auth attempt audit log. Redis: rate limit counters, brute-force detection state, temporary auth challenge state. |
| **API Boundaries** | gRPC (internal): `Authenticate`, `ValidateMFA`, `VerifyBiometric`, `VerifyCertificate`, `Challenge`. REST (internal health): `/health`, `/ready`. |
| **Event Contracts** | Produces: `inwp.identity.v1.auth.attempted`, `inwp.identity.v1.auth.succeeded`, `inwp.identity.v1.auth.failed`, `inwp.security.v1.brute-force.detected`. Consumes: `inwp.identity.v1.user.*`, `inwp.identity.v1.credential.*`. |
| **Sync Behavior** | Authentication policies synced from identity-service. Failed attempt counters synced across replicas via Redis. No user data sync -- reads from identity-service. |
| **Scaling Strategy** | Stateless -- scales horizontally with Redis-backed rate limiting. Target: 5000 auth operations/sec per replica. Biometric verification may be CPU-bound; consider GPU instances for high-volume deployments. |
| **Fault Tolerance** | Circuit breaker on identity-service dependency. Local credential cache for offline authentication (cached public keys, certificate chains). Graceful degradation: MFA downgrade to password-only when MFA service unavailable (configurable per ministry). |
| **Security Boundaries** | Rate limiting: 10 attempts/min per user, 100 attempts/min per IP. Account lockout after N consecutive failures (configurable per ministry). All auth attempts logged to audit. Brute-force detection with exponential backoff. |
| **Deployment Strategy** | Co-located with identity-service. K8s Deployment with anti-affinity. HPA on CPU and request rate. |

### 5.3 authorization-service

| Attribute | Specification |
|---|---|
| **Responsibility** | Policy-based access control: RBAC role evaluation, ABAC attribute evaluation, policy decision point (PDP), resource-level permissions, ministry/site scoped authorization, dynamic policy evaluation via OPA/Rego |
| **Storage Model** | PostgreSQL (`inwp_authz`, schema `authz_*`): role definitions, role assignments, permission mappings, resource policies. Redis: policy cache (hot path), decision cache with TTL. |
| **API Boundaries** | gRPC (internal): `Authorize`, `CheckPermission`, `GetPermissions`, `EvaluatePolicy`. REST (admin): `/policies`, `/roles`, `/permissions`. |
| **Event Contracts** | Produces: `inwp.policy.v1.evaluated`, `inwp.security.v1.access.denied`, `inwp.security.v1.access.granted`. Consumes: `inwp.identity.v1.role.*`, `inwp.policy.v1.*`. |
| **Sync Behavior** | Policy definitions synced from policy-engine. Role assignments synced from identity-service. Decision cache invalidated on policy/role change events. |
| **Scaling Strategy** | Stateless -- scales horizontally. Policy cache in Redis allows zero-cold-start decisions. Target: 10000 evaluations/sec per replica. |
| **Fault Tolerance** | Policy cache provides read-through during database outage. Built-in default-deny on any evaluation error. Fallback to cached decisions when policy-engine unavailable. |
| **Security Boundaries** | Default-deny for all requests. Just-in-time policy compilation from Rego source. Policy evaluation timing out after 100ms. Audit log for every deny decision. |
| **Deployment Strategy** | Sidecar pattern: deployed alongside every service as a local PDP. Also deployed as centralized service for batch evaluations. |

### 5.4 attendance-service

| Attribute | Specification |
|---|---|
| **Responsibility** | Clock-in/out processing, shift schedule management, overtime calculation, attendance policy enforcement, biometric verification delegation, exception management, attendance record correction, offline queue management |
| **Storage Model** | PostgreSQL (`inwp_attendance`, schema `att_*`): clock_events (partitioned monthly by ministry), shifts, overtime_rules, exceptions, corrections. Redis: active session cache, rate limits, policy cache. |
| **API Boundaries** | gRPC (internal): `AttendanceService` (ClockIn, ClockOut, GetRecord, ListRecords, SyncAttendance). REST (external): `/v1/attendance`, `/v1/shifts`, `/v1/overtime`, `/v1/exceptions`. |
| **Event Contracts** | Produces: `inwp.attendance.v1.*` (clock-in.created, clock-out.created, break.started, break.ended, attendance.corrected, attendance.disputed, attendance.exception.*, shift.*, policy.*). Consumes: `inwp.identity.v1.user.*`, `inwp.policy.v1.*`, `inwp.sync.v1.*`. |
| **Sync Behavior** | Sync Priority: HIGH (payroll-critical). All clock events buffered locally when offline, batch-synced on reconnect. Dedup by event_id. Conflicts resolved via LWW (timestamp) for immutable clock events. |
| **Scaling Strategy** | Horizontal scaling. Write-heavy -- partition by ministry + month. Read replicas for historical queries. HPA on event ingestion rate. Target: 10000 clock events/min per replica. |
| **Fault Tolerance** | Full offline operation: all CRUD succeeds locally, event_outbox stores pending events. Queue depth unlimited (disk-based). Exponential backoff on reconnect. Conflict queue for admin review. |
| **Security Boundaries** | Device identity required for clock events (mTLS from device gateway). Geo-fencing: clock-in only allowed within site boundary. Time-fencing: clock-in only within shift window (+ grace period). Biometric verification required per ministry policy. |
| **Deployment Strategy** | K8s Deployment at national DC (3 replicas). Docker Compose at regional hubs (2 replicas). Docker Compose at every edge node (1 instance). |

### 5.5 leave-service

| Attribute | Specification |
|---|---|
| **Responsibility** | Leave request management, approval workflow orchestration, leave balance tracking, accrual processing, ministry-specific leave types (Hajj, Umrah, examination, etc.), calendar integration, fiscal year management |
| **Storage Model** | PostgreSQL (`inwp_leave`, schema `lv_*`): leave_requests, leave_balances, approval_chains, accrual_policies, leave_types. Partitioned by ministry + fiscal year. |
| **API Boundaries** | gRPC (internal): `LeaveService` (RequestLeave, ApproveLeave, RejectLeave, CancelLeave, GetBalance, GetRequests). REST (external): `/v1/leaves`, `/v1/balances`, `/v1/approvals`. |
| **Event Contracts** | Produces: `inwp.leave.v1.*` (request.created, request.submitted, request.approved, request.rejected, request.cancelled, request.expired, balance.adjusted, balance.accrued, accrual.processed). Consumes: `inwp.identity.v1.user.*`, `inwp.attendance.v1.*`, `inwp.policy.v1.*`. |
| **Sync Behavior** | Sync Priority: MEDIUM. Offline request/approve capability. Balance adjustments use PN-Counter CRDT for conflict-free merge. Accrual processed as batch operation during low-activity windows. |
| **Scaling Strategy** | Horizontal scaling. Read-heavy with occasional write bursts during end-of-month. Batch accrual processing can be background job (not real-time). Partition by ministry. |
| **Fault Tolerance** | Offline leave request creation and approval. Pending approvals queued locally and synced. Balance calculations use local snapshots with reconciliation on sync. |
| **Security Boundaries** | Hierarchical approval chains enforced at application layer. Approver must have `leave_approver` role at appropriate scope. Self-approval prohibited. Balance queries require employee or HR scope. |
| **Deployment Strategy** | K8s Deployment at national DC (2 replicas). Docker Compose at regional hubs (2 replicas). Docker Compose at edge nodes (1 instance, may be subset functionality). |

### 5.6 workforce-state-engine

| Attribute | Specification |
|---|---|
| **Responsibility** | CQRS state projection: consumes events from all domains, projects current employee state, computes work history, generates timesheets, determines eligibility, provides real-time state queries, publishes aggregated state |
| **Storage Model** | PostgreSQL (`inwp_workforce`, schema `wf_*`): materialized views (employee_state, work_history, timesheets, eligibility). Redis: real-time employee state cache with pub/sub for live updates. |
| **API Boundaries** | gRPC (internal): `WorkforceStateService` (GetEmployeeState, GetWorkHistory, GetTimesheet, CheckEligibility). REST (external): `/v1/workforce/state`, `/v1/timesheets`. |
| **Event Contracts** | Produces: `inwp.workforce.v1.*` (employee.state.changed, timesheet.generated, work.history.updated, eligibility.changed, snapshot.computed). Consumes: ALL domain events (attendance, leave, identity, policy, device). |
| **Sync Behavior** | Sync Priority: LOW (read model, eventual consistency). State rebuilt from event replay on new node or recovery. Snapshots persisted periodically for faster recovery. |
| **Scaling Strategy** | Horizontal scaling of read replicas. Write master handles event consumption and state projection. Partition by employee_id hash for parallel event processing. Redis pub/sub for real-time subscribers. |
| **Fault Tolerance** | Event sourcing allows full state rebuild from event log. Redis cache provides read-through during rebuild. Materialized views refreshed on schedule or on-demand. |
| **Security Boundaries** | Read-only API (state queries). All state derived from domain events -- no direct mutation. Access controlled by authorization-service policies. |
| **Deployment Strategy** | K8s Deployment at national DC (2 replicas). Deployed where full event stream is available. Not typically deployed at edge (edge has limited event history). |

### 5.7 sync-engine

| Attribute | Specification |
|---|---|
| **Responsibility** | Merkle tree reconciliation, delta computation, conflict resolution (CRDT/LWW/merge), data compression (zstd), bandwidth management, LAN discovery (mDNS), sync scheduling, peer-to-peer mesh sync, checkpoint/resume, store-and-forward for disconnected regions |
| **Storage Model** | PostgreSQL (`inwp_sync`, schema `sync_*`): sync_checkpoints (merkle roots per partition), sync_batch_log, sync_conflicts, sync_queue, node_registry. |
| **API Boundaries** | gRPC (internal): `SyncService` (InitiateSync, GetMerkleRoot, GetDelta, ApplyDelta, ResolveConflict, GetStatus). HTTPS + WebSocket (external peer-facing): sync protocol endpoints. |
| **Event Contracts** | Produces: `inwp.sync.v1.*` (batch.committed, conflict.detected, conflict.resolved, heartbeat.sent, schema.negotiated, node.state.changed). Consumes: all domain events for routing. |
| **Sync Behavior** | Self-synchronizing (meta-sync protocol). 5-phase sync: Discovery -> Merkle Exchange -> Delta Transfer -> Reconciliation -> Commitment. Conflict resolution per entity matrix. Bandwidth throttling per node. Priority queuing. |
| **Scaling Strategy** | Written in Rust for performance. Per-partition parallel sync (ministry + entity-type + time-bucket). Horizontal scaling: each replica handles subset of partitions. Target: 10000 events/sec sync throughput per replica. |
| **Fault Tolerance** | Checkpoint/resume on any interruption. Store-and-forward for disconnected regions. Dead-letter queue for failed sync batches. Automatic retry with exponential backoff. Peer mesh provides alternative sync paths. |
| **Security Boundaries** | mTLS required for all sync connections. Ed25519 signing of all sync batches. Merkle root publication for integrity verification. Node identity verified against device CA. |
| **Deployment Strategy** | K8s Deployment at national DC (3 replicas -- Sync Hub). Docker Compose at regional hubs (2 replicas -- Sync Relay). Docker Compose at edge nodes (1 instance -- Sync Agent). Rust binary, statically linked, minimal container. |

### 5.8 audit-ledger

| Attribute | Specification |
|---|---|
| **Responsibility** | Immutable event ingestion, hash-chain construction, tamper-evident seal generation, periodic merkle root publication, compliance queries, retention management, WAL-based replication, cryptographic verification |
| **Storage Model** | PostgreSQL (`inwp_audit`, schema `ledger_*`): ledger_entries (append-only, hash-chained), ledger_seals, integrity_checks. MinIO (cold storage for aged-out entries, sealed archives). |
| **API Boundaries** | gRPC (internal): `AuditLedgerService` (AppendEntry, GetEntry, VerifyChain, GetSeal, RangeQuery). REST (read-only compliance): `/v1/audit/entries`, `/v1/audit/seals`, `/v1/audit/verify`. |
| **Event Contracts** | Produces: `inwp.audit.v1.*` (entry.appended, seal.generated, seal.verified, integrity.failure). Consumes: ALL events from all services (the ledger records every event). |
| **Sync Behavior** | Append-only -- no conflict resolution needed. WAL-based streaming replication to regional hubs. Periodic seal generation (every 10000 entries or 1 hour). Seals published to public transparency log. |
| **Scaling Strategy** | Write-heavy ingestion. Batch writes for throughput. Partition by time (monthly). Read replicas for compliance queries. MinIO archive for aged-out hot data. |
| **Fault Tolerance** | Immutable by design -- no UPDATE/DELETE permitted. Hash-chain detects any tampering. WAL replication provides DR. Multiple seal verification nodes can independently verify chain integrity. |
| **Security Boundaries** | Only append operations permitted (no UPDATE, no DELETE). Ed25519 signing of each entry. Periodic seals signed by dedicated seal key. Public verification of seals. Write access restricted to event bus, read access controlled by authorization-service. |
| **Deployment Strategy** | K8s StatefulSet at national DC (3 replicas). Regional hub maintains audit buffer (streaming replica). Edge nodes do NOT run full audit ledger (resource-constrained); forward all events to regional hub. |

### 5.9 notification-service

| Attribute | Specification |
|---|---|
| **Responsibility** | Multi-channel notification dispatch (push/FCM, SMS, email, on-screen display, in-app notification), template engine with Arabic/Kurdish/English localization, delivery guarantees (at-least-once), read receipts, ministry branding, notification preferences |
| **Storage Model** | PostgreSQL (`inwp_notification`, schema `notif_*`): notifications, templates, delivery_log, user_preferences, channels. |
| **API Boundaries** | gRPC (internal): `NotificationService` (SendNotification, GetStatus, GetNotifications). REST (admin): `/v1/notifications`, `/v1/templates`, `/v1/preferences`. |
| **Event Contracts** | Produces: `inwp.notification.v1.*` (notification.created, notification.sent, notification.delivered, notification.read, notification.failed, notification.expired). Consumes: various domain events to trigger notifications. |
| **Sync Behavior** | Offline-capable delivery queue. Notifications queued locally if channel unavailable. Delivery retry with exponential backoff. Read receipts synced on reconnect. |
| **Scaling Strategy** | Channel-specific worker pools. Horizontal scaling with shared delivery queue in PostgreSQL. FCM/SMS channels may have external API rate limits -- worker pool size tuned accordingly. |
| **Fault Tolerance** | Delivery queue persisted in PostgreSQL. Dead-letter queue after max retries. Fallback channel: if push fails, try SMS, then email. Ministry-configurable fallback chains. |
| **Security Boundaries** | Template rendering: no arbitrary code execution. Rate limiting per ministry per channel. PII content encrypted at rest. Delivery audit trail for compliance. |
| **Deployment Strategy** | K8s Deployment at national DC (2 replicas). Regional hubs may run notifications for local delivery. Edge nodes: minimal notification queue for on-screen alerts. |

### 5.10 policy-engine

| Attribute | Specification |
|---|---|
| **Responsibility** | Centralized policy definition, storage, versioning, evaluation, and distribution. Supports: attendance rules, leave policies, overtime rules, security policies, sync policies, approval chains. Policy-as-code (OPA/Rego compatible). |
| **Storage Model** | PostgreSQL (`inwp_policy`, schema `pol_*`): policy_definitions, policy_versions, policy_activations, evaluation_logs. Redis: policy cache (hot path, <5ms evaluation). |
| **API Boundaries** | gRPC (internal): `PolicyService` (Evaluate, GetPolicy, ListPolicies, DryRun). REST (admin): `/v1/policies`, `/v1/policies/{id}/versions`, `/v1/evaluate`. |
| **Event Contracts** | Produces: `inwp.policy.v1.*` (policy.created, policy.updated, policy.activated, policy.deactivated, policy.evaluated, policy.violation.detected). Consumes: `inwp.identity.v1.*`, `inwp.system.v1.*`. |
| **Sync Behavior** | Policies synced to all nodes as configuration objects. Policy cache invalidated on policy change events. Versioned policies allow gradual rollout (canary -> regional -> national). |
| **Scaling Strategy** | Read-heavy: policy evaluation on every request path. Redis cache absorbs majority of reads. Policy write (admin) operations low volume. Horizontal scaling for evaluation capacity. |
| **Fault Tolerance** | Policy cache read-through during database outage. Default-deny on evaluation failure. Compiled policies cached locally. Policy version pinning prevents partial deployment. |
| **Security Boundaries** | Policy compilation sandboxed (no file system, no network from Rego runtime). Policy CRUD restricted to ministry_admin and above. Every policy change produces audit event. Dry-run mode for testing before activation. |
| **Deployment Strategy** | K8s Deployment at national DC (3 replicas). Regional hubs: local policy cache. Edge nodes: embedded OPA sidecar with local policy bundles. |

### 5.11 analytics-service

| Attribute | Specification |
|---|---|
| **Responsibility** | Workforce analytics: attendance summaries, leave utilization reports, workforce KPIs, ministry dashboards, trend analysis, anomaly detection, data export (PDF, CSV, Excel), scheduled report generation, data lake management |
| **Storage Model** | PostgreSQL (`inwp_analytics`, schema `an_*`): aggregated tables, materialized views, report definitions, dashboard configs, schedule definitions. MinIO (Parquet files, raw event archive, export artifacts). |
| **API Boundaries** | REST (dashboard API, export): `/v1/analytics/dashboards`, `/v1/analytics/reports`, `/v1/analytics/metrics`, `/v1/analytics/export`. gRPC (internal): `AnalyticsService` (GetMetric, GetDashboardData, TriggerReport). |
| **Event Contracts** | Produces: `inwp.analytics.v1.*` (report.generated, report.scheduled, metric.computed, anomaly.detected). Consumes: ALL events (consumer of last resort for data lake). |
| **Sync Behavior** | Sync Priority: LOW. Read model, eventual consistency. Data lake populated via event stream. Aggregations recomputed on schedule or on-demand. |
| **Scaling Strategy** | Computation jobs run as background workers (K8s Jobs/CronJobs). Dashboard reads served from materialized views. Report generation scales horizontally via work queue. |
| **Fault Tolerance** | Materialized views survive database restart. Failed report generation retries. Data lake events never deleted (replayable from event stream). |
| **Security Boundaries** | Report data scoped by ministry/site RBAC. Exports contain only authorized data. Anomaly detection alerts routed to security if anomaly indicates fraud. |
| **Deployment Strategy** | K8s Deployment + CronJobs at national DC. Regional hubs: local aggregation for performance. Edge: no analytics (data forwarded to region). |

### 5.12 device-gateway

| Attribute | Specification |
|---|---|
| **Responsibility** | Device abstraction layer, protocol bridging (ZKteco, Suprema, HID, RS485, MQTT, generic HTTP), device health monitoring, firmware management, trust score computation, device attestation verification, clock event buffering and forwarding |
| **Storage Model** | PostgreSQL (`inwp_device`, schema `dev_*`): device_registry, clock_event_buffer, firmware_versions, health_history, trust_scores. Redis: device session cache, rate limiters. |
| **API Boundaries** | gRPC (service-facing): `DeviceGatewayService` (RegisterDevice, GetDevice, ListDevices, GetClockEvents, UpdateFirmware). REST/HTTP (device-facing): device protocol endpoints, health, clock-in/out. MQTT (IoT device-facing): sensor data ingestion. |
| **Event Contracts** | Produces: `inwp.device.v1.*` (device.connected, device.disconnected, device.firmware.available, device.firmware.updated, device.diagnostic.reported, device.config.updated, device.alert.generated). Consumes: `inwp.identity.v1.device.*`, `inwp.policy.v1.*`. |
| **Sync Behavior** | Clock events buffered locally and forwarded to attendance-service. Device config synced from edge node. Firmware updates distributed via sync. Biometric templates NEVER synced across WAN -- remain local. |
| **Scaling Strategy** | Device connections are stateful -- sticky sessions to maintain device connections. Scale by device group (site-based partitioning). Protocol adapter workers scale independently. |
| **Fault Tolerance** | Device clock events buffered in PostgreSQL during attendance-service outage. Health monitoring with automatic reconnection. Trust decay computation runs regardless of connectivity. |
| **Security Boundaries** | Each device authenticated via mTLS (device certificate signed by Device CA). Device trust level enforced: TRUSTED = full access, PROVISIONAL = limited, DEGRADED/SUSPENDED/REVOKED = blocked. Biometric templates never leave the enrolling device. |
| **Deployment Strategy** | K8s Deployment at national DC (device management). Docker Compose at edge nodes (local device proxy). Physical device connectivity limited to LAN -- device-gateway must be on same LAN segment as devices. |

---

## 6. Event Architecture

### 6.1 Event Schema Standard

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
  "ministry_id": "uuid",
  "site_id": "uuid",
  "device_id": "uuid",
  "user_id": "uuid",
  "offline_generated": true,
  "local_timestamp": "2026-06-01T13:00:00.000+03:00",
  "sync_id": "uuid",
  "trace_id": "uuid"
}
```

### 6.2 Complete Event Taxonomy

```
inwp
+-- attendance.v1
|   +-- clock-in.created
|   +-- clock-out.created
|   +-- break.started
|   +-- break.ended
|   +-- attendance.corrected
|   +-- attendance.disputed
|   +-- attendance.exception.created
|   +-- attendance.exception.justified
|   +-- attendance.exception.resolved
|   +-- attendance.exception.escalated
|   +-- shift.created
|   +-- shift.modified
|   +-- shift.deactivated
|   +-- policy.created
|   +-- policy.activated
|   +-- policy.superseded
|
+-- leave.v1
|   +-- request.created
|   +-- request.submitted
|   +-- request.approved
|   +-- request.rejected
|   +-- request.cancelled
|   +-- request.expired
|   +-- request.completed
|   +-- balance.adjusted
|   +-- balance.accrued
|   +-- balance.deducted
|   +-- balance.expired
|   +-- balance.warning
|   +-- accrual.processed
|   +-- accrual.policy.created
|   +-- accrual.policy.updated
|
+-- identity.v1
|   +-- user.registered
|   +-- user.verified
|   +-- user.deactivated
|   +-- user.reactivated
|   +-- credential.added
|   +-- credential.changed
|   +-- credential.revoked
|   +-- role.assigned
|   +-- role.revoked
|   +-- device.enrolled
|   +-- device.activated
|   +-- device.suspended
|   +-- device.revoked
|   +-- device.trust.changed
|   +-- device.heartbeat
|   +-- realm.created
|   +-- realm.deactivated
|   +-- auth.policy.updated
|   +-- auth.attempted
|   +-- auth.succeeded
|   +-- auth.failed
|   +-- session.created
|   +-- session.terminated
|   +-- session.expired
|
+-- workforce.v1
|   +-- employee.state.changed
|   +-- timesheet.generated
|   +-- work.history.updated
|   +-- eligibility.changed
|   +-- snapshot.computed
|
+-- policy.v1
|   +-- policy.created
|   +-- policy.updated
|   +-- policy.activated
|   +-- policy.deactivated
|   +-- policy.evaluated
|   +-- policy.violation.detected
|   +-- policy.bundle.synced
|
+-- device.v1
|   +-- device.connected
|   +-- device.disconnected
|   +-- device.firmware.available
|   +-- device.firmware.updated
|   +-- device.diagnostic.reported
|   +-- device.config.updated
|   +-- device.alert.generated
|   +-- device.clock-event.forwarded
|
+-- sync.v1
|   +-- batch.committed
|   +-- conflict.detected
|   +-- conflict.resolved
|   +-- heartbeat.sent
|   +-- schema.negotiated
|   +-- node.state.changed
|   +-- node.online
|   +-- node.offline
|   +-- partition.synced
|
+-- audit.v1
|   +-- entry.appended
|   +-- seal.generated
|   +-- seal.verified
|   +-- integrity.failure
|
+-- notification.v1
|   +-- notification.created
|   +-- notification.sent
|   +-- notification.delivered
|   +-- notification.read
|   +-- notification.failed
|   +-- notification.expired
|
+-- analytics.v1
|   +-- report.generated
|   +-- report.scheduled
|   +-- metric.computed
|   +-- anomaly.detected
|
+-- security.v1
|   +-- access.granted
|   +-- access.denied
|   +-- brute-force.detected
|   +-- certificate.expiring
|   +-- certificate.expired
|   +-- trust.degraded
|   +-- anomaly.detected
|   +-- incident.triggered
|
+-- system.v1
    +-- node.online
    +-- node.offline
    +-- service.healthy
    +-- service.degraded
    +-- service.unhealthy
    +-- configuration.changed
    +-- maintenance.started
```

### 6.3 Event Lifecycle

```
[Producer Service]
    |
    |---(1) Create domain event in aggregate
    |---(2) Validate against JSON Schema (schema registry)
    |---(3) Sign with Ed25519 (producer private key from Vault)
    |---(4) Append to local event_outbox table (PostgreSQL)
    |---(5) Publish to event bus (NATS national / pg_notify edge)
    |
    v
[Event Bus]
    |
    |---(6) Content-based routing by type + tenant
    |---(7) Persistent store (NATS JetStream / event_outbox)
    |---(8) Deliver to subscribers (at-least-once)
    |
    v
[Consumer Service]
    |
    |---(9) Verify Ed25519 signature (producer public key)
    |---(10) Validate schema version (dataschema field)
    |---(11) Idempotent processing (dedup by event_id, idempotency window 24h)
    |---(12) Acknowledge or dead-letter on failure (max 3 retries)
    |
    v
[Audit Ledger]
    |
    |---(13) Append ALL events to immutable hash chain
    |---(14) Generate periodic seals (every 10000 entries or 1 hour)
    |---(15) Publish seal for external verification
```

### 6.4 Schemas Directory Structure

```
events/schemas/
+-- v1/
|   +-- attendance/
|   |   +-- clock-in.created.json
|   |   +-- clock-out.created.json
|   |   +-- break.started.json
|   |   +-- break.ended.json
|   |   +-- attendance.corrected.json
|   |   +-- attendance.disputed.json
|   |   +-- shift.created.json
|   |   +-- shift.modified.json
|   +-- leave/
|   +-- identity/
|   +-- audit/
|   +-- sync/
|   +-- device/
|   +-- policy/
|   +-- notification/
|   +-- workforce/
|   +-- analytics/
|   +-- security/
|   +-- system/
+-- registry/
|   +-- schema-registry.json
+-- versioning/
    +-- changelog.md
    +-- migrations/
```

### 6.5 Event Bus Architecture

```
                    NATIONAL DC
        +---------------------------------------+
        |         NATS JetStream Cluster        |
        |  +------+  +------+  +------+        |
        |  | Node1|  | Node2|  | Node3|        |
        |  +--+---+  +--+---+  +--+---+        |
        |     |          |          |            |
        |     +----------+----------+            |
        |                |                       |
        +----------------+-----------------------+
                         | NATS Leaf Node connections
        +----------------+-----------------------+
        |                |                       |
    +---v---+        +---v---+           +---v---+
    |Region1|        |Region2|           |Region3|
    |Leaf   |        |Leaf   |           |Leaf   |
    +---+---+        +---+---+           +---+---+
        |                |                   |
    +---v---+        +---v---+           +---v---+
    |Edge 1 |        |Edge 2 |           |Edge 3 |
    |pg_not.|        |pg_not.|           |pg_not.|
    +-------+        +-------+           +-------+
```

---

## 7. Synchronization Architecture

### 7.1 Design Principles

- **No single source of truth** -- truth is eventually converged across all nodes
- **Anti-entropy over consensus** -- merkle tree reconciliation; no RAFT/Paxos across WAN
- **Peer-to-peer on LAN** -- direct sync between edge nodes without hub mediation
- **Hub-and-spoke on WAN** -- regional relays optimize bandwidth and provide store-and-forward
- **CRDT-inspired** -- LWW, ministry-author-wins, service-merge, additive merge per entity type
- **Delta-only** -- only changed data transmitted; full snapshot on first sync only
- **Bandwidth-aware** -- compression, prioritization, scheduling, budgeting per node

### 7.2 Sync Protocol -- 5 Phases

```
Phase 1: DISCOVERY
  Edge Node A broadcasts presence via mDNS (LAN) or connects to assigned Regional Relay (WAN).
  Exchange: node_id, node_type, capabilities, schema versions, last checkpoint, certificate.
  Mutual authentication via mTLS.

Phase 2: MERKLE EXCHANGE
  Each node maintains a merkle tree per partition.
  Partition = {ministry_id}/{entity_type}/{time_bucket}
  Node A sends merkle root hashes. Node B compares, identifies differing branches.
  Recursively request missing leaf hashes until divergent record IDs identified.

Phase 3: DELTA TRANSFER
  For each differing record: full record + event chain.
  Compressed via zstd (L19 cold, L3 hot). Chunked at 1MB with checkpoint/resume.

Phase 4: RECONCILIATION
  Both nodes apply mutual changes. Conflicts resolved per entity matrix.
  Auto-resolvable: LWW, ministry-author-wins, CRDT merge.
  Manual: escalated to admin dashboard.

Phase 5: COMMITMENT
  Both compute new merkle root. Sign sync batch receipt.
  {sync_id, partition_key, events_count, new_merkle_root, timestamp}
  Receipt appended to local sync_batch_log (immutable).
  Advance checkpoint. Publish batch committed event.
```

### 7.3 Per-Entity Conflict Resolution

| Entity | Default Strategy | Overridable | Rationale |
|---|---|---|---|
| ClockEvent | LWW + Dedup | No | Immutable; dedup by event_id |
| AttendanceException | LWW (justification) | Yes | Latest justification valid |
| Shift | Ministry Author Wins | N/A | HR authoritative |
| AttendancePolicy | Ministry Author Wins | N/A | Only one active per site |
| LeaveRequest | Service Merge + Manual | Yes | State machine + human judgment |
| LeaveBalance | PN-Counter CRDT | No | Financial accuracy requires commutative merge |
| AccrualPolicy | LWW | Yes | Latest policy wins |
| UserProfile | LWW per field | Yes | Per-field independent resolution |
| RoleAssignment | Additive Merge (G-Set) | Yes | Union of role assignments |
| DeviceTrust | LWW | No | Computed value from trust engine |
| LedgerEntry | None (append-only) | N/A | Immutable by design |
| PolicyDefinition | Ministry Author Wins | N/A | Authoritative source |

### 7.4 Offline Operation

| Capability | Detail |
|---|---|
| Local writes | All CRUD succeed locally; event_outbox stores pending events |
| Local reads | Complete local PostgreSQL; zero dependency on remote data |
| Queue depth | Unlimited (disk-based); purged only after confirmed sync |
| Reconnection | Exponential backoff: 1s -> 5s -> 30s -> 5min -> 30min |
| Heartbeat | Every 60s; hub detects stale after 3 missed |
| Event replay | Full event log replay for new nodes or DR |

### 7.5 Bandwidth Management

| Technique | Implementation |
|---|---|
| Delta compression | zstd; only send diffs after initial sync |
| Priority queuing | Attendance=HIGH, Leave=MEDIUM, Analytics=LOW |
| Bandwidth budgeting | Per-node daily cap, Redis token bucket |
| Sync scheduling | Configurable windows per entity type |
| Chunking | 1MB chunks with checkpoint/resume |
| Image compression | 640x480 max, JPEG quality 70 |

---

## 8. Security Architecture

### 8.1 Zero-Trust Model

All requests verified at three layers regardless of origin:
1. **mTLS 1.3** -- verify certificate chain, CRL, validity
2. **JWT** -- verify signature, expiry, claims
3. **Policy Evaluation** -- RBAC + ABAC via OPA/Rego

### 8.2 Security Layers

| Layer | Mechanism |
|---|---|
| Transport | mTLS 1.3, certificate pinning on mobile |
| Authentication | OAuth2 + OIDC (external), JWT (internal) |
| Authorization | RBAC + ABAC (ministry/site/region/device context) |
| Data at rest | AES-256-GCM per-tenant envelope encryption |
| Data in transit | TLS 1.3 + payload-level Ed25519 signing |
| Device trust | TPM/TrustZone attestation, trust score decay |
| Audit | Immutable hash chain, cryptographic seals |
| Secrets | HashiCorp Vault (national), Docker secrets (edge) |
| Supply chain | Cosign signatures, SBOM, Trivy scanning |

### 8.3 Certificate Authority Hierarchy

```
            +-----------------------------+
            | INWP Root CA (offline)      |
            | Air-gapped HSM              |
            +-----------------------------+
                         |
          +--------------+--------------+
          |              |              |
+---------v----+  +------v------+  +---v----------+
| National CA  |  | Ministry CA |  | Device CA    |
| (HSM,online) |  | (per-minist) |  | (short TTL)  |
+--------------+  +-------------+  +--------------+
```

### 8.4 RBAC Structure

```
/national
  +-- national_admin       Full system access
  +-- national_auditor     Read-only audit, all ministries
  +-- system_operator      Infrastructure management
/ministries/{ministry_id}
  +-- ministry_admin       Full ministry access
  +-- ministry_hr          HR operations
  +-- ministry_auditor     Read-only ministry data
  +-- ministry_viewer      Dashboard read-only
/sites/{site_id}
  +-- site_admin           Site operations management
  +-- hr_operator          Attendance, leave operations
  +-- attendance_operator  Clock events, exceptions
  +-- leave_approver       Leave approval authority
  +-- viewer               Site data read-only
```

### 8.5 Device Trust Model

| Level | Criteria | Permissions |
|---|---|---|
| TRUSTED | Attestation verified, firmware current | Full read/write |
| PROVISIONAL | Enrolled, attestation incomplete | Read-only, limited write |
| DEGRADED | Expired cert, outdated firmware | Read-only, alert |
| SUSPENDED | Known vulnerability, anomaly | Blocked, admin notification |
| REVOKED | Compromised, decommissioned | All access blocked |

---

## 9. Deployment Topology

### 9.1 Ministry Level (National DC -- Baghdad)

```
K8s Cluster (3+ worker nodes)
Namespaces: inwp-{ministry}-{env}
Services (3 replicas each):
  identity-service, authentication-service, authorization-service
  attendance-service, leave-service, workforce-state-engine
  sync-engine (hub), audit-ledger, notification-service
  policy-engine, analytics-service, device-gateway (mgmt)
  web-portal, api-gateway
Infrastructure:
  Patroni PG (3 pods), Redis Cluster (3 pods)
  NATS JetStream (3 pods), MinIO (4 pods, erasure)
  Vault (3 pods), Keycloak (2 pods)
  Prometheus + Grafana, Loki + Tempo
```

### 9.2 Regional Level (x18 Governorates)

```
Docker Compose / Nomad
Services (2 replicas each):
  sync-engine (relay), attendance-service (agg)
  leave-service, device-gateway (proxy)
Infrastructure:
  PG streaming replica, Redis standalone
  NATS Leaf Node, MinIO Gateway
  Local Prometheus, Vector log shipper
```

### 9.3 Institution Level (Edge Node)

```
Docker Compose (Single Host)
Services (1 instance each):
  sync-engine (agent), attendance-service
  leave-service, device-gateway
Infrastructure:
  PostgreSQL (local), Redis (standalone)
  pg_notify (event bus), Node Exporter
Connected Devices:
  Biometric terminals, card readers
  Attendance kiosks, admin workstations
```

### 9.4 Disconnected Operation

Fully autonomous when WAN unavailable:
- All services run locally, clock-in/out works
- Leave requests & approvals work
- Local PostgreSQL is source of truth
- Events queue in event_outbox
- On reconnection: merkle discovery -> delta transfer -> conflict resolution -> normal

---

## 10. Infrastructure Architecture

### 10.1 Database-per-Service

| Service | Database | Schema |
|---|---|---|
| identity-service | `inwp_identity` | `idm_` |
| authentication-service | `inwp_auth` | `auth_` |
| authorization-service | `inwp_authz` | `authz_` |
| sync-engine | `inwp_sync` | `sync_` |
| audit-ledger | `inwp_audit` | `ledger_` |
| attendance-service | `inwp_attendance` | `att_` |
| leave-service | `inwp_leave` | `lv_` |
| workforce-state-engine | `inwp_workforce` | `wf_` |
| notification-service | `inwp_notification` | `notif_` |
| policy-engine | `inwp_policy` | `pol_` |
| analytics-service | `inwp_analytics` | `an_` |
| device-gateway | `inwp_device` | `dev_` |

### 10.2 Multi-Tenancy

Row-Level Security (RLS) on all tables:
```sql
ALTER TABLE attendance.clock_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY ministry_isolation ON attendance.clock_events
    USING (ministry_id = current_setting('app.current_ministry_id')::UUID);
```

### 10.3 Partitioning

Time-based partitioning for high-volume tables (monthly via pg_partman).

### 10.4 Port Mapping

| Service | Internal Port | External |
|---|---|---|
| identity-service | 50051 (gRPC), 8081 (REST) | 443 |
| sync-engine | 50052 (sync), 50053 (mesh) | 8443 |
| audit-ledger | 50054 | 443 |
| attendance-service | 50055 | 8080 |
| leave-service | 50056 | 8082 |
| notification-service | 50057 | 8083 |
| PostgreSQL | 5432 | - |
| Prometheus metrics | 9090 | - |
| Health/readiness | 8080 | - |

---

## 11. Observability & Monitoring

### 11.1 Stack

| Component | National DC | Regional Hub | Edge Node |
|---|---|---|---|
| Metrics | Prometheus (pull) | Prometheus (pull+push) | Node Exporter (push) |
| Logging | Vector -> Loki | Vector -> national | Vector -> regional |
| Tracing | OpenTelemetry -> Tempo | Forwarded | Not collected |
| Dashboards | Grafana | Grafana (read-only) | - |
| Alerting | Alertmanager | Alertmanager leaf | - |
| Long-term | Thanos (1yr retention) | Local (7d retention) | Local (1d) |

### 11.2 Key Metrics per Service

| Service | Key Metrics |
|---|---|
| identity-service | auth_requests_total, active_sessions, failed_attempts |
| attendance-service | clock_events_total, offline_ratio, sync_lag_seconds |
| leave-service | leave_requests_total, approval_duration_hours |
| sync-engine | sync_batches_total, conflict_rate, queue_depth |
| audit-ledger | entries_appended_total, seal_interval_ms |

### 11.3 SLI/SLO Framework

| SLI | SLO Target |
|---|---|
| Service availability (national) | 99.99% |
| Service availability (regional) | 99.9% |
| Sync latency (within region) | <5 min p99 |
| Sync latency (cross-region) | <1h p99 |
| API latency (p99) | <200ms |
| Clock event ingestion | <2s p99 |
| Audit integrity | 100% verification pass |

---

## 12. CI/CD Architecture

### 12.1 CI Pipeline

```
Code Push -> Lint -> Build -> Unit Test -> Security Scan -> Build Image -> Integration Tests -> Artifacts Published
```

### 12.2 CD Pipeline

```
CI Success -> Staging Deploy -> Smoke Tests -> Regional Canary -> Regional Rollout -> Edge Rollout
```

### 12.3 GitHub Actions Workflows

`.github/workflows/ci.yml` -- Build, lint, test all services on every push/PR
`.github/workflows/cd-national.yml` -- Deploy to national DC with approval gate
`.github/workflows/cd-regional.yml` -- Phased rollout to 18 regional hubs
`.github/workflows/cd-edge.yml` -- 10% per day rollout to edge nodes
`.github/workflows/security-scan.yml` -- Daily SAST/DAST/dependency scan
`.github/workflows/chaos-weekly.yml` -- Weekly chaos engineering tests

---

## 13. Testing Strategy

### 13.1 Test Pyramid

```
          /\          E2E Tests (5%)
         /  \
        /    \
       / Integ \
      /  ation  \
     /   Tests   \
    /   (25%)     \
   /----------------\
  /                  \
 /   Unit Tests       \
/(70% of test effort)  \
/----------------------\
```

### 13.2 Testing Directories

```
testing/
+-- unit/                 # Unit tests (simulated dependencies)
+-- integration/          # Integration tests (real dependencies)
+-- e2e/                  # Full system workflows
+-- performance/          # Load, stress, endurance (k6)
+-- chaos/                # Chaos engineering (Chaos Mesh)
+-- security/             # Pentest, fuzzing, audit
+-- syncing/              # Sync protocol conformance tests
```

---

## 14. Disaster Recovery

### 14.1 Recovery Architecture

- **Primary DC**: Baghdad (active-active, 3-node Patroni)
- **DR DC**: Erbil (standby, logical replication)
- **Regional**: 18 hubs, each with streaming replica + local autonomy
- **Edge**: 1000+ nodes, fully independent operation

### 14.2 Recovery SLAs

| Scenario | RPO | RTO |
|---|---|---|
| Service crash | 0s | <30s |
| National DC network outage | <5min | <15min |
| National DC total failure | <1min | <30min |
| Regional hub failure | <5min | <15min |
| Edge node hardware failure | <24h | <4h |
| Data corruption (detected) | <5min | <1h |
| Data corruption (undetected) | <24h | <4h |
| Full region isolation | <24h | <48h |

### 14.3 DR Playbooks Structure

```
disaster-recovery/
+-- playbooks/
|   +-- failover/               # National DC, regional, edge, cross-region
|   +-- restore/                # PITR, full, partial
|   +-- rebuild/                # From event log, sync mesh
|   +-- data-loss/              # Mitigation, forensic analysis
+-- backup/                     # PostgreSQL, config, secrets, events
+-- restore/                    # Point-in-time, full, partial scripts
+-- replication/                # Streaming, logical, cross-region config
+-- testing/                    # Drills, automated tests, game days
```

---

## 15. Edge-Node Architecture

### 15.1 Edge Components

```
Edge Node (Single Server)
+-- Docker Host
    +-- PostgreSQL (local, reduced config)
    +-- Redis (cache, TTL-based)
    +-- Sync Engine (agent mode, merkle tree)
    +-- Attendance Service (offline-first)
    +-- Leave Service (offline-capable)
    +-- Device Gateway (local proxy, LAN)
    +-- Monitoring Agent (node exporter, vector)
```

### 15.2 Edge Specs by Size

| Resource | Large Site | Small Site | Mobile |
|---|---|---|---|
| CPU | 4 cores | 2 cores | ARM64 4 cores |
| RAM | 16 GB | 8 GB | 4 GB |
| Storage | 500 GB SSD | 250 GB SSD | 128 GB |
| Network | 100 Mbps LAN | 10 Mbps + 4G | 4G/5G |
| Employees | 500-2000 | 50-500 | 1-50 |
| Offline window | 30 days | 90 days | 7 days |

---

## 16. Regional Synchronization

### 16.1 Hub & Spoke with Mesh

- Regional hubs buffer events for disconnected edge nodes
- Hub-and-spoke to national DC (WAN)
- Mesh between peer regional hubs for redundancy
- LAN mesh between edge nodes in same site

### 16.2 Regional Recovery

1. Detection: 3 missed heartbeats -> alert
2. Isolation: Region operates autonomously
3. Reconnection: exponential backoff, mTLS re-established
4. Catch-up: merkle exchange -> delta transfer -> conflict resolution
5. Normalization: queues drained, heartbeat resumes

Duration: 1h offline = ~2min catch-up, 1 day = ~30min, 1 week = ~4h

### 16.3 Partition Tolerance

| Scenario | Behavior |
|---|---|
| National DC offline | Region operate independently |
| Regional hub offline | Edge nodes operate independently |
| Edge node offline | Fully autonomous |
| Network partition | CRDT merge on reconnection |
| Long-term (>30d) | Independent operation, manual review on merge |

---

## 17. Policy Engine Architecture

### 17.1 Components

```
Policy Engine
+-- Policy Definitions (OPA/Rego + JSON conditions)
+-- Policy Compiler (OPA engine, sandboxed)
+-- Evaluation Runtime (hot path: Redis cache, cold path: compiled)
+-- Admin CRUD (PostgreSQL, versioned policies)
+-- Policy Bundles (synced to edge nodes as config)
```

### 17.2 Policy Domains

```
policies/
+-- attendance/      shift-rules, overtime, grace-periods
+-- leave/           accrual, approval-chains, types
+-- security/        access-control, device-trust, encryption
+-- sync/            priority, bandwidth, scheduling
+-- compliance/      data-retention, audit, privacy
```

---

## 18. Device Integration Architecture

### 18.1 Device Classes

| Class | Examples | Protocol | Offline |
|---|---|---|---|
| Biometric terminals | ZKteco, Suprema | LAN/RS485 | Full |
| Attendance terminals | Card readers, PIN pads | LAN/USB | Full |
| Mobile devices | Smartphones, tablets | WiFi/4G | Full |
| IoT sensors | Badge readers | Zigbee/WiFi | Partial |

### 18.2 Protocol Adapters

```
device-gateway/internal/infrastructure/adapters/
+-- zkteco/          # ZKteco SDK wrapper (gRPC)
+-- suprema/         # Suprema face recognition
+-- hid/             # HID Global card readers
+-- rs485/           # Serial-to-Ethernet legacy adapter
+-- mqtt/            # IoT sensor class
+-- generic-http/    # Modern devices with REST APIs
```

### 18.3 Biometric Template Policy

- Templates NEVER synced across WAN
- Remain local to enrolling device/site
- Only template hashes synced (SHA-256 of template)
- Employee-to-template mappings synced (metadata only)

---

## 19. Audit Ledger Architecture

### 19.1 Ledger Data Model

```sql
CREATE TABLE ledger.entries (
    entry_id        UUID PRIMARY KEY,
    prev_hash       BYTEA NOT NULL,           -- SHA-256 of previous entry
    payload_hash    BYTEA NOT NULL,           -- SHA-256 of event payload
    entry_hash      BYTEA NOT NULL,           -- SHA-256(prev || payload || nonce)
    nonce           BYTEA NOT NULL,           -- 8 bytes ensuring uniqueness
    source_service  TEXT NOT NULL,
    source_node     UUID NOT NULL,
    event_type      TEXT NOT NULL,
    event_id        UUID NOT NULL,
    ministry_id     UUID,
    payload         JSONB NOT NULL,
    signature       BYTEA NOT NULL,           -- Ed25519 of entry_hash
    signing_key     TEXT NOT NULL,
    entry_position  BIGINT NOT NULL,          -- Monotonic position
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### 19.2 Seal Generation

Periodic seals (every 10000 entries or 1 hour):
- Compute merkle root of all entries in range
- Chain seals: `seal_hash = SHA-256(merkle_root || prev_seal_hash || timestamp)`
- Sign with dedicated seal key (Ed25519)
- Publish to public transparency endpoint

### 19.3 Verification

- Any auditor can verify chain integrity
- Verify: hash chain -> seal chain -> seal signature
- Automated verification runs daily
- Integrity failure triggers CATASPHERIC alert

---

## 20. Architecture Decision Records

All ADRs stored in `docs/architecture/decisions/`:

| ADR | Decision | Rationale |
|---|---|---|
| ADR-001 | PostgreSQL for all persistence | Mandated; no polyglot persistence |
| ADR-002 | Go as primary service language | Single binary, fast compilation, ARM64 support |
| ADR-003 | Rust for sync engine | Memory safety for correctness-critical sync code |
| ADR-004 | Custom sync protocol | Requirements-specific (offline-first, LAN mesh, air-gap) |
| ADR-005 | NATS for national event bus | Lightweight, at-least-once, cluster mode |
| ADR-006 | pg_notify for edge event bus | Zero additional dependencies, works offline |
| ADR-007 | Event sourcing + append-only audit | Immutable audit trail requirement |
| ADR-008 | mTLS for all service-to-service | Zero-trust mandate |
| ADR-009 | OAuth2 + OIDC for identity | Industry standard, offline-capable with local sessions |
| ADR-010 | Per-ministry schema isolation | Multi-tenant requirement without shared schema complexity |
| ADR-011 | CRDT/LWW conflict resolution | Eventual consistency without consensus overhead |
| ADR-012 | 3-tier deployment (edge/region/national) | Government-grade reliability, regional autonomy |
| ADR-013 | Merkle tree anti-entropy sync | Efficient reconciliation over low-bandwidth links |
| ADR-014 | Ed25519 for event signing | Fast, secure, small signatures for resource-constrained devices |
| ADR-015 | MinIO for object storage | Self-hosted S3-compatible, no cloud dependency |
| ADR-016 | OPA/Rego for policy engine | Policy-as-code, versioned, testable, sandboxed |
| ADR-017 | HashiCorp Vault for secrets management | Industry standard, HSM integration, audit logging |
| ADR-018 | UUID v7 for all primary keys | Time-ordered, index-friendly, no collision risk |

---

## 21. Contracts Structure

### 21.1 Protobuf/gRPC Contracts

```
events/contracts/protobuf/
+-- inwp/
|   +-- common/
|   |   +-- types.proto          # Shared types (UUID, Timestamp, Money, etc.)
|   |   +-- sync.proto           # Sync metadata types
|   |   +-- security.proto       # Security context types
|   +-- identity/
|   |   +-- v1/
|   |       +-- service.proto    # IdentityService RPC definitions
|   |       +-- messages.proto   # Request/response messages
|   +-- authentication/
|   |   +-- v1/
|   +-- authorization/
|   |   +-- v1/
|   +-- attendance/
|   |   +-- v1/
|   +-- leave/
|   |   +-- v1/
|   +-- workforce/
|   |   +-- v1/
|   +-- sync/
|   |   +-- v1/
|   +-- audit/
|   |   +-- v1/
|   +-- notification/
|   |   +-- v1/
|   +-- policy/
|   |   +-- v1/
|   +-- analytics/
|   |   +-- v1/
|   +-- device/
|       +-- v1/
+-- google/
    +-- type/                    # Well-known Google types
```

### 21.2 OpenAPI Contracts

```
docs/api/openapi/
+-- v1/
|   +-- identity.yaml            # Identity/REST API (OAuth2/OIDC)
|   +-- attendance.yaml          # Attendance REST API
|   +-- leave.yaml               # Leave REST API
|   +-- workforce.yaml           # Workforce state REST API
|   +-- notification.yaml        # Notification admin REST API
|   +-- policy.yaml              # Policy admin REST API
|   +-- analytics.yaml           # Analytics REST API
|   +-- device.yaml              # Device management REST API
|   +-- admin.yaml               # System admin REST API
+-- common/
    +-- schemas.yaml             # Shared schemas (pagination, errors, etc.)
```

### 21.3 AsyncAPI Contracts

```
events/contracts/asyncapi/
+-- v1/
|   +-- attendance-events.yaml
|   +-- leave-events.yaml
|   +-- identity-events.yaml
|   +-- sync-events.yaml
|   +-- audit-events.yaml
|   +-- notification-events.yaml
|   +-- policy-events.yaml
|   +-- device-events.yaml
|   +-- analytics-events.yaml
|   +-- security-events.yaml
|   +-- system-events.yaml
```

---

## 22. Frontend Architecture

### 22.1 Web Portal (React/Next.js)

```
frontend/web-portal/
+-- src/
|   +-- components/              # Reusable UI components
|   |   +-- common/              # Button, Input, Modal, Table, Card, Form, etc.
|   |   +-- layout/              # AppShell, Sidebar, Topbar, Navigation
|   |   +-- attendance/          # ClockEventList, ShiftManager, ExceptionCard
|   |   +-- leave/               # LeaveRequestForm, ApprovalChain, BalanceCard
|   |   +-- identity/            # UserTable, RoleManager, DeviceList
|   |   +-- workforce/           # StateExplorer, TimesheetView
|   |   +-- analytics/           # ChartWidget, DashboardGrid, MetricCard
|   |   +-- admin/               # SystemConfig, MinistryManager
|   |   +-- reports/             # ReportBuilder, ExportButton
|   |   +-- settings/            # MinistrySettings, SiteSettings
|   |   +-- sync/                # SyncStatusPanel, ConflictResolver
|   |   +-- audit/               # AuditTrailView, SealVerification
|   +-- pages/                   # Route-based pages
|   +-- hooks/                   # useAuth, useSync, useOffline, etc.
|   +-- services/                # API client, gRPC-web client, offline cache
|   +-- stores/                  # Zustand/Pinia stores
|   +-- utils/                   # Date, format, validation utilities
|   +-- types/                   # TypeScript type definitions
|   +-- styles/                  # Tailwind/SCSS global styles
|   +-- assets/                  # Icons, images, fonts
|   +-- locales/                 # ar.json, ku.json, en.json
+-- public/                      # Static files
+-- configs/                     # Environment-specific config
+-- tests/                       # Vitest, Playwright
+-- deploy/                      # Dockerfile, nginx.conf
```

### 22.2 Mobile App (React Native)

```
frontend/mobile-app/
+-- src/
|   +-- components/              # Reusable mobile components
|   +-- screens/
|   |   +-- attendance/          # ClockInScreen, HistoryScreen
|   |   +-- leave/               # RequestScreen, StatusScreen
|   |   +-- profile/             # ProfileScreen, EditProfileScreen
|   |   +-- approvals/           # ApprovalsList, ApprovalDetail
|   |   +-- dashboard/           # EmployeeDashboard
|   |   +-- settings/            # AppSettings, LanguageSelect
|   |   +-- sync/                # SyncStatusScreen, ConflictScreen
|   +-- hooks/                   # Custom hooks
|   +-- services/                # API client, offline queue
|   +-- stores/                  # State management (offline-first)
|   +-- utils/                   # Mobile utilities
|   +-- types/                   # TypeScript types
|   +-- assets/                  # Images, fonts
|   +-- locales/                 # ar.json, ku.json, en.json
+-- android/                     # Android config
+-- ios/                         # iOS config
+-- tests/                       # Jest, Detox
+-- deploy/                      # Build config
```

### 22.3 Offline-First Frontend Strategy

- Service Worker caches API responses (Cache-first for reads)
- IndexedDB for offline data store (syncs with PostgreSQL via sync engine)
- Background sync for queued mutations
- Optimistic UI updates with rollback on conflict
- Connectivity-aware rendering (online/offline indicators)
- Language auto-detection (Arabic RTL, Kurdish, English)

---

## 23. Gateway Architecture

### 23.1 API Gateway

```
gateway/api-gateway/
+-- config/
|   +-- routes.yaml              # Route definitions per service
|   +-- rate-limits.yaml         # Rate limit configuration per route/ministry
|   +-- cors.yaml                # CORS policies per ministry domain
+-- plugins/
|   +-- auth.lua                 # JWT validation plugin
|   +-- audit.lua                # Request audit logging
|   +-- rate-limit.lua           # Rate limiting
|   +-- transform.lua            # Request/response transformation
+-- routes/
    +-- identity.yaml            # Identity service routes
    +-- attendance.yaml          # Attendance service routes
    +-- leave.yaml               # Leave service routes
    +-- sync.yaml                # Sync endpoint routes
```

### 23.2 Device Gateway

```
gateway/device-gateway/
+-- config/
|   +-- devices.yaml             # Device registry configuration
|   +-- protocols.yaml           # Protocol adapter configuration
|   +-- trust.yaml               # Trust threshold configuration
+-- protocols/
    +-- zkteco.yaml              # ZKteco SDK configuration
    +-- suprema.yaml             # Suprema SDK configuration
    +-- mqtt.yaml                # MQTT topic configuration
```

---

## 24. Identity Architecture

### 24.1 Realm Hierarchy

```
/ministries/{ministry_id}
  /sites/{site_id}
    /roles: site_admin, hr_operator, attendance_operator, leave_approver, viewer
  /roles: ministry_admin, ministry_hr, ministry_auditor, ministry_viewer
  /groups: payroll_team, inspection_team
/national
  /roles: national_admin, national_auditor, system_operator
```

### 24.2 Identity Realms Configuration

```
identity/realms/
+-- mohe/                        # Ministry of Education
|   +-- realm.json               # Realm configuration
|   +-- policies.json            # Realm-specific auth policies
+-- moh/                         # Ministry of Health
+-- moo/                         # Ministry of Oil
+-- mop/                         # Ministry of Planning
+-- moel/                        # Ministry of Electricity
+-- moa/                         # Ministry of Agriculture
+-- other/                       # Other ministries
+-- federation/
|   +-- saml/                    # SAML identity provider config
|   +-- ldap/                    # LDAP directory integration
|   +-- scim/                    # SCIM provisioning config
+-- policies/
    +-- rbac/                    # Role definitions
    +-- abac/                    # Attribute definitions
    +-- password/                # Password policy per ministry
```

### 24.3 Authentication Methods

| Method | MFA | Offline | Use Case |
|---|---|---|---|
| Password + TOTP | Yes | No | Ministry admins, web portal |
| Smart card (PKCS#11) | Inherent | Yes | National admins, auditors |
| Biometric (fingerprint) | Yes | Yes | Field workers |
| Biometric (face) | Yes | Yes | Mobile workers |
| Certificate (device) | Inherent | Yes | IoT devices, terminals |
| SMS OTP | Second factor | No | Password reset, recovery |

---

## 25. Multi-Tenancy Architecture

### 25.1 Tenant Model

Each ministry is a separate tenant with:

- **Database isolation**: per-service schemas with `ministry_id` on every row
- **RLS enforcement**: Row-Level Security on all tables filtering by `ministry_id`
- **Encryption isolation**: per-tenant envelope encryption keys (HSM-backed)
- **Identity isolation**: separate OIDC realms per ministry
- **Network isolation**: optional separate virtual networks for sensitive ministries
- **Policy isolation**: per-ministry policy overrides on national baseline

### 25.2 Cross-Tenant Boundaries

- Cross-ministry data access requires `national_admin` or `national_auditor` role
- Ministry-to-ministry data transfer requires audit trail
- National-level reporting uses aggregated views (no raw cross-ministry data)
- Sync engine partitions by ministry -- no cross-contamination
- Audit ledger entries tagged with ministry_id for tenant-scoped queries

---

## Appendix A: Technology Stack Summary

| Category | Technology |
|---|---|
| Core backend | Go 1.22+ |
| Sync engine | Rust |
| Database | PostgreSQL 16 (PostGIS, pg_partman, pgBackRest) |
| Event bus | NATS JetStream (national), pg_notify (edge) |
| Cache | Redis 7 (cluster national, standalone edge) |
| Object storage | MinIO (distributed national, gateway regional) |
| Service mesh | Direct mTLS (no mesh in MVP) |
| API gateway | Kong / Traefik |
| Identity | Custom OAuth2/OIDC + Keycloak bridge |
| Secrets | HashiCorp Vault (national), Docker secrets (edge) |
| Policy | OPA/Rego |
| Monitoring | Prometheus + Grafana |
| Logging | Vector -> Loki |
| Tracing | OpenTelemetry -> Tempo |
| Alerting | Alertmanager |
| Container | Docker, Kubernetes (national only) |
| CI/CD | GitHub Actions |

## Appendix B: Port Mapping

| Service | Port | Protocol |
|---|---|---|
| identity-service (gRPC) | 50051 | gRPC |
| identity-service (REST) | 8081 | HTTPS |
| authentication-service | 50058 | gRPC |
| authorization-service | 50059 | gRPC |
| sync-engine (sync) | 50052 | HTTPS/WS |
| sync-engine (mesh) | 50053 | gRPC |
| audit-ledger | 50054 | gRPC |
| attendance-service | 50055 | gRPC |
| leave-service | 50056 | gRPC |
| notification-service | 50057 | gRPC |
| policy-engine | 50060 | gRPC |
| analytics-service | 8084 | HTTPS |
| device-gateway | 50061 | gRPC |
| PostgreSQL | 5432 | TCP |
| Redis | 6379 | TCP |
| NATS | 4222 | TCP |
| Prometheus metrics | 9090 | HTTP |
| Health/readiness | 8080 | HTTP |

## Appendix C: Key ADR Summaries

See `docs/architecture/decisions/` for full ADR documents. Key decisions referenced throughout this document are summarized in section 20.
