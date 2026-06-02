package application

import (
	"time"

	"github.com/google/uuid"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

type SetAttendancePolicyCommand struct {
	MinistryID    uuid.UUID
	SiteID        *uuid.UUID
	Name          string
	Rules         domain.AttendanceRuleSet
	EffectiveFrom time.Time
	ApprovedBy    uuid.UUID
}

type SetAttendancePolicyHandler struct {
	policyRepo domain.AttendancePolicyRepository
	eventPub   EventPublisher
}

func NewSetAttendancePolicyHandler(
	policyRepo domain.AttendancePolicyRepository,
	eventPub EventPublisher,
) *SetAttendancePolicyHandler {
	return &SetAttendancePolicyHandler{
		policyRepo: policyRepo,
		eventPub:   eventPub,
	}
}

func (h *SetAttendancePolicyHandler) Handle(cmd SetAttendancePolicyCommand) (*domain.AttendancePolicy, error) {
	siteID := cmd.SiteID

	existing, err := h.policyRepo.FindActiveBySite(*siteID)
	if err == nil && existing != nil {
		existing.Supersede(uuid.Nil, cmd.EffectiveFrom.Add(-time.Second))
		if err := h.policyRepo.Save(existing); err != nil {
			return nil, err
		}
	}

	policy := domain.NewAttendancePolicy(
		cmd.MinistryID,
		siteID,
		cmd.Name,
		cmd.Rules,
		cmd.EffectiveFrom,
		cmd.ApprovedBy,
	)

	policy.RaiseEvent(&domain.PolicyCreated{
		BaseEvent: domain.BaseEvent{
			ID:      policy.Identity(),
			Type:    "inwp.attendance.v1.policy.created",
			Version: "1.0.0",
			Time:    time.Now().UTC(),
		},
		PolicyID:      policy.Identity(),
		EffectiveFrom: cmd.EffectiveFrom,
	})

	if err := h.policyRepo.Save(policy); err != nil {
		return nil, err
	}

	for _, e := range policy.DomainEvents() {
		if err := h.eventPub.Publish(e); err != nil {
			return nil, err
		}
	}

	policy.ClearEvents()
	return policy, nil
}
