package application

import (
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

var (
	ErrDuplicateEvent    = errors.New("duplicate clock event detected")
	ErrInvalidEventType  = errors.New("invalid event type")
	ErrEventInFuture     = errors.New("event time cannot be in the future")
	ErrBiometricMismatch = errors.New("biometric verification failed")
	ErrDeviceNotTrusted  = errors.New("device is not trusted")
)

type ClockInCommand struct {
	EmployeeID     domain.EmployeeID
	MinistryID     uuid.UUID
	SiteID         uuid.UUID
	DeviceID       domain.DeviceID
	EventTime      time.Time
	Timezone       string
	BiometricData  []byte
	Latitude       *float64
	Longitude      *float64
	IPAddress      *string
	NodeID         uuid.UUID
}

type ClockInHandler struct {
	eventRepo     domain.ClockEventRepository
	shiftRepo     domain.ShiftRepository
	policyRepo    domain.AttendancePolicyRepository
	dupDetect     domain.DuplicateDetectionService
	biometricSvc  domain.BiometricVerificationService
	eventPub      EventPublisher
	syncQueue     SyncQueue
}

func NewClockInHandler(
	eventRepo domain.ClockEventRepository,
	shiftRepo domain.ShiftRepository,
	policyRepo domain.AttendancePolicyRepository,
	dupDetect domain.DuplicateDetectionService,
	biometricSvc domain.BiometricVerificationService,
	eventPub EventPublisher,
	syncQueue SyncQueue,
) *ClockInHandler {
	return &ClockInHandler{
		eventRepo:    eventRepo,
		shiftRepo:    shiftRepo,
		policyRepo:   policyRepo,
		dupDetect:    dupDetect,
		biometricSvc: biometricSvc,
		eventPub:     eventPub,
		syncQueue:    syncQueue,
	}
}

func (h *ClockInHandler) Handle(cmd ClockInCommand) (*domain.ClockEvent, error) {
	now := time.Now().UTC()
	if cmd.EventTime.After(now.Add(5 * time.Minute)) {
		return nil, ErrEventInFuture
	}

	duplicate, err := h.dupDetect.IsDuplicate(
		cmd.EmployeeID, cmd.DeviceID, domain.ClockIn, cmd.EventTime,
	)
	if err != nil {
		return nil, err
	}
	if duplicate {
		return nil, ErrDuplicateEvent
	}

	var biometricMatch *float64
	if cmd.BiometricData != nil {
		result, err := h.biometricSvc.Verify(cmd.EmployeeID, cmd.DeviceID, cmd.BiometricData)
		if err != nil {
			return nil, err
		}
		if !result.Matched {
			return nil, ErrBiometricMismatch
		}
		biometricMatch = &result.Confidence
	}

	offlineTS := domain.OfflineTimestamp{
		DeviceTime: cmd.EventTime,
		ServerTime: &now,
		Timezone:   cmd.Timezone,
	}

	event := domain.NewClockEvent(
		cmd.EmployeeID,
		cmd.MinistryID,
		cmd.SiteID,
		cmd.DeviceID,
		domain.ClockIn,
		offlineTS,
		biometricMatch,
		cmd.Latitude,
		cmd.Longitude,
		cmd.IPAddress,
		cmd.NodeID,
	)

	clockInEvent := &domain.ClockInCreated{
		BaseEvent: domain.BaseEvent{
			ID:      event.Identity(),
			Type:    "inwp.attendance.v1.clock-in.created",
			Version: "1.0.0",
			Time:    now,
			Src:     "/ministries/" + cmd.MinistryID.String() + "/sites/" + cmd.SiteID.String() + "/services/attendance-service",
			Mtd:     cmd.MinistryID,
			StID:    &cmd.SiteID,
			DevID:   uuidPtr(cmd.DeviceID),
			EmpID:   uuidPtr(cmd.EmployeeID),
		},
		EmployeeID: uuid.UUID(cmd.EmployeeID),
		DeviceID:   uuid.UUID(cmd.DeviceID),
		EventTime:  cmd.EventTime,
	}
	event.RaiseEvent(clockInEvent)

	if err := h.eventRepo.Save(event); err != nil {
		return nil, err
	}

	for _, e := range event.DomainEvents() {
		if err := h.eventPub.Publish(e); err != nil {
			return nil, err
		}
	}

	if err := h.syncQueue.Enqueue(event.Identity(), event.SyncMetadata()); err != nil {
		return nil, err
	}

	event.ClearEvents()
	return event, nil
}

type ClockOutCommand struct {
	EmployeeID     domain.EmployeeID
	MinistryID     uuid.UUID
	SiteID         uuid.UUID
	DeviceID       domain.DeviceID
	EventTime      time.Time
	Timezone       string
	BiometricData  []byte
	Latitude       *float64
	Longitude      *float64
	IPAddress      *string
	NodeID         uuid.UUID
}

type ClockOutHandler struct {
	eventRepo     domain.ClockEventRepository
	dupDetect     domain.DuplicateDetectionService
	biometricSvc  domain.BiometricVerificationService
	eventPub      EventPublisher
	syncQueue     SyncQueue
}

func NewClockOutHandler(
	eventRepo domain.ClockEventRepository,
	dupDetect domain.DuplicateDetectionService,
	biometricSvc domain.BiometricVerificationService,
	eventPub EventPublisher,
	syncQueue SyncQueue,
) *ClockOutHandler {
	return &ClockOutHandler{
		eventRepo:    eventRepo,
		dupDetect:    dupDetect,
		biometricSvc: biometricSvc,
		eventPub:     eventPub,
		syncQueue:    syncQueue,
	}
}

func (h *ClockOutHandler) Handle(cmd ClockOutCommand) (*domain.ClockEvent, error) {
	now := time.Now().UTC()
	if cmd.EventTime.After(now.Add(5 * time.Minute)) {
		return nil, ErrEventInFuture
	}

	duplicate, err := h.dupDetect.IsDuplicate(
		cmd.EmployeeID, cmd.DeviceID, domain.ClockOut, cmd.EventTime,
	)
	if err != nil {
		return nil, err
	}
	if duplicate {
		return nil, ErrDuplicateEvent
	}

	var biometricMatch *float64
	if cmd.BiometricData != nil {
		result, err := h.biometricSvc.Verify(cmd.EmployeeID, cmd.DeviceID, cmd.BiometricData)
		if err != nil {
			return nil, err
		}
		if !result.Matched {
			return nil, ErrBiometricMismatch
		}
		biometricMatch = &result.Confidence
	}

	offlineTS := domain.OfflineTimestamp{
		DeviceTime: cmd.EventTime,
		ServerTime: &now,
		Timezone:   cmd.Timezone,
	}

	event := domain.NewClockEvent(
		cmd.EmployeeID,
		cmd.MinistryID,
		cmd.SiteID,
		cmd.DeviceID,
		domain.ClockOut,
		offlineTS,
		biometricMatch,
		cmd.Latitude,
		cmd.Longitude,
		cmd.IPAddress,
		cmd.NodeID,
	)

	clockOutEvent := &domain.ClockOutCreated{
		BaseEvent: domain.BaseEvent{
			ID:      event.Identity(),
			Type:    "inwp.attendance.v1.clock-out.created",
			Version: "1.0.0",
			Time:    now,
			Src:     "/ministries/" + cmd.MinistryID.String() + "/sites/" + cmd.SiteID.String() + "/services/attendance-service",
			Mtd:     cmd.MinistryID,
			StID:    &cmd.SiteID,
			DevID:   uuidPtr(cmd.DevID),
			EmpID:   uuidPtr(cmd.EmployeeID),
		},
		EmployeeID: uuid.UUID(cmd.EmployeeID),
		DeviceID:   uuid.UUID(cmd.DeviceID),
		EventTime:  cmd.EventTime,
	}
	event.RaiseEvent(clockOutEvent)

	if err := h.eventRepo.Save(event); err != nil {
		return nil, err
	}

	for _, e := range event.DomainEvents() {
		if err := h.eventPub.Publish(e); err != nil {
			return nil, err
		}
	}

	if err := h.syncQueue.Enqueue(event.Identity(), event.SyncMetadata()); err != nil {
		return nil, err
	}

	event.ClearEvents()
	return event, nil
}

func uuidPtr(id interface{}) *uuid.UUID {
	switch v := id.(type) {
	case uuid.UUID:
		return &v
	case domain.EmployeeID:
		u := uuid.UUID(v)
		return &u
	case domain.DeviceID:
		u := uuid.UUID(v)
		return &u
	default:
		return nil
	}
}
