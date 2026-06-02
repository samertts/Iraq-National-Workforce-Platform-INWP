DROP TRIGGER IF EXISTS trg_clock_event_notify ON attendance.clock_events;
DROP FUNCTION IF EXISTS attendance.notify_clock_event();

DROP TABLE IF EXISTS sync.outbox CASCADE;
DROP TABLE IF EXISTS attendance.attendance_exceptions CASCADE;
DROP TABLE IF EXISTS attendance.attendance_policies CASCADE;
DROP TABLE IF EXISTS attendance.shifts CASCADE;
DROP TABLE IF EXISTS attendance.clock_events CASCADE;

DROP TYPE IF EXISTS attendance.justification_type;
DROP TYPE IF EXISTS attendance.exception_severity;
DROP TYPE IF EXISTS attendance.exception_type;
DROP TYPE IF EXISTS attendance.sync_status;
DROP TYPE IF EXISTS attendance.clock_event_type;

DROP SCHEMA IF EXISTS sync CASCADE;
DROP SCHEMA IF EXISTS attendance CASCADE;
