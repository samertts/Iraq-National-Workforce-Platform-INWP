package application

import (
	"time"

	"github.com/google/uuid"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

type CreateShiftCommand struct {
	MinistryID     uuid.UUID
	SiteID         uuid.UUID
	Name           string
	StartTime      time.Time
	EndTime        time.Time
	GracePeriod    time.Duration
	BreakDuration  time.Duration
	ApplicableDays []time.Weekday
	OvertimePolicy domain.OvertimePolicy
}

type CreateShiftHandler struct {
	shiftRepo domain.ShiftRepository
	eventPub  EventPublisher
}

func NewCreateShiftHandler(shiftRepo domain.ShiftRepository, eventPub EventPublisher) *CreateShiftHandler {
	return &CreateShiftHandler{
		shiftRepo: shiftRepo,
		eventPub:  eventPub,
	}
}

func (h *CreateShiftHandler) Handle(cmd CreateShiftCommand) (*domain.Shift, error) {
	shift := domain.NewShift(
		cmd.MinistryID,
		cmd.SiteID,
		cmd.Name,
		cmd.StartTime,
		cmd.EndTime,
		cmd.GracePeriod,
		cmd.BreakDuration,
		cmd.ApplicableDays,
		cmd.OvertimePolicy,
	)

	shiftEvent := &domain.ShiftCreated{
		BaseEvent: domain.BaseEvent{
			ID:      shift.Identity(),
			Type:    "inwp.attendance.v1.shift.created",
			Version: "1.0.0",
			Time:    time.Now().UTC(),
			Src:     "/ministries/" + cmd.MinistryID.String() + "/sites/" + cmd.SiteID.String() + "/services/attendance-service",
			Mtd:     cmd.MinistryID,
			StID:    &cmd.SiteID,
		},
		ShiftID: shift.Identity(),
		Name:    cmd.Name,
	}
	shift.RaiseEvent(shiftEvent)

	if err := h.shiftRepo.Save(shift); err != nil {
		return nil, err
	}

	for _, e := range shift.DomainEvents() {
		if err := h.eventPub.Publish(e); err != nil {
			return nil, err
		}
	}

	shift.ClearEvents()
	return shift, nil
}

type DeactivateShiftCommand struct {
	ShiftID uuid.UUID
}

type DeactivateShiftHandler struct {
	shiftRepo domain.ShiftRepository
	eventPub  EventPublisher
}

func NewDeactivateShiftHandler(shiftRepo domain.ShiftRepository, eventPub EventPublisher) *DeactivateShiftHandler {
	return &DeactivateShiftHandler{
		shiftRepo: shiftRepo,
		eventPub:  eventPub,
	}
}

func (h *DeactivateShiftHandler) Handle(cmd DeactivateShiftCommand) error {
	shift, err := h.shiftRepo.FindByID(cmd.ShiftID)
	if err != nil {
		return err
	}

	shift.Deactivate()
	shift.RaiseEvent(&domain.ShiftDeactivated{
		BaseEvent: domain.BaseEvent{
			ID:      shift.Identity(),
			Type:    "inwp.attendance.v1.shift.deactivated",
			Version: "1.0.0",
			Time:    time.Now().UTC(),
		},
		ShiftID:       shift.Identity(),
		DeactivatedAt: time.Now().UTC(),
	})

	if err := h.shiftRepo.Save(shift); err != nil {
		return err
	}

	for _, e := range shift.DomainEvents() {
		if err := h.eventPub.Publish(e); err != nil {
			return err
		}
	}

	shift.ClearEvents()
	return nil
}
