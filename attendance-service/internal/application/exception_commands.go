package application

import (
	"time"

	"github.com/google/uuid"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

type JustifyExceptionCommand struct {
	ExceptionID uuid.UUID
	Reason      string
	Type        domain.JustificationType
}

type JustifyExceptionHandler struct {
	exceptionRepo domain.AttendanceExceptionRepository
	eventPub      EventPublisher
}

func NewJustifyExceptionHandler(
	exceptionRepo domain.AttendanceExceptionRepository,
	eventPub EventPublisher,
) *JustifyExceptionHandler {
	return &JustifyExceptionHandler{
		exceptionRepo: exceptionRepo,
		eventPub:      eventPub,
	}
}

func (h *JustifyExceptionHandler) Handle(cmd JustifyExceptionCommand) error {
	exception, err := h.exceptionRepo.FindByID(cmd.ExceptionID)
	if err != nil {
		return err
	}

	exception.Justify(cmd.Reason, cmd.Type)
	exception.RaiseEvent(&domain.ExceptionJustified{
		BaseEvent: domain.BaseEvent{
			ID:      exception.Identity(),
			Type:    "inwp.attendance.v1.exception.justified",
			Version: "1.0.0",
			Time:    time.Now().UTC(),
		},
		ExceptionID:      exception.Identity(),
		JustificationType: cmd.Type,
	})

	if err := h.exceptionRepo.Save(exception); err != nil {
		return err
	}

	for _, e := range exception.DomainEvents() {
		if err := h.eventPub.Publish(e); err != nil {
			return err
		}
	}

	exception.ClearEvents()
	return nil
}

type ResolveExceptionCommand struct {
	ExceptionID uuid.UUID
	ResolvedBy  uuid.UUID
}

type ResolveExceptionHandler struct {
	exceptionRepo domain.AttendanceExceptionRepository
	eventPub      EventPublisher
}

func NewResolveExceptionHandler(
	exceptionRepo domain.AttendanceExceptionRepository,
	eventPub EventPublisher,
) *ResolveExceptionHandler {
	return &ResolveExceptionHandler{
		exceptionRepo: exceptionRepo,
		eventPub:      eventPub,
	}
}

func (h *ResolveExceptionHandler) Handle(cmd ResolveExceptionCommand) error {
	exception, err := h.exceptionRepo.FindByID(cmd.ExceptionID)
	if err != nil {
		return err
	}

	exception.Resolve(cmd.ResolvedBy)
	exception.RaiseEvent(&domain.ExceptionResolved{
		BaseEvent: domain.BaseEvent{
			ID:      exception.Identity(),
			Type:    "inwp.attendance.v1.exception.resolved",
			Version: "1.0.0",
			Time:    time.Now().UTC(),
		},
		ExceptionID: exception.Identity(),
		ResolvedBy:  cmd.ResolvedBy,
	})

	if err := h.exceptionRepo.Save(exception); err != nil {
		return err
	}

	for _, e := range exception.DomainEvents() {
		if err := h.eventPub.Publish(e); err != nil {
			return err
		}
	}

	exception.ClearEvents()
	return nil
}
