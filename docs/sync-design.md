# Sync Design

See [architecture/overview.md §4 Sync Architecture](../architecture/overview.md#4-sync-architecture) for the complete synchronization design.

Key sections:
- Design principles (no single source of truth, anti-entropy, P2P LAN, hub-and-spoke WAN)
- Sync topology (Edge → Regional Relay → National Hub)
- Sync protocol phases (Discovery → Merkle Exchange → Delta Transfer → Reconciliation → Commitment)
- Conflict resolution matrix
- Bandwidth management (delta compression, priority queuing, budgeting)
- Offline operation model
- Sync data model (`sync_checkpoint`, `sync_batch_log` tables)
