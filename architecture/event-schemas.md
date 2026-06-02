# INWP Event Schema Registry

> Complete event schemas for all core domains — CloudEvents 1.0 compliant with JSON Schema validation.

---

## Schema Conventions

### Envelope (all events)

```json
{
  "specversion": "1.0",
  "id": "01J8X2Y3Z4A5B6C7D8E9F0G1H2",
  "source": "/ministries/{ministry_id}/sites/{site_id}/services/{service_name}",
  "type": "inwp.{domain}.v1.{entity}.{action}",
  "datacontenttype": "application/json",
  "subject": "{entity_type}:{entity_id}",
  "time": "2026-05-31T10:00:00.000Z",
  "dataschema": "inwp:{domain}:{entity}:{action}:v1",
  "ministry_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "site_id": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
  "device_id": "c3d4e5f6-a7b8-9012-cdef-123456789012",
  "user_id": "d4e5f6a7-b8c9-0123-defa-234567890123",
  "offline_generated": true,
  "local_timestamp": "2026-05-31T13:00:00.000+03:00",
  "sync_id": "e5f6a7b8-c9d0-1234-efab-345678901234"
}
```

### Common Extension Attributes

| Attribute | Type | Required | Description |
|---|---|---|---|
| `ministry_id` | `string` (UUID) | Yes | Tenant scope |
| `site_id` | `string` (UUID) | No | Site scope |
| `device_id` | `string` (UUID) | No | Originating device |
| `user_id` | `string` (UUID) | No | Acting user |
| `offline_generated` | `boolean` | No | `true` if created while offline |
| `local_timestamp` | `string` (RFC3339) | No | Device-local time when event was created |
| `sync_id` | `string` (UUID) | No | Sync batch that delivered this event |
| `trace_id` | `string` (UUID) | No | Distributed tracing correlation |

### Schema Naming

```
inwp:{domain}:{entity}:{action}:v{major}

Examples:
  inwp:attendance:clock-in:created:v1
  inwp:leave:request:approved:v1
  inwp:identity:user:registered:v1
```

### Validation Rules (applied to all schemas)

- All UUIDs are v7 (time-ordered)
- All timestamps are RFC 3339 with optional fractional seconds
- Monetary/balance values use `float64` with 2 decimal precision
- Strings are UTF-8, max 2048 characters unless otherwise specified
- Arrays max 1000 items unless otherwise specified

---

## 1. Attendance Events

### `inwp:attendance:clock-in:created:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:attendance:clock-in:created:v1",
  "title": "Clock-In Created",
  "description": "Emitted when an employee clocks in at a device",
  "type": "object",
  "required": [
    "event_id", "employee_id", "ministry_id", "site_id",
    "device_id", "event_type", "event_time", "recorded_at"
  ],
  "properties": {
    "event_id": {
      "type": "string",
      "format": "uuid",
      "description": "Unique identifier for this clock event"
    },
    "employee_id": {
      "type": "string",
      "format": "uuid",
      "description": "The employee who clocked in"
    },
    "ministry_id": {
      "type": "string",
      "format": "uuid",
      "description": "Ministry the employee belongs to"
    },
    "site_id": {
      "type": "string",
      "format": "uuid",
      "description": "Site where clock-in occurred"
    },
    "device_id": {
      "type": "string",
      "format": "uuid",
      "description": "Device that recorded the clock-in"
    },
    "event_type": {
      "type": "string",
      "enum": ["clock_in", "clock_out", "break_start", "break_end", "manual_correction"],
      "description": "Must be 'clock_in' for this event type"
    },
    "event_time": {
      "type": "string",
      "format": "date-time",
      "description": "Time when the clock-in occurred (device local time)"
    },
    "recorded_at": {
      "type": "string",
      "format": "date-time",
      "description": "Time when the server received the event"
    },
    "timezone": {
      "type": "string",
      "pattern": "^[A-Za-z]+/[A-Za-z_]+$",
      "description": "IANA timezone of the site",
      "default": "Asia/Baghdad"
    },
    "biometric": {
      "type": "object",
      "description": "Biometric verification result (if applicable)",
      "properties": {
        "matched": { "type": "boolean" },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
        "template_hash": { "type": "string", "pattern": "^[a-f0-9]{64}$" }
      },
      "required": ["matched", "confidence"]
    },
    "location": {
      "type": "object",
      "description": "GPS coordinates (if mobile clock-in)",
      "properties": {
        "latitude": { "type": "number", "minimum": -90, "maximum": 90 },
        "longitude": { "type": "number", "minimum": -180, "maximum": 180 },
        "accuracy": { "type": "number", "minimum": 0 }
      }
    },
    "device_time": {
      "type": "string",
      "format": "date-time",
      "description": "Raw device timestamp before any sync adjustment"
    }
  }
}
```

**Example:**

```json
{
  "specversion": "1.0",
  "id": "01J8X2Y3Z4A5B6C7D8E9F0G1H2",
  "source": "/ministries/a1b2c3d4/sites/b2c3d4e5/services/attendance-service",
  "type": "inwp.attendance.v1.clock-in.created",
  "datacontenttype": "application/json",
  "subject": "employee:d4e5f6a7-b8c9-0123-defa-234567890123",
  "time": "2026-05-31T10:00:00.000Z",
  "dataschema": "inwp:attendance:clock-in:created:v1",
  "ministry_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "site_id": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
  "device_id": "c3d4e5f6-a7b8-9012-cdef-123456789012",
  "offline_generated": false,
  "data": {
    "event_id": "e5f6a7b8-c9d0-1234-efab-345678901234",
    "employee_id": "d4e5f6a7-b8c9-0123-defa-234567890123",
    "ministry_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "site_id": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
    "device_id": "c3d4e5f6-a7b8-9012-cdef-123456789012",
    "event_type": "clock_in",
    "event_time": "2026-05-31T07:55:00.000+03:00",
    "recorded_at": "2026-05-31T07:55:03.000+03:00",
    "timezone": "Asia/Baghdad",
    "biometric": {
      "matched": true,
      "confidence": 0.97,
      "template_hash": "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1"
    },
    "device_time": "2026-05-31T07:55:00.000+03:00"
  }
}
```

### `inwp:attendance:clock-out:created:v1`

Same schema as `clock-in:created:v1` with `event_type` = `"clock_out"`.

### `inwp:attendance:break:started:v1`

Same as clock-in schema with `event_type` = `"break_start"`.

### `inwp:attendance:break:ended:v1`

Same as clock-in schema with `event_type` = `"break_end"`.

### `inwp:attendance:corrected:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:attendance:corrected:v1",
  "title": "Attendance Corrected",
  "description": "Emitted when an attendance record is manually corrected by an admin",
  "type": "object",
  "required": [
    "correction_id", "original_event_id", "employee_id",
    "corrected_by", "reason", "old_values", "new_values"
  ],
  "properties": {
    "correction_id": {
      "type": "string",
      "format": "uuid"
    },
    "original_event_id": {
      "type": "string",
      "format": "uuid",
      "description": "The original clock event being corrected"
    },
    "employee_id": {
      "type": "string",
      "format": "uuid"
    },
    "corrected_by": {
      "type": "string",
      "format": "uuid",
      "description": "Admin user who performed the correction"
    },
    "reason": {
      "type": "string",
      "minLength": 10,
      "maxLength": 500,
      "description": "Justification for the correction"
    },
    "old_values": {
      "type": "object",
      "description": "The original field values before correction",
      "properties": {
        "event_time": { "type": "string", "format": "date-time" },
        "event_type": { "type": "string" }
      }
    },
    "new_values": {
      "type": "object",
      "description": "The corrected field values",
      "properties": {
        "event_time": { "type": "string", "format": "date-time" },
        "event_type": { "type": "string" }
      }
    },
    "approved_by": {
      "type": "string",
      "format": "uuid",
      "description": "Supervisor who approved the correction (if different from corrected_by)"
    }
  }
}
```

### `inwp:attendance:disputed:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:attendance:disputed:v1",
  "title": "Attendance Disputed",
  "description": "Emitted when an employee disputes an attendance record",
  "type": "object",
  "required": ["dispute_id", "event_id", "employee_id", "reason", "disputed_at"],
  "properties": {
    "dispute_id": { "type": "string", "format": "uuid" },
    "event_id": { "type": "string", "format": "uuid" },
    "employee_id": { "type": "string", "format": "uuid" },
    "reason": { "type": "string", "minLength": 10, "maxLength": 1000 },
    "disputed_at": { "type": "string", "format": "date-time" },
    "supporting_evidence": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "type": { "type": "string", "enum": ["photo", "document", "witness_statement"] },
          "url": { "type": "string", "format": "uri" }
        }
      },
      "maxItems": 5
    }
  }
}
```

---

## 2. Leave Events

### `inwp:leave:request:created:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:leave:request:created:v1",
  "title": "Leave Request Created",
  "description": "Emitted when a leave request is created",
  "type": "object",
  "required": [
    "request_id", "employee_id", "ministry_id", "site_id",
    "leave_type", "start_date", "end_date", "duration_days", "reason"
  ],
  "properties": {
    "request_id": { "type": "string", "format": "uuid" },
    "employee_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "site_id": { "type": "string", "format": "uuid" },
    "leave_type": {
      "type": "string",
      "enum": [
        "annual", "sick", "maternity", "paternity",
        "hajj", "umrah", "examination", "compassionate",
        "unpaid", "special", "compensatory"
      ]
    },
    "start_date": {
      "type": "string",
      "format": "date",
      "description": "First day of leave (YYYY-MM-DD)"
    },
    "end_date": {
      "type": "string",
      "format": "date",
      "description": "Last day of leave (YYYY-MM-DD)"
    },
    "duration_days": {
      "type": "integer",
      "minimum": 1,
      "maximum": 365,
      "description": "Number of business days"
    },
    "reason": {
      "type": "string",
      "minLength": 5,
      "maxLength": 2000
    },
    "status": {
      "type": "string",
      "enum": ["draft", "pending_approval", "approved", "rejected", "cancelled"],
      "description": "Initial status is 'draft' or 'pending_approval'"
    },
    "supporting_documents": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "document_id": { "type": "string", "format": "uuid" },
          "document_type": { "type": "string", "enum": ["medical_certificate", "approval_letter", "other"] },
          "file_hash": { "type": "string", "pattern": "^[a-f0-9]{64}$" }
        }
      },
      "maxItems": 5
    },
    "submitted_at": { "type": "string", "format": "date-time" },
    "created_by": { "type": "string", "format": "uuid" }
  }
}
```

### `inwp:leave:request:approved:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:leave:request:approved:v1",
  "title": "Leave Request Approved",
  "description": "Emitted when a leave request is approved at a step in the approval chain",
  "type": "object",
  "required": ["request_id", "employee_id", "ministry_id", "approved_by", "step_order", "approved_at"],
  "properties": {
    "request_id": { "type": "string", "format": "uuid" },
    "employee_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "leave_type": { "type": "string" },
    "approved_by": { "type": "string", "format": "uuid", "description": "Approver user ID" },
    "approver_role": { "type": "string", "description": "Role of the approver at this step" },
    "step_order": { "type": "integer", "minimum": 1, "description": "Which step in the approval chain" },
    "total_steps": { "type": "integer", "minimum": 1, "description": "Total steps in the approval chain" },
    "is_final": { "type": "boolean", "description": "True if this is the final approval step" },
    "approved_at": { "type": "string", "format": "date-time" },
    "comment": { "type": "string", "maxLength": 500 },
    "start_date": { "type": "string", "format": "date" },
    "end_date": { "type": "string", "format": "date" },
    "duration_days": { "type": "integer" }
  }
}
```

### `inwp:leave:request:rejected:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:leave:request:rejected:v1",
  "title": "Leave Request Rejected",
  "description": "Emitted when a leave request is rejected",
  "type": "object",
  "required": ["request_id", "employee_id", "ministry_id", "rejected_by", "reason", "rejected_at"],
  "properties": {
    "request_id": { "type": "string", "format": "uuid" },
    "employee_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "rejected_by": { "type": "string", "format": "uuid" },
    "rejector_role": { "type": "string" },
    "step_order": { "type": "integer" },
    "reason": { "type": "string", "minLength": 5, "maxLength": 1000 },
    "rejected_at": { "type": "string", "format": "date-time" }
  }
}
```

### `inwp:leave:request:cancelled:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:leave:request:cancelled:v1",
  "title": "Leave Request Cancelled",
  "description": "Emitted when a leave request is cancelled",
  "type": "object",
  "required": ["request_id", "employee_id", "ministry_id", "cancelled_by", "reason", "cancelled_at"],
  "properties": {
    "request_id": { "type": "string", "format": "uuid" },
    "employee_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "cancelled_by": { "type": "string", "format": "uuid" },
    "reason": { "type": "string", "maxLength": 500 },
    "cancelled_at": { "type": "string", "format": "date-time" },
    "previous_status": { "type": "string", "enum": ["pending_approval", "approved"] }
  }
}
```

### `inwp:leave:balance:adjusted:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:leave:balance:adjusted:v1",
  "title": "Leave Balance Adjusted",
  "description": "Emitted when a leave balance is manually adjusted or modified by system operation",
  "type": "object",
  "required": [
    "balance_id", "employee_id", "ministry_id", "leave_type",
    "adjustment_type", "amount", "previous_balance", "new_balance", "reason"
  ],
  "properties": {
    "balance_id": { "type": "string", "format": "uuid" },
    "employee_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "leave_type": { "type": "string" },
    "adjustment_type": {
      "type": "string",
      "enum": ["accrual", "deduction", "manual_adjustment", "expiry", "forfeiture", "transfer_in", "transfer_out"]
    },
    "amount": { "type": "number", "description": "Signed value: positive for increase, negative for decrease" },
    "previous_balance": { "type": "number", "minimum": 0 },
    "new_balance": { "type": "number", "minimum": 0 },
    "reason": { "type": "string", "maxLength": 500 },
    "reference_id": {
      "type": "string",
      "format": "uuid",
      "description": "Related leave request ID (for deductions) or accrual policy ID (for accruals)"
    },
    "adjusted_by": { "type": "string", "format": "uuid" },
    "adjusted_at": { "type": "string", "format": "date-time" },
    "fiscal_year": { "type": "string", "pattern": "^\\d{4}-\\d{4}$", "example": "2026-2027" }
  }
}
```

### `inwp:leave:accrual:processed:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:leave:accrual:processed:v1",
  "title": "Accrual Processed",
  "description": "Emitted after a batch accrual run completes for a ministry/fiscal period",
  "type": "object",
  "required": [
    "process_id", "ministry_id", "leave_type", "fiscal_period",
    "employees_affected", "total_amount_accrued"
  ],
  "properties": {
    "process_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "leave_type": { "type": "string" },
    "fiscal_period": {
      "type": "object",
      "properties": {
        "year": { "type": "integer" },
        "month": { "type": "integer", "minimum": 1, "maximum": 12 }
      }
    },
    "employees_affected": { "type": "integer", "minimum": 0 },
    "total_amount_accrued": { "type": "number", "minimum": 0 },
    "policy_id": { "type": "string", "format": "uuid" },
    "errors_count": { "type": "integer", "minimum": 0 },
    "processed_at": { "type": "string", "format": "date-time" },
    "summary": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "employee_id": { "type": "string", "format": "uuid" },
          "amount": { "type": "number" },
          "new_balance": { "type": "number" },
          "success": { "type": "boolean" },
          "error": { "type": "string" }
        }
      },
      "maxItems": 1000
    }
  }
}
```

---

## 3. Identity Events

### `inwp:identity:user:registered:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:identity:user:registered:v1",
  "title": "User Registered",
  "description": "Emitted when a new user account is created in the system",
  "type": "object",
  "required": [
    "user_id", "ministry_id", "national_id", "full_name",
    "status", "registered_at"
  ],
  "properties": {
    "user_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "site_id": { "type": "string", "format": "uuid" },
    "national_id": {
      "type": "string",
      "pattern": "^\\d{14}$",
      "description": "Iraqi national ID number (14 digits)"
    },
    "full_name": {
      "type": "object",
      "properties": {
        "first_name": { "type": "string", "minLength": 2, "maxLength": 50 },
        "father_name": { "type": "string", "minLength": 2, "maxLength": 50 },
        "last_name": { "type": "string", "minLength": 2, "maxLength": 50 }
      }
    },
    "email": {
      "type": "string",
      "format": "email"
    },
    "phone": {
      "type": "string",
      "pattern": "^\\+964\\d{10}$",
      "description": "Iraqi phone number with country code"
    },
    "employment": {
      "type": "object",
      "properties": {
        "employee_code": { "type": "string" },
        "department": { "type": "string" },
        "position": { "type": "string" },
        "employment_type": { "type": "string", "enum": ["full_time", "part_time", "contractor", "temporary"] },
        "joined_at": { "type": "string", "format": "date" }
      }
    },
    "status": {
      "type": "string",
      "enum": ["active", "inactive", "pending_verification"]
    },
    "registered_at": { "type": "string", "format": "date-time" },
    "registered_by": { "type": "string", "format": "uuid", "description": "Admin who created the user" },
    "registration_method": {
      "type": "string",
      "enum": ["admin_create", "self_registration", "scim_provision", "bulk_import"]
    }
  }
}
```

### `inwp:identity:user:verified:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:identity:user:verified:v1",
  "title": "User Verified",
  "description": "Emitted when a user's identity has been verified (e.g., document check, in-person verification)",
  "type": "object",
  "required": ["user_id", "ministry_id", "verified_by", "verification_method", "verified_at"],
  "properties": {
    "user_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "verified_by": { "type": "string", "format": "uuid" },
    "verification_method": {
      "type": "string",
      "enum": ["in_person", "document_review", "biometric_enrollment", "background_check"]
    },
    "verified_at": { "type": "string", "format": "date-time" },
    "previous_status": { "type": "string" },
    "new_status": { "type": "string", "default": "active" },
    "notes": { "type": "string", "maxLength": 500 }
  }
}
```

### `inwp:identity:user:deactivated:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:identity:user:deactivated:v1",
  "title": "User Deactivated",
  "description": "Emitted when a user account is deactivated or suspended",
  "type": "object",
  "required": ["user_id", "ministry_id", "reason", "deactivated_by", "deactivated_at"],
  "properties": {
    "user_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "reason": { "type": "string", "maxLength": 500 },
    "deactivation_type": {
      "type": "string",
      "enum": ["resignation", "termination", "retirement", "transfer", "security_suspension", "death"]
    },
    "deactivated_by": { "type": "string", "format": "uuid" },
    "deactivated_at": { "type": "string", "format": "date-time" },
    "previous_status": { "type": "string" },
    "sessions_terminated": { "type": "boolean", "default": true },
    "device_bindings_removed": { "type": "boolean", "default": true }
  }
}
```

### `inwp:identity:role:assigned:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:identity:role:assigned:v1",
  "title": "Role Assigned",
  "description": "Emitted when a role is assigned to a user within a scope",
  "type": "object",
  "required": ["user_id", "role_id", "role_name", "scope", "assigned_by", "assigned_at"],
  "properties": {
    "user_id": { "type": "string", "format": "uuid" },
    "role_id": { "type": "string", "format": "uuid" },
    "role_name": { "type": "string", "description": "Human-readable role name" },
    "scope": {
      "type": "object",
      "properties": {
        "type": { "type": "string", "enum": ["national", "ministry", "site"] },
        "ministry_id": { "type": "string", "format": "uuid" },
        "site_id": { "type": "string", "format": "uuid" }
      }
    },
    "assigned_by": { "type": "string", "format": "uuid" },
    "assigned_at": { "type": "string", "format": "date-time" },
    "expires_at": { "type": "string", "format": "date-time", "description": "Optional role expiry" },
    "ministry_id": { "type": "string", "format": "uuid" }
  }
}
```

### `inwp:identity:role:revoked:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:identity:role:revoked:v1",
  "title": "Role Revoked",
  "description": "Emitted when a role is revoked from a user",
  "type": "object",
  "required": ["user_id", "role_id", "role_name", "scope", "revoked_by", "revoked_at"],
  "properties": {
    "user_id": { "type": "string", "format": "uuid" },
    "role_id": { "type": "string", "format": "uuid" },
    "role_name": { "type": "string" },
    "scope": {
      "type": "object",
      "properties": {
        "type": { "type": "string", "enum": ["national", "ministry", "site"] },
        "ministry_id": { "type": "string", "format": "uuid" },
        "site_id": { "type": "string", "format": "uuid" }
      }
    },
    "revoked_by": { "type": "string", "format": "uuid" },
    "revoked_at": { "type": "string", "format": "date-time" },
    "reason": { "type": "string", "maxLength": 500 }
  }
}
```

### `inwp:identity:credential:changed:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:identity:credential:changed:v1",
  "title": "Credential Changed",
  "description": "Emitted when a user's credential is added, changed, or revoked",
  "type": "object",
  "required": ["user_id", "ministry_id", "credential_type", "action", "changed_at"],
  "properties": {
    "user_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "credential_type": {
      "type": "string",
      "enum": ["password", "totp", "biometric_fingerprint", "biometric_face", "smart_card", "device_certificate"]
    },
    "action": { "type": "string", "enum": ["added", "changed", "revoked", "expired"] },
    "changed_at": { "type": "string", "format": "date-time" },
    "changed_by": { "type": "string", "format": "uuid" },
    "reason": { "type": "string", "maxLength": 200 }
  }
}
```

---

## 4. Audit Events

### `inwp:audit:entry:appended:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:audit:entry:appended:v1",
  "title": "Ledger Entry Appended",
  "description": "Emitted when a new entry is appended to the immutable audit ledger",
  "type": "object",
  "required": [
    "entry_id", "prev_hash", "payload_hash", "entry_hash",
    "source_service", "source_node_id", "event_type", "event_id", "timestamp"
  ],
  "properties": {
    "entry_id": { "type": "string", "format": "uuid", "description": "Ledger sequence ID" },
    "prev_hash": {
      "type": "string",
      "pattern": "^[a-f0-9]{64}$",
      "description": "SHA-256 of previous ledger entry (64 hex chars)"
    },
    "payload_hash": {
      "type": "string",
      "pattern": "^[a-f0-9]{64}$",
      "description": "SHA-256 of the event payload"
    },
    "entry_hash": {
      "type": "string",
      "pattern": "^[a-f0-9]{64}$",
      "description": "SHA-256(prev_hash || payload_hash || timestamp || nonce)"
    },
    "nonce": {
      "type": "string",
      "pattern": "^[a-f0-9]{16}$",
      "description": "16 hex chars (8 bytes) ensuring uniqueness"
    },
    "source_service": { "type": "string", "description": "Service that produced the event" },
    "source_node_id": { "type": "string", "format": "uuid" },
    "event_type": { "type": "string", "description": "The original event type" },
    "event_id": { "type": "string", "format": "uuid", "description": "Reference to original event" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "timestamp": { "type": "string", "format": "date-time" },
    "entry_position": { "type": "integer", "minimum": 1, "description": "Monotonic position in ledger" },
    "signature": {
      "type": "string",
      "pattern": "^[a-f0-9]{128}$",
      "description": "Ed25519 signature of entry_hash (128 hex chars)"
    },
    "signing_key_id": { "type": "string" }
  }
}
```

### `inwp:audit:seal:generated:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:audit:seal:generated:v1",
  "title": "Audit Seal Generated",
  "description": "Emitted when a periodic cryptographic seal is created over a range of ledger entries",
  "type": "object",
  "required": [
    "seal_id", "start_entry_id", "end_entry_id", "entry_count",
    "merkle_root", "previous_seal_hash", "seal_hash", "sealed_at"
  ],
  "properties": {
    "seal_id": { "type": "string", "format": "uuid" },
    "start_entry_id": { "type": "string", "format": "uuid" },
    "end_entry_id": { "type": "string", "format": "uuid" },
    "entry_count": { "type": "integer", "minimum": 1 },
    "merkle_root": {
      "type": "string",
      "pattern": "^[a-f0-9]{64}$",
      "description": "Merkle root of all entries in this seal range"
    },
    "previous_seal_hash": {
      "type": "string",
      "pattern": "^[a-f0-9]{64}$",
      "description": "Hash of the previous seal (zero hash for first seal)"
    },
    "seal_hash": {
      "type": "string",
      "pattern": "^[a-f0-9]{64}$",
      "description": "SHA-256(merkle_root || previous_seal_hash || sealed_at)"
    },
    "signature": {
      "type": "string",
      "pattern": "^[a-f0-9]{128}$",
      "description": "Ed25519 signature of seal_hash"
    },
    "sealed_at": { "type": "string", "format": "date-time" },
    "published_url": { "type": "string", "format": "uri", "description": "URL where seal is published for external verification" }
  }
}
```

### `inwp:audit:seal:verified:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:audit:seal:verified:v1",
  "title": "Audit Seal Verified",
  "description": "Emitted when an audit seal is verified (either by automated process or external auditor)",
  "type": "object",
  "required": ["seal_id", "verified_at", "valid"],
  "properties": {
    "seal_id": { "type": "string", "format": "uuid" },
    "start_entry_id": { "type": "string", "format": "uuid" },
    "end_entry_id": { "type": "string", "format": "uuid" },
    "verified_at": { "type": "string", "format": "date-time" },
    "valid": { "type": "boolean" },
    "verifier": { "type": "string", "description": "Entity that performed the verification" },
    "chain_valid": { "type": "boolean" },
    "seal_hash_valid": { "type": "boolean" },
    "signature_valid": { "type": "boolean" },
    "failures": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "entry_id": { "type": "string", "format": "uuid" },
          "expected_hash": { "type": "string" },
          "actual_hash": { "type": "string" },
          "position": { "type": "integer" }
        }
      }
    }
  }
}
```

### `inwp:audit:integrity:failure:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:audit:integrity:failure:v1",
  "title": "Audit Integrity Failure",
  "description": "Emitted when a hash chain integrity check fails — indicates potential tampering",
  "type": "object",
  "required": ["check_id", "failed_entry_id", "expected_hash", "actual_hash", "detected_at", "severity"],
  "properties": {
    "check_id": { "type": "string", "format": "uuid" },
    "failed_entry_id": { "type": "string", "format": "uuid" },
    "entry_position": { "type": "integer" },
    "expected_hash": { "type": "string", "pattern": "^[a-f0-9]{64}$" },
    "actual_hash": { "type": "string", "pattern": "^[a-f0-9]{64}$" },
    "source_service": { "type": "string" },
    "source_node_id": { "type": "string", "format": "uuid" },
    "detected_at": { "type": "string", "format": "date-time" },
    "severity": { "type": "string", "enum": ["warning", "critical", "catastrophic"] },
    "affected_range": {
      "type": "object",
      "properties": {
        "start_entry": { "type": "string", "format": "uuid" },
        "end_entry": { "type": "string", "format": "uuid" },
        "entry_count": { "type": "integer" }
      }
    },
    "alert": { "type": "boolean", "default": true },
    "alert_sent_at": { "type": "string", "format": "date-time" }
  }
}
```

---

## 5. Synchronization Events

### `inwp:sync:batch:committed:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:sync:batch:committed:v1",
  "title": "Sync Batch Committed",
  "description": "Emitted when a sync batch is fully committed by both source and target nodes",
  "type": "object",
  "required": [
    "sync_id", "source_node_id", "target_node_id",
    "partition_key", "event_count", "byte_count",
    "source_signature", "target_signature", "committed_at"
  ],
  "properties": {
    "sync_id": { "type": "string", "format": "uuid" },
    "source_node_id": { "type": "string", "format": "uuid", "description": "Sender node" },
    "target_node_id": { "type": "string", "format": "uuid", "description": "Receiver node" },
    "partition_key": { "type": "string", "description": "e.g., 'mohe/attendance/2026-05'" },
    "direction": { "type": "string", "enum": ["upload", "download", "bidirectional"] },
    "event_count": { "type": "integer", "minimum": 0 },
    "byte_count": { "type": "integer", "minimum": 0 },
    "conflict_count": { "type": "integer", "minimum": 0, "default": 0 },
    "conflicts_resolved_auto": { "type": "integer", "minimum": 0, "default": 0 },
    "conflicts_manual_required": { "type": "integer", "minimum": 0, "default": 0 },
    "source_merkle_root": { "type": "string", "pattern": "^[a-f0-9]{64}$" },
    "target_merkle_root": { "type": "string", "pattern": "^[a-f0-9]{64}$" },
    "source_signature": { "type": "string", "pattern": "^[a-f0-9]{128}$" },
    "target_signature": { "type": "string", "pattern": "^[a-f0-9]{128}$" },
    "committed_at": { "type": "string", "format": "date-time" },
    "duration_ms": { "type": "integer", "description": "Sync duration in milliseconds" },
    "compression_ratio": { "type": "number", "minimum": 1, "description": "Compression ratio achieved" }
  }
}
```

### `inwp:sync:conflict:detected:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:sync:conflict:detected:v1",
  "title": "Sync Conflict Detected",
  "description": "Emitted when a data conflict is detected during synchronization",
  "type": "object",
  "required": [
    "conflict_id", "sync_id", "record_id", "record_type",
    "partition_key", "local_version", "remote_version", "detected_at"
  ],
  "properties": {
    "conflict_id": { "type": "string", "format": "uuid" },
    "sync_id": { "type": "string", "format": "uuid" },
    "record_id": { "type": "string", "description": "The record that has conflicting versions" },
    "record_type": { "type": "string", "description": "e.g., 'attendance.clock_events', 'leave.requests'" },
    "partition_key": { "type": "string" },
    "local_version": { "type": "integer" },
    "remote_version": { "type": "integer" },
    "local_timestamp": { "type": "string", "format": "date-time" },
    "remote_timestamp": { "type": "string", "format": "date-time" },
    "local_snapshot": { "type": "object", "description": "Local version of the conflicting record" },
    "remote_snapshot": { "type": "object", "description": "Remote version of the conflicting record" },
    "auto_resolvable": { "type": "boolean" },
    "suggested_strategy": {
      "type": "string",
      "enum": ["lww_timestamp", "ministry_author_wins", "service_merge", "manual"]
    },
    "detected_at": { "type": "string", "format": "date-time" }
  }
}
```

### `inwp:sync:conflict:resolved:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:sync:conflict:resolved:v1",
  "title": "Sync Conflict Resolved",
  "description": "Emitted when a data conflict is resolved (auto or manual)",
  "type": "object",
  "required": [
    "conflict_id", "sync_id", "record_id", "record_type",
    "resolution", "resolved_by", "resolved_at"
  ],
  "properties": {
    "conflict_id": { "type": "string", "format": "uuid" },
    "sync_id": { "type": "string", "format": "uuid" },
    "record_id": { "type": "string" },
    "record_type": { "type": "string" },
    "resolution": {
      "type": "string",
      "enum": ["local_wins", "remote_wins", "merged", "manual_override"]
    },
    "resolution_strategy": {
      "type": "string",
      "enum": ["lww_timestamp", "ministry_author_wins", "service_merge", "admin_decision"]
    },
    "resolved_by": {
      "type": "string",
      "description": "'auto-lww', 'auto-ministry-author', or admin user UUID"
    },
    "resolved_at": { "type": "string", "format": "date-time" },
    "winning_version": { "type": "integer", "description": "The version that was chosen" },
    "merged_payload": { "type": "object", "description": "Result of merge (if resolution = merged)" }
  }
}
```

### `inwp:sync:heartbeat:sent:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:sync:heartbeat:sent:v1",
  "title": "Node Heartbeat Sent",
  "description": "Emitted periodically by each sync node to signal liveness",
  "type": "object",
  "required": ["node_id", "node_type", "sent_at", "status"],
  "properties": {
    "node_id": { "type": "string", "format": "uuid" },
    "node_type": { "type": "string", "enum": ["edge", "regional_relay", "national_hub"] },
    "sent_at": { "type": "string", "format": "date-time" },
    "status": { "type": "string", "enum": ["online", "degraded"] },
    "uptime_seconds": { "type": "integer", "minimum": 0 },
    "services_running": {
      "type": "array",
      "items": { "type": "string", "enum": ["attendance", "leave", "sync", "identity_cache"] }
    },
    "last_sync_at": { "type": "string", "format": "date-time" },
    "pending_events_count": { "type": "integer", "minimum": 0, "description": "Events awaiting sync" },
    "storage_usage_percent": { "type": "number", "minimum": 0, "maximum": 100 },
    "cpu_load_percent": { "type": "number", "minimum": 0, "maximum": 100 },
    "memory_usage_percent": { "type": "number", "minimum": 0, "maximum": 100 },
    "ministry_id": { "type": "string", "format": "uuid" },
    "site_id": { "type": "string", "format": "uuid" },
    "schema_versions": {
      "type": "object",
      "additionalProperties": { "type": "string" },
      "description": "Map of entity type to schema version supported"
    }
  }
}
```

---

## 6. Device Registration Events

### `inwp:device:enrolled:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:device:enrolled:v1",
  "title": "Device Enrolled",
  "description": "Emitted when a new device is enrolled into the platform",
  "type": "object",
  "required": [
    "device_id", "device_type", "manufacturer", "model",
    "serial_number", "ministry_id", "site_id", "trust_level", "enrolled_at"
  ],
  "properties": {
    "device_id": { "type": "string", "format": "uuid" },
    "device_type": {
      "type": "string",
      "enum": [
        "fingerprint_scanner", "face_recognition_terminal", "card_reader",
        "pin_pad", "mobile_device", "tablet", "iot_sensor"
      ]
    },
    "manufacturer": { "type": "string", "maxLength": 100 },
    "model": { "type": "string", "maxLength": 100 },
    "serial_number": { "type": "string", "maxLength": 100 },
    "firmware_version": { "type": "string" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "site_id": { "type": "string", "format": "uuid" },
    "trust_level": {
      "type": "string",
      "enum": ["trusted", "provisional", "degraded", "suspended", "revoked"],
      "description": "Initial trust level after enrollment (typically 'provisional')"
    },
    "enrolled_at": { "type": "string", "format": "date-time" },
    "enrolled_by": { "type": "string", "format": "uuid", "description": "Admin who enrolled the device" },
    "attestation_method": {
      "type": "string",
      "enum": ["tpm", "trustzone", "manufacturer_cert", "manual_approval"]
    },
    "attestation_result": {
      "type": "object",
      "properties": {
        "verified": { "type": "boolean" },
        "verifier": { "type": "string" },
        "verified_at": { "type": "string", "format": "date-time" }
      }
    },
    "certificate_serial": { "type": "string", "description": "Issued device certificate serial number" },
    "certificate_expiry": { "type": "string", "format": "date-time" },
    "network_info": {
      "type": "object",
      "properties": {
        "mac_address": { "type": "string", "pattern": "^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$" },
        "ip_address": { "type": "string", "format": "ipv4" }
      }
    },
    "capabilities": {
      "type": "array",
      "items": { "type": "string", "enum": ["biometric_match", "card_read", "pin_entry", "face_capture"] }
    }
  }
}
```

### `inwp:device:activated:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:device:activated:v1",
  "title": "Device Activated",
  "description": "Emitted when a device transitions from 'provisional' to 'trusted' after attestation verification",
  "type": "object",
  "required": ["device_id", "ministry_id", "site_id", "trust_level", "activated_at"],
  "properties": {
    "device_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "site_id": { "type": "string", "format": "uuid" },
    "previous_trust_level": { "type": "string", "default": "provisional" },
    "trust_level": { "type": "string", "default": "trusted" },
    "activated_at": { "type": "string", "format": "date-time" },
    "activated_by": { "type": "string", "format": "uuid" },
    "activation_method": {
      "type": "string",
      "enum": ["attestation_verified", "admin_override", "certificate_issued"]
    }
  }
}
```

### `inwp:device:suspended:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:device:suspended:v1",
  "title": "Device Suspended",
  "description": "Emitted when a device is suspended due to trust degradation, security concern, or admin action",
  "type": "object",
  "required": ["device_id", "ministry_id", "site_id", "reason", "suspended_by", "suspended_at"],
  "properties": {
    "device_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "site_id": { "type": "string", "format": "uuid" },
    "previous_trust_level": { "type": "string" },
    "new_trust_level": { "type": "string", "default": "suspended" },
    "reason": { "type": "string", "maxLength": 500 },
    "suspension_reason_code": {
      "type": "string",
      "enum": [
        "certificate_expired", "attestation_failure", "firmware_outdated",
        "anomalous_activity", "compromised_reported", "admin_action",
        "trust_score_below_threshold", "inactivity_timeout"
      ]
    },
    "suspended_by": {
      "type": "string",
      "description": "'system' for automatic suspension, or admin UUID"
    },
    "suspended_at": { "type": "string", "format": "date-time" },
    "auto_unsuspend_at": { "type": "string", "format": "date-time", "description": "If suspension is temporary" }
  }
}
```

### `inwp:device:revoked:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:device:revoked:v1",
  "title": "Device Revoked",
  "description": "Emitted when a device is permanently revoked (decommissioned or compromised)",
  "type": "object",
  "required": ["device_id", "ministry_id", "site_id", "reason", "revoked_by", "revoked_at"],
  "properties": {
    "device_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "site_id": { "type": "string", "format": "uuid" },
    "reason": { "type": "string", "maxLength": 500 },
    "revocation_reason_code": {
      "type": "string",
      "enum": ["decommissioned", "compromised", "stolen", "replaced", "ministry_audit"]
    },
    "revoked_by": { "type": "string", "format": "uuid" },
    "revoked_at": { "type": "string", "format": "date-time" },
    "certificate_revoked": { "type": "boolean", "default": true },
    "sessions_terminated": { "type": "boolean", "default": true },
    "employee_bindings_removed": { "type": "boolean", "default": true },
    "replacement_device_id": { "type": "string", "format": "uuid", "description": "If device was replaced" }
  }
}
```

### `inwp:device:trust-changed:v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "inwp:device:trust-changed:v1",
  "title": "Device Trust Changed",
  "description": "Emitted when a device's trust score or trust level changes (e.g., due to decay or improvement)",
  "type": "object",
  "required": [
    "device_id", "ministry_id", "site_id",
    "previous_trust_level", "new_trust_level",
    "previous_trust_score", "new_trust_score", "changed_at"
  ],
  "properties": {
    "device_id": { "type": "string", "format": "uuid" },
    "ministry_id": { "type": "string", "format": "uuid" },
    "site_id": { "type": "string", "format": "uuid" },
    "previous_trust_level": { "type": "string", "enum": ["trusted", "provisional", "degraded", "suspended", "revoked"] },
    "new_trust_level": { "type": "string", "enum": ["trusted", "provisional", "degraded", "suspended", "revoked"] },
    "previous_trust_score": { "type": "number", "minimum": 0, "maximum": 1 },
    "new_trust_score": { "type": "number", "minimum": 0, "maximum": 1 },
    "change_reason": {
      "type": "string",
      "enum": [
        "attestation_refresh", "trust_decay", "firmware_update",
        "certificate_renewal", "anomaly_detected", "admin_adjustment",
        "heartbeat_missed", "successful_sync"
      ]
    },
    "changed_at": { "type": "string", "format": "date-time" },
    "days_since_last_attestation": { "type": "integer" },
    "anomaly_count_30d": { "type": "integer" }
  }
}
```

---

## Schema Index

| # | Domain | Event Type | Schema ID | Version |
|---|---|---|---|---|
| 1 | attendance | `clock-in.created` | `inwp:attendance:clock-in:created:v1` | 1.0.0 |
| 2 | attendance | `clock-out.created` | `inwp:attendance:clock-out:created:v1` | 1.0.0 |
| 3 | attendance | `break.started` | `inwp:attendance:break:started:v1` | 1.0.0 |
| 4 | attendance | `break.ended` | `inwp:attendance:break:ended:v1` | 1.0.0 |
| 5 | attendance | `attendance.corrected` | `inwp:attendance:corrected:v1` | 1.0.0 |
| 6 | attendance | `attendance.disputed` | `inwp:attendance:disputed:v1` | 1.0.0 |
| 7 | leave | `request.created` | `inwp:leave:request:created:v1` | 1.0.0 |
| 8 | leave | `request.approved` | `inwp:leave:request:approved:v1` | 1.0.0 |
| 9 | leave | `request.rejected` | `inwp:leave:request:rejected:v1` | 1.0.0 |
| 10 | leave | `request.cancelled` | `inwp:leave:request:cancelled:v1` | 1.0.0 |
| 11 | leave | `balance.adjusted` | `inwp:leave:balance:adjusted:v1` | 1.0.0 |
| 12 | leave | `accrual.processed` | `inwp:leave:accrual:processed:v1` | 1.0.0 |
| 13 | identity | `user.registered` | `inwp:identity:user:registered:v1` | 1.0.0 |
| 14 | identity | `user.verified` | `inwp:identity:user:verified:v1` | 1.0.0 |
| 15 | identity | `user.deactivated` | `inwp:identity:user:deactivated:v1` | 1.0.0 |
| 16 | identity | `role.assigned` | `inwp:identity:role:assigned:v1` | 1.0.0 |
| 17 | identity | `role.revoked` | `inwp:identity:role:revoked:v1` | 1.0.0 |
| 18 | identity | `credential.changed` | `inwp:identity:credential:changed:v1` | 1.0.0 |
| 19 | audit | `entry.appended` | `inwp:audit:entry:appended:v1` | 1.0.0 |
| 20 | audit | `seal.generated` | `inwp:audit:seal:generated:v1` | 1.0.0 |
| 21 | audit | `seal.verified` | `inwp:audit:seal:verified:v1` | 1.0.0 |
| 22 | audit | `integrity.failure` | `inwp:audit:integrity:failure:v1` | 1.0.0 |
| 23 | sync | `batch.committed` | `inwp:sync:batch:committed:v1` | 1.0.0 |
| 24 | sync | `conflict.detected` | `inwp:sync:conflict:detected:v1` | 1.0.0 |
| 25 | sync | `conflict.resolved` | `inwp:sync:conflict:resolved:v1` | 1.0.0 |
| 26 | sync | `heartbeat.sent` | `inwp:sync:heartbeat:sent:v1` | 1.0.0 |
| 27 | device | `device.enrolled` | `inwp:device:enrolled:v1` | 1.0.0 |
| 28 | device | `device.activated` | `inwp:device:activated:v1` | 1.0.0 |
| 29 | device | `device.suspended` | `inwp:device:suspended:v1` | 1.0.0 |
| 30 | device | `device.revoked` | `inwp:device:revoked:v1` | 1.0.0 |
| 31 | device | `device.trust-changed` | `inwp:device:trust-changed:v1` | 1.0.0 |

---

## Validation Requirements

| Rule | Enforced At | Behaviour |
|---|---|---|
| Required field missing | Schema validation | Event rejected with `400 VALIDATION_ERROR` |
| UUID format invalid | Schema validation | Event rejected |
| Date in future (for historical events) | Schema validation | Rejected unless `offline_generated: true` |
| Event type mismatch | Schema registry | Rejected (`type` must match schema) |
| Unknown `dataschema` reference | Schema registry | Rejected (must reference registered schema) |
| Payload exceeds 1MB | Gateway | Rejected with `413 PAYLOAD_TOO_LARGE` |
| Duplicate `id` within 24h | Idempotency check | Silently acknowledged (no re-processing) |
| Invalid signature | Consumer | Dead-letter queue, security alert |

## Schema Registry Location

Schemas are stored in `platform-core/pkg/schemas/` as versioned JSON files:

```
pkg/schemas/
├── v1/
│   ├── attendance/
│   │   ├── clock-in.created.json
│   │   ├── clock-out.created.json
│   │   ├── break.started.json
│   │   ├── break.ended.json
│   │   ├── attendance.corrected.json
│   │   └── attendance.disputed.json
│   ├── leave/
│   │   ├── request.created.json
│   │   ├── request.approved.json
│   │   ├── request.rejected.json
│   │   ├── request.cancelled.json
│   │   ├── balance.adjusted.json
│   │   └── accrual.processed.json
│   ├── identity/
│   │   ├── user.registered.json
│   │   ├── user.verified.json
│   │   ├── user.deactivated.json
│   │   ├── role.assigned.json
│   │   ├── role.revoked.json
│   │   └── credential.changed.json
│   ├── audit/
│   │   ├── entry.appended.json
│   │   ├── seal.generated.json
│   │   ├── seal.verified.json
│   │   └── integrity.failure.json
│   ├── sync/
│   │   ├── batch.committed.json
│   │   ├── conflict.detected.json
│   │   ├── conflict.resolved.json
│   │   └── heartbeat.sent.json
│   └── device/
│       ├── device.enrolled.json
│       ├── device.activated.json
│       ├── device.suspended.json
│       ├── device.revoked.json
│       └── device.trust-changed.json
└── schema-registry.json         # Master index
```
