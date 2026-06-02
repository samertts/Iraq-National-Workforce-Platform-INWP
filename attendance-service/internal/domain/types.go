package domain

import (
	"time"

	"github.com/google/uuid"
)

type EmployeeID uuid.UUID
type DeviceID uuid.UUID
type SiteID uuid.UUID
type MinistryID uuid.UUID
type GroupID uuid.UUID

type ClockEventType string

const (
	ClockIn           ClockEventType = "clock_in"
	ClockOut          ClockEventType = "clock_out"
	BreakStart        ClockEventType = "break_start"
	BreakEnd          ClockEventType = "break_end"
	ManualCorrection  ClockEventType = "manual_correction"
)

type SyncStatus string

const (
	SyncStatusLocalOnly  SyncStatus = "local_only"
	SyncStatusSynced     SyncStatus = "synced"
	SyncStatusConflicted SyncStatus = "conflicted"
	SyncStatusResolved   SyncStatus = "resolved"
)

type SyncMetadata struct {
	SyncID       uuid.UUID  `json:"sync_id"`
	SourceNodeID uuid.UUID  `json:"source_node_id"`
	Status       SyncStatus `json:"status"`
	LastSyncedAt *time.Time `json:"last_synced_at,omitempty"`
	Version      int64      `json:"version"`
}

type OfflineTimestamp struct {
	DeviceTime time.Time `json:"device_time"`
	ServerTime *time.Time `json:"server_time,omitempty"`
	Timezone   string     `json:"timezone"`
}

type DomainEvent interface {
	EventID() uuid.UUID
	EventType() string
	EventVersion() string
	OccurredAt() time.Time
	Source() string
	MinistryID() uuid.UUID
	SiteID() *uuid.UUID
}

type Entity interface {
	Identity() uuid.UUID
}

type AggregateRoot interface {
	Entity
	Version() int64
	DomainEvents() []DomainEvent
	ClearEvents()
}

type HasMinistry interface {
	MinistryID() uuid.UUID
}

type HasSite interface {
	MinistryID() uuid.UUID
	SiteID() uuid.UUID
}

type ExceptionType string

const (
	LateClockIn        ExceptionType = "LATE_CLOCK_IN"
	EarlyClockOut      ExceptionType = "EARLY_CLOCK_OUT"
	MissingClockIn     ExceptionType = "MISSING_CLOCK_IN"
	MissingClockOut    ExceptionType = "MISSING_CLOCK_OUT"
	MissingBreak       ExceptionType = "MISSING_BREAK"
	ExceededMaxHours   ExceptionType = "EXCEEDED_MAX_HOURS"
	DuplicateClock     ExceptionType = "DUPLICATE_CLOCK"
	BiometricMismatch  ExceptionType = "BIOMETRIC_MISMATCH"
	GPSOutOfRange      ExceptionType = "GPS_OUT_OF_RANGE"
)

type ExceptionSeverity string

const (
	SeverityLow      ExceptionSeverity = "LOW"
	SeverityMedium   ExceptionSeverity = "MEDIUM"
	SeverityHigh     ExceptionSeverity = "HIGH"
	SeverityCritical ExceptionSeverity = "CRITICAL"
)

type JustificationType string

const (
	JustificationSickness      JustificationType = "SICKNESS"
	JustificationOfficialDuty  JustificationType = "OFFICIAL_DUTY"
	JustificationTechnicalIssue JustificationType = "TECHNICAL_ISSUE"
	JustificationOther         JustificationType = "OTHER"
)

type OvertimePolicy struct {
	Enabled            bool          `json:"enabled"`
	RateMultiplier     float64       `json:"rate_multiplier"`
	ThresholdHours     int           `json:"threshold_hours"`
	MaxOvertimePerDay  time.Duration `json:"max_overtime_per_day"`
	RequiresApproval   bool          `json:"requires_approval"`
	DoubleHolidayRate  bool          `json:"double_holiday_rate"`
}

type AttendanceRuleSet struct {
	ClockInWindow                TimeRange     `json:"clock_in_window"`
	LateThreshold                time.Duration `json:"late_threshold"`
	EarlyDepartureThreshold      time.Duration `json:"early_departure_threshold"`
	AutoApprovalWindow           time.Duration `json:"auto_approval_window"`
	RequireBiometric             bool          `json:"require_biometric"`
	RequireGPS                   bool          `json:"require_gps"`
	MaxDailyClocks               int           `json:"max_daily_clocks"`
	MinHoursBetweenClocks        time.Duration `json:"min_hours_between_clocks"`
	AllowManualCorrection        bool          `json:"allow_manual_correction"`
	CorrectionRequiresApproval   bool          `json:"correction_requires_approval"`
	DisputeWindow                time.Duration `json:"dispute_window"`
}

type TimeRange struct {
	Start time.Time `json:"start"`
	End   time.Time `json:"end"`
}

type Justification struct {
	Reason         string            `json:"reason"`
	Type           JustificationType `json:"type"`
	SubmittedAt    time.Time         `json:"submitted_at"`
	ApprovedBy     *uuid.UUID        `json:"approved_by,omitempty"`
	ApprovedAt     *time.Time        `json:"approved_at,omitempty"`
}
