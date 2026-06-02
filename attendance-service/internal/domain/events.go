package domain

import (
	"time"

	"github.com/google/uuid"
)

type BaseEvent struct {
	ID          uuid.UUID  `json:"id"`
	Type        string     `json:"type"`
	Version     string     `json:"version"`
	Time        time.Time  `json:"time"`
	Src         string     `json:"source"`
	Mtd         uuid.UUID  `json:"ministry_id"`
	StID        *uuid.UUID `json:"site_id,omitempty"`
	DevID       *uuid.UUID `json:"device_id,omitempty"`
	EmpID       *uuid.UUID `json:"employee_id,omitempty"`
}

func (e BaseEvent) EventID() uuid.UUID       { return e.ID }
func (e BaseEvent) EventType() string        { return e.Type }
func (e BaseEvent) EventVersion() string     { return e.Version }
func (e BaseEvent) OccurredAt() time.Time    { return e.Time }
func (e BaseEvent) Source() string           { return e.Src }
func (e BaseEvent) MinistryID() uuid.UUID    { return e.Mtd }
func (e BaseEvent) SiteID() *uuid.UUID       { return e.StID }

type ClockInCreated struct {
	BaseEvent
	EmployeeID uuid.UUID `json:"employee_id"`
	DeviceID   uuid.UUID `json:"device_id"`
	EventTime  time.Time `json:"event_time"`
}

type ClockOutCreated struct {
	BaseEvent
	EmployeeID uuid.UUID `json:"employee_id"`
	DeviceID   uuid.UUID `json:"device_id"`
	EventTime  time.Time `json:"event_time"`
}

type BreakStarted struct {
	BaseEvent
	EmployeeID uuid.UUID `json:"employee_id"`
	EventTime  time.Time `json:"event_time"`
}

type BreakEnded struct {
	BaseEvent
	EmployeeID uuid.UUID `json:"employee_id"`
	EventTime  time.Time `json:"event_time"`
}

type AttendanceCorrected struct {
	BaseEvent
	EmployeeID  uuid.UUID `json:"employee_id"`
	CorrectedBy uuid.UUID `json:"corrected_by"`
	OldEvent    uuid.UUID `json:"old_event_id"`
	NewEvent    uuid.UUID `json:"new_event_id"`
}

type AttendanceDisputed struct {
	BaseEvent
	EmployeeID uuid.UUID `json:"employee_id"`
	Reason     string    `json:"reason"`
	DisputedAt time.Time `json:"disputed_at"`
}

type ShiftCreated struct {
	BaseEvent
	ShiftID uuid.UUID `json:"shift_id"`
	Name    string    `json:"name"`
}

type ShiftModified struct {
	BaseEvent
	ShiftID    uuid.UUID `json:"shift_id"`
	ModifiedBy uuid.UUID `json:"modified_by"`
}

type ShiftDeactivated struct {
	BaseEvent
	ShiftID       uuid.UUID `json:"shift_id"`
	DeactivatedAt time.Time `json:"deactivated_at"`
}

type PolicyCreated struct {
	BaseEvent
	PolicyID      uuid.UUID `json:"policy_id"`
	EffectiveFrom time.Time `json:"effective_from"`
}

type PolicyActivated struct {
	BaseEvent
	PolicyID    uuid.UUID `json:"policy_id"`
	ActivatedAt time.Time `json:"activated_at"`
}

type PolicySuperseded struct {
	BaseEvent
	PolicyID      uuid.UUID `json:"policy_id"`
	SupersededBy  uuid.UUID `json:"superseded_by"`
	SupersededAt  time.Time `json:"superseded_at"`
}

type ExceptionCreated struct {
	BaseEvent
	ExceptionID   uuid.UUID     `json:"exception_id"`
	ExceptionType ExceptionType `json:"exception_type"`
}

type ExceptionJustified struct {
	BaseEvent
	ExceptionID      uuid.UUID        `json:"exception_id"`
	JustificationType JustificationType `json:"justification_type"`
}

type ExceptionResolved struct {
	BaseEvent
	ExceptionID uuid.UUID `json:"exception_id"`
	ResolvedBy  uuid.UUID `json:"resolved_by"`
}

type ExceptionEscalated struct {
	BaseEvent
	ExceptionID uuid.UUID `json:"exception_id"`
	EscalatedTo uuid.UUID `json:"escalated_to"`
}
