# Phase 1: Attendance Service

> Iraq National Workforce Platform — Phase 1 implementation of the offline-first attendance tracking service.

---

## Scope

Phase 1 delivers a production-ready attendance service covering:

| Module | Status | Description |
|--------|--------|-------------|
| Clock Events | Implemented | Offline-first clock-in/out, break start/end, manual corrections |
| Shift Management | Implemented | Site-level shift definitions with grace periods and overtime policies |
| Attendance Policies | Implemented | Configurable rules engine for site/ministry attendance enforcement |
| Attendance Exceptions | Implemented | Auto-detection of irregularities with justification workflow |
| Event Publishing | Implemented | CloudEvents 1.0 compliant event emission via NATS + pg_notify |
| Offline Queue | Implemented | Local event store with sync metadata for offline-first operation |
| REST API | Implemented | HTTP API for mobile/web clients |
| gRPC API | Implemented | Internal service-to-service communication |
| PostgreSQL Migrations | Implemented | Partitioned tables, RLS policies, indexes |
| Docker | Implemented | Multi-stage Dockerfile for production deployment |

---

## Architecture

### Layers (Clean Architecture / DDD)

```
interfaces/       ← HTTP handlers, gRPC services, CLI
    ↓
application/      ← Use cases / commands
    ↓
domain/           ← Aggregates, entities, value objects, domain services
    ↓
infrastructure/   ← PostgreSQL repos, NATS publisher, sync queue
```

### Key Design Decisions

1. **Append-only ClockEvent** — never modified after creation; corrections create new events with references
2. **Policy-as-config** — attendance rules are database-configurable, not hardcoded
3. **Offline-first** — every write stores to local PG first, then publishes event asynchronously
4. **Sync metadata** — every aggregate carries `SyncMetadata` for merkle-tree reconciliation

### Event Flow

```
Client → POST /api/v1/clock-in
         → application.ClockInHandler
           → domain.ClockInService.Record()
             → domain.ClockEvent aggregate created
             → domain events raised
           → infrastructure.ClockEventRepository.Save()
             → INSERT INTO attendance.clock_events
           → infrastructure.EventPublisher.Publish()
             → NATS: inwp.attendance.v1.clock-in.created
             → pg_notify: attendance_events channel
           → infrastructure.SyncQueue.Enqueue()
             → INSERT INTO sync.outbox
```

### Conflict Resolution Strategy

| Entity | Strategy |
|--------|----------|
| ClockEvent | Last-writer-wins (append-only, no conflict) |
| Shift | Last-writer-wins |
| AttendancePolicy | Ministry-authoritative |
| AttendanceException | Last-writer-wins |

---

## API Endpoints

### REST API

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/attendance/clock-in` | Record clock-in |
| POST | `/api/v1/attendance/clock-out` | Record clock-out |
| POST | `/api/v1/attendance/break/start` | Start break |
| POST | `/api/v1/attendance/break/end` | End break |
| POST | `/api/v1/attendance/correct` | Manual correction |
| POST | `/api/v1/attendance/dispute` | Dispute event |
| GET | `/api/v1/attendance/events` | Query clock events |
| GET | `/api/v1/attendance/events/:id` | Get event by ID |
| GET | `/api/v1/attendance/summary` | Get attendance summary |
| POST | `/api/v1/shifts` | Create shift |
| GET | `/api/v1/shifts` | List shifts |
| POST | `/api/v1/policies` | Set attendance policy |
| GET | `/api/v1/policies` | List policies |
| POST | `/api/v1/exceptions/justify` | Justify exception |
| POST | `/api/v1/exceptions/resolve` | Resolve exception |

### gRPC Services

| Service | Method | Description |
|---------|--------|-------------|
| `AttendanceService` | `ClockIn` | Record clock-in |
| `AttendanceService` | `ClockOut` | Record clock-out |
| `AttendanceService` | `GetEvents` | Query events |
| `AttendanceService` | `GetSummary` | Get summary |
| `ShiftService` | `CreateShift` | Create shift |
| `ShiftService` | `ListShifts` | List shifts |
| `PolicyService` | `SetPolicy` | Set policy |
| `PolicyService` | `GetActivePolicy` | Get active policy |

---

## Database Schema

### Schema: `attendance`

| Table | Type | Description |
|-------|------|-------------|
| `clock_events` | Partitioned (by month) | Immutable clock-in/out records |
| `shifts` | Standard | Shift definitions |
| `attendance_policies` | Standard | Policy configurations (versioned) |
| `attendance_exceptions` | Standard | Detected irregularities |
| `exception_justifications` | Standard | Employee justifications |

### Partitioning

```sql
CREATE TABLE attendance.clock_events (
    id UUID NOT NULL,
    employee_id UUID NOT NULL,
    ministry_id UUID NOT NULL,
    site_id UUID NOT NULL,
    device_id UUID NOT NULL,
    event_type attendance.clock_event_type NOT NULL,
    event_time TIMESTAMPTZ NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    biometric_match DOUBLE PRECISION,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    ip_address INET,
    sync_id UUID,
    source_node_id UUID,
    sync_status attendance.sync_status DEFAULT 'local_only',
    version BIGINT DEFAULT 1,
    PRIMARY KEY (id, recorded_at)
) PARTITION BY RANGE (recorded_at);
```

### RLS Policies

Row-level security enforced per ministry:

```sql
ALTER TABLE attendance.clock_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY ministry_isolation ON attendance.clock_events
    USING (ministry_id = current_setting('app.current_ministry_id')::UUID);
```

---

## Security

- All endpoints require JWT with ministry-scoped claims
- mTLS between services
- Ed25519-signed events
- AES-256-GCM encrypted payloads at rest
- Input validation on all commands
- Rate limiting per device/site
- Audit trail for all administrative actions

---

## Offline-First Strategy

1. **Local PostgreSQL** — every edge node runs PostgreSQL with full schema
2. **Write-local** — all writes go to local PG first, regardless of connectivity
3. **Sync outbox** — written events are queued in `sync.outbox` table
4. **Anti-entropy** — sync engine uses merkle trees to reconcile
5. **Conflict resolution** — per-entity strategy (see above)
6. **Idempotency** — every event has a unique ID for deduplication

---

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `github.com/google/uuid` | UUID v7 generation |
| `github.com/jackc/pgx/v5` | PostgreSQL driver |
| `github.com/nats-io/nats.go` | NATS client |
| `github.com/julienschmidt/httprouter` | HTTP router |
| `google.golang.org/grpc` | gRPC framework |
| `github.com/rs/zerolog` | Structured logging |
| `github.com/prometheus/client_golang` | Metrics |
