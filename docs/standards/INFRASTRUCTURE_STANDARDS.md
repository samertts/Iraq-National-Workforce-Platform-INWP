# INWP Sovereign Infrastructure Standards

## Federation Topology

Sovereign (Baghdad DC)
  ├── Ministry of Labor (mTLS mesh)
  │   ├── Regional Hub — Baghdad
  │   │   ├── Institution 1..N
  │   │   └── Edge Nodes
  │   ├── Regional Hub — Basra
  │   │   ├── Institution 1..N
  │   │   └── Edge Nodes
  │   └── Regional Hub — Erbil
  │       ├── Institution 1..N
  │       └── Edge Nodes
  ├── Ministry of Education (sovereign boundary)
  └── Ministry of Health (sovereign boundary)

## Network Architecture

- National Hub: Full mesh with all ministries, PostgreSQL + NATS JetStream
- Regional Hub: PostgreSQL only, relay to national hub, NATS optional
- Edge Node: SQLite or local PostgreSQL, mDNS discovery, autonomous operation
- Sovereign Boundary: Air-gapped or cryptographically isolated per sovereignty zone

## Deployment Architecture

- Docker for all components
- docker-compose for regional/edge deployments
- Kubernetes (national hub only — K8s adds unacceptable complexity for edge)
- Multi-stage Docker builds for minimal image size
- Distroless base images for production

## Storage Architecture

- PostgreSQL for all persistent state
- NATS JetStream for event streaming (national hub only)
- Local storage for edge autonomy during disconnection
- Cryptographic event chains for immutable audit trails
- All storage encrypted at rest via filesystem-level encryption

## Security Architecture

- mTLS between all gRPC endpoints
- Ed25519 for event and artifact signing
- Rustls for TLS termination
- Trust scoring per node (0.0–1.0)
- Automatic isolation for trust score < 0.3
- HSM integration for sovereign key material

## Observability Architecture

- OpenTelemetry for distributed tracing
- Prometheus metrics exposition
- Structured JSON logging
- Health check endpoints (/health, /ready)
- Metrics: Merkle trees, sync operations, conflicts, queue depth, recovery operations
