package domain

import (
	"time"

	"github.com/google/uuid"
)

type AttendanceException struct {
	id              uuid.UUID
	employeeID      EmployeeID
	clockEventID    *uuid.UUID
	exceptionType   ExceptionType
	severity        ExceptionSeverity
	description     string
	occurredAt      time.Time
	detectedAt      time.Time
	justification   *Justification
	resolvedAt      *time.Time
	resolvedBy      *uuid.UUID
	escalatedAt     *time.Time
	domainEvents    []DomainEvent
}

func NewAttendanceException(
	employeeID EmployeeID,
	clockEventID *uuid.UUID,
	exceptionType ExceptionType,
	severity ExceptionSeverity,
	description string,
	occurredAt time.Time,
) *AttendanceException {
	return &AttendanceException{
		id:            uuid.New(),
		employeeID:    employeeID,
		clockEventID:  clockEventID,
		exceptionType: exceptionType,
		severity:      severity,
		description:   description,
		occurredAt:    occurredAt,
		detectedAt:    time.Now().UTC(),
		domainEvents:  make([]DomainEvent, 0),
	}
}

func (e *AttendanceException) Identity() uuid.UUID        { return e.id }
func (e *AttendanceException) Version() int64              { return 1 }
func (e *AttendanceException) DomainEvents() []DomainEvent { return e.domainEvents }
func (e *AttendanceException) ClearEvents()                { e.domainEvents = nil }
func (e *AttendanceException) EmployeeID() EmployeeID      { return e.employeeID }
func (e *AttendanceException) ExceptionType() ExceptionType { return e.exceptionType }
func (e *AttendanceException) Severity() ExceptionSeverity  { return e.severity }
func (e *AttendanceException) Description() string          { return e.description }
func (e *AttendanceException) IsResolved() bool             { return e.resolvedAt != nil }
func (e *AttendanceException) IsEscalated() bool            { return e.escalatedAt != nil }

func (e *AttendanceException) Justify(reason string, jType JustificationType) {
	e.justification = &Justification{
		Reason:      reason,
		Type:        jType,
		SubmittedAt: time.Now().UTC(),
	}
}

func (e *AttendanceException) Resolve(resolvedBy uuid.UUID) {
	now := time.Now().UTC()
	e.resolvedAt = &now
	e.resolvedBy = &resolvedBy
}

func (e *AttendanceException) Escalate() {
	now := time.Now().UTC()
	e.escalatedAt = &now
}

func (e *AttendanceException) RaiseEvent(event DomainEvent) {
	e.domainEvents = append(e.domainEvents, event)
}

type AttendanceExceptionRepository interface {
	Save(exception *AttendanceException) error
	FindByID(id uuid.UUID) (*AttendanceException, error)
	FindByEmployee(employeeID EmployeeID, from, to time.Time) ([]*AttendanceException, error)
	FindUnresolved(siteID uuid.UUID) ([]*AttendanceException, error)
	FindEscalated(ministryID uuid.UUID) ([]*AttendanceException, error)
}
