# INWP Deployment Topology

> Production-grade deployment architecture for all tiers: ministry, regional, institution, edge, and disconnected.

---

## 1. Ministry Level (National DC)

```
Location: Baghdad Data Center
Orchestration: Kubernetes (HA control plane, 3+ worker nodes)
Tier: Mission-critical, 99.99% availability

Services (3 replicas each, anti-affinity):
+---------------------+---------------------+---------------------+
| identity-service    | authentication-svc  | authorization-svc   |
| (3 pods)            | (3 pods)            | (3 pods)            |
+---------------------+---------------------+---------------------+
| attendance-service  | leave-service       | workforce-state     |
| (3 pods)            | (2 pods)            | (2 pods)            |
+---------------------+---------------------+---------------------+
| sync-engine (hub)   | audit-ledger        | notification-svc    |
| (3 pods)            | (3 pods)            | (2 pods)            |
+---------------------+---------------------+---------------------+
| policy-engine       | analytics-service   | device-gateway      |
| (3 pods)            | (2 pods)            | (2 pods)            |
+---------------------+---------------------+---------------------+
| web-portal          | api-gateway         |                     |
| (2 pods)            | (2 pods)            |                     |
+---------------------+---------------------+---------------------+

Infrastructure (StatefulSets):
+---------------------+---------------------+---------------------+
| Patroni PostgreSQL  | Redis Cluster       | NATS JetStream      |
| (3 pods + 2 replicas)| (3 masters + 3 reps)| (3 pods, R3)        |
+---------------------+---------------------+---------------------+
| MinIO (distributed) | Vault (HA)          | Keycloak            |
| (4 pods, erasure)   | (3 pods)            | (2 pods)            |
+---------------------+---------------------+---------------------+

Monitoring:
+---------------------+---------------------+---------------------+
| Prometheus (HA)     | Grafana             | Loki + Tempo        |
| (2 pods + Thanos)   | (2 pods)            | (3 pods each)       |
+---------------------+---------------------+---------------------+
| Alertmanager (HA)   | Vector (DaemonSet)  |                     |
| (3 pods)            |                    |                     |
+---------------------+---------------------+---------------------+

Network:
- 10 Gbps redundant uplinks
- mTLS encrypted tunnels to regional hubs
- IPsec VPN to other government networks
- WAF + DDoS protection at perimeter

Storage:
- PostgreSQL: 4TB NVMe RAID10 (primary), 8TB SSD (replicas)
- MinIO: 16TB distributed (4 nodes x 4TB)
- Prometheus: 1TB SSD (30d retention)

Disaster Recovery:
- Synchronous replication to DR DC (Erbil)
- RPO: 0-5s, RTO: <30min
- Automated failover via Patroni + keepalived
```

## 2. Regional Level (Governorate Hub)

```
Location: 18 Governorate data centers
Orchestration: Docker Compose (2-node active/passive)
Tier: High-availability, 99.9% availability

Services (2 replicas each):
+---------------------+---------------------+
| sync-engine (relay) | attendance-service  |
| (2 containers)      | (2 containers)      |
+---------------------+---------------------+
| leave-service       | device-gateway      |
| (2 containers)      | (2 containers)      |
+---------------------+---------------------+
| identity-cache      | notification-cache  |
| (1 container)       | (1 container)       |
+---------------------+---------------------+

Infrastructure:
+---------------------+---------------------+
| PostgreSQL (stream) | Redis (standalone)  |
| (1 container)       | (1 container)       |
+---------------------+---------------------+
| NATS Leaf Node      | MinIO Gateway       |
| (1 container)       | (1 container)       |
+---------------------+---------------------+

Monitoring:
+---------------------+---------------------+
| Prometheus (local)  | Vector (log shipper)|
| (1 container)       | (Daemon)            |
+---------------------+---------------------+

Network:
- 1 Gbps WAN to National DC
- 100 Mbps+ WAN to edge nodes in region
- mTLS encrypted tunnels to all peers
- 4G/LTE backup link for critical services

Storage:
- PostgreSQL: 1TB SSD (streaming replica)
- Local cache: 256GB SSD

Regional Administration:
- Local admin console for offline operations
- Conflict resolution dashboard
- Sync status monitoring

Failover:
- Active/passive with keepalived VIP
- Auto-failover on health check failure (3s interval)
- Data replication via PostgreSQL streaming
```

## 3. Institution Level (Edge Node)

```
Location: Individual institution (university, hospital, ministry building)
Orchestration: Docker Compose (single host)
Tier: Standard availability, 99.5% uptime target

Services (1 instance each):
+---------------------+---------------------+
| sync-engine (agent) | attendance-service  |
| (1 container)       | (1 container)       |
+---------------------+---------------------+
| leave-service       | device-gateway      |
| (1 container)       | (1 container)       |
+---------------------+---------------------+

Infrastructure:
+---------------------+---------------------+
| PostgreSQL (local)  | Redis (standalone)  |
| (1 container)       | (1 container)       |
+---------------------+---------------------+
| pg_notify (eventbus)|                     |

Monitoring:
+---------------------+
| Node Exporter       |
| (1 container)       |
+---------------------+

Network:
- 100 Mbps LAN (internal device network)
- 10 Mbps+ WAN to regional hub (shared/4G)
- mDNS for peer discovery (LAN mesh)
- mTLS to regional hub, no inbound WAN

Storage:
- PostgreSQL: 500GB SSD (large site), 250GB SSD (small site)
- Event queue: unlimited (disk-based)

Connected Devices (LAN):
- Up to 20 biometric terminals (ZKteco, Suprema)
- Card readers, PIN pads
- Admin workstations
- Optional: local printer for reports

Large Site Specs: 4 CPU, 16GB RAM, 500GB SSD, 500-2000 employees
Small Site Specs: 2 CPU, 8GB RAM, 250GB SSD, 50-500 employees
```

## 4. Disconnected Operation Mode

```
Trigger: WAN connectivity lost >30 seconds

IMMEDIATE EFFECT:
+----------------------------------------------------+
| All services continue running locally              |
| PostgreSQL is local source of truth                |
| Event queue accumulates outbound events            |
| Heartbeat stops (no outbound pings)                |
+----------------------------------------------------+

OPERATIONAL CAPABILITIES:
+----------------------------------------------------+
| [YES] Clock-in/out with biometric verification     |
| [YES] Leave requests and approvals                 |
| [YES] Shift schedule management                    |
| [YES] Local employee directory (cached)            |
| [YES] Local admin dashboard                        |
| [NO]  Cross-site data access                       |
| [NO]  National reports or analytics                |
| [NO]  SMS/push/email notifications                 |
| [NO]  Identity changes from other sites            |
+----------------------------------------------------+

QUEUE MANAGEMENT:
- Event queue: append-only, no limit (disk-based)
- Queue depth monitor: admin visible on dashboard
- Auto-rotation: oldest events never purged until confirmed synced
- Compression: events rotated to compressed archive after 7 days offline

RECONNECTION SEQUENCE:
Phase 1: Detection (connectivity restored)
  - Heartbeat sent successfully -> sync triggered
  - Manual: admin triggers sync button

Phase 2: Authentication
  - mTLS handshake with regional hub
  - Certificate status verified
  - Schema version negotiated

Phase 3: Discovery
  - Exchange last checkpoint per partition
  - Compute diffs via merkle tree

Phase 4: Data Transfer
  - Upload local changes (priority: attendance first)
  - Download remote changes (priority: identity first)
  - Bandwidth throttled to configured cap

Phase 5: Reconciliation
  - Auto-resolve conflicts (LWW, CRDT)
  - Escalate unresolvable to admin queue
  - Verify data integrity (merkle roots)

Phase 6: Normalization
  - Sync queues drained
  - Event_outbox marked as synced
  - Heartbeat resumes normal interval
  - Admin notified of sync completion

RECOVERY TIMING (est.):
  1 hour offline:  ~2 minutes catch-up
  1 day offline:   ~30 minutes catch-up
  1 week offline:  ~4 hours catch-up
  30 days offline: ~1 day catch-up (admin review required)
```

## 5. Mobile Deployment

```
Workflow: Field employee mobile app (BYOD or issued device)

Architecture:
- React Native app with offline-first data layer
- Local SQLite for offline data store
- Background sync engine (periodic + on-connect)
- Certificate-pinned HTTPS to edge node or regional hub

Connectivity:
- WiFi (on-site) -> sync to edge node
- 4G/5G (remote) -> sync to regional hub
- Offline queue -> sync on next connectivity

Capabilities:
- Clock-in/out with GPS + biometric (on-device face/fingerprint)
- Leave requests and status tracking
- Employee profile and schedule view
- Approvals (manager role)
- Notifications (push, in-app)
- Offline: all data cached, actions queued
```

## 6. Container Image Strategy

```
Multi-stage Build Pattern (all services):
  Stage 1 (builder):   golang:1.22-alpine / rust:1.78-slim
  Stage 2 (runtime):   gcr.io/distroless/base (Go) / alpine:3.19 (Rust)

Base Images (docker/base-images/):
  golang/    - Go builder base + CA certs + tzdata + build tools
  rust/      - Rust builder base + musl target
  node/      - Node.js builder base (frontend)
  alpine/    - Minimal runtime base

Image Hardening:
  - Non-root user: USER 1001:1001
  - Read-only root filesystem
  - Dropped capabilities (no NET_RAW, no SYS_ADMIN)
  - Immutable tags: {service}-{git-sha}-{timestamp}
  - Distroless base: no shell, no package manager, no suid

Security:
  - Trivy vulnerability scan in CI (fail on CRITICAL/HIGH)
  - Cosign image signing
  - SBOM generation (Syft) embedded as attestation
  - Image pull policy: Always (enforce fresh)

Registry:
  - Harbor per ministry (private)
  - Image replication to regional hubs (for edge pulls)
  - Automatic garbage collection (untagged images >30d)
```
