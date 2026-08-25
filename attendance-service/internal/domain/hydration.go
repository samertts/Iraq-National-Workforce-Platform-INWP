package domain

import (
	"time"

	"github.com/google/uuid"
)

// RehydrateClockEvent rebuilds an aggregate from its persisted representation.
// It is intentionally kept in the domain package so infrastructure cannot
// mutate private aggregate state directly.
func RehydrateClockEvent(
	id uuid.UUID,
	employeeID EmployeeID,
	ministryID uuid.UUID,
	siteID uuid.UUID,
	deviceID DeviceID,
	eventType ClockEventType,
	eventTime OfflineTimestamp,
	recordedAt time.Time,
	biometricMatch *float64,
	latitude *float64,
	longitude *float64,
	ipAddress *string,
	syncMetadata SyncMetadata,
) *ClockEvent {
	return &ClockEvent{
		id:             id,
		employeeID:     employeeID,
		ministryID:     ministryID,
		siteID:         siteID,
		deviceID:       deviceID,
		eventType:      eventType,
		eventTime:      eventTime,
		recordedAt:     recordedAt,
		biometricMatch: biometricMatch,
		latitude:       latitude,
		longitude:      longitude,
		ipAddress:      ipAddress,
		syncMetadata:   syncMetadata,
		domainEvents:   make([]DomainEvent, 0),
	}
}

// RehydrateShift rebuilds a persisted shift aggregate.
func RehydrateShift(
	id uuid.UUID,
	ministryID uuid.UUID,
	siteID uuid.UUID,
	name string,
	startTime time.Time,
	endTime time.Time,
	gracePeriod time.Duration,
	breakDuration time.Duration,
	overtimePolicy OvertimePolicy,
	isActive bool,
	version int64,
) *Shift {
	return &Shift{
		id:             id,
		ministryID:     ministryID,
		siteID:         siteID,
		name:           name,
		startTime:      startTime,
		endTime:        endTime,
		gracePeriod:    gracePeriod,
		breakDuration:  breakDuration,
		overtimePolicy: overtimePolicy,
		isActive:       isActive,
		version:        version,
		domainEvents:   make([]DomainEvent, 0),
	}
}

// RehydrateAttendancePolicy rebuilds a persisted attendance policy.
func RehydrateAttendancePolicy(
	id uuid.UUID,
	ministryID uuid.UUID,
	siteID *uuid.UUID,
	name string,
	rules AttendanceRuleSet,
	effectiveFrom time.Time,
	effectiveTo *time.Time,
	version int,
	supersedes *uuid.UUID,
	approvedBy uuid.UUID,
	createdAt time.Time,
) *AttendancePolicy {
	return &AttendancePolicy{
		id:            id,
		ministryID:    ministryID,
		siteID:        siteID,
		name:          name,
		rules:         rules,
		effectiveFrom: effectiveFrom,
		effectiveTo:   effectiveTo,
		version:       version,
		supersedes:    supersedes,
		approvedBy:    approvedBy,
		createdAt:     createdAt,
		domainEvents:  make([]DomainEvent, 0),
	}
}

// RehydrateAttendanceException rebuilds a persisted attendance exception.
func RehydrateAttendanceException(
	id uuid.UUID,
	employeeID EmployeeID,
	clockEventID *uuid.UUID,
	exceptionType ExceptionType,
	severity ExceptionSeverity,
	description string,
	occurredAt time.Time,
	detectedAt time.Time,
	justification *Justification,
	resolvedAt *time.Time,
	resolvedBy *uuid.UUID,
	escalatedAt *time.Time,
) *AttendanceException {
	return &AttendanceException{
		id:            id,
		employeeID:    employeeID,
		clockEventID:  clockEventID,
		exceptionType: exceptionType,
		severity:      severity,
		description:   description,
		occurredAt:    occurredAt,
		detectedAt:    detectedAt,
		justification: justification,
		resolvedAt:    resolvedAt,
		resolvedBy:    resolvedBy,
		escalatedAt:   escalatedAt,
		domainEvents:  make([]DomainEvent, 0),
	}
}
