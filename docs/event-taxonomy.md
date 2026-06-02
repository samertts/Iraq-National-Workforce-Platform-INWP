# Event Taxonomy

See [architecture/overview.md §3 Event Model](../architecture/overview.md#3-event-model) for the complete event taxonomy, schema standard, lifecycle, versioning strategy, and storage model.

Key sections:
- CloudEvents 1.0 standard with mandatory extensions
- Full event type hierarchy (`inwp.attendance.v1.*`, `inwp.leave.v1.*`, etc.)
- Event lifecycle (produce → validate → sign → persist → publish → route → consume)
- Schema registry and versioning
- `event_outbox` table definition
