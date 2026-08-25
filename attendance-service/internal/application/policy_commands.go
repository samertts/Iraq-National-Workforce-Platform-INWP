package application

import (
	"errors"
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

func NewSetAttendancePolicyHandler(policyRepo domain.AttendancePolicyRepository, eventPub EventPublisher) *SetAttendancePolicyHandler {
	return &SetAttendancePolicyHandler{policyRepo: policyRepo, eventPub: eventPub}
}

func (h *SetAttendancePolicyHandler) Handle(cmd SetAttendancePolicyCommand) (*domain.AttendancePolicy, error) {
	if cmd.MinistryID == uuid.Nil || cmd.ApprovedBy == uuid.Nil {
		return nil, errors.New("ministry_id and approved_by are required")
	}
	if cmd.Name == "" {
		return nil, errors.New("policy name is required")
	}
	if cmd.EffectiveFrom.IsZero() {
		return nil, errors.New("effective_from is required")
	}

	var existing *domain.AttendancePolicy
	var err error
	if cmd.SiteID != nil {
		existing, err = h.policyRepo.FindActiveBySite(*cmd.SiteID)
	} else {
		existing, err = h.policyRepo.FindActiveByMinistry(cmd.MinistryID)
	}
	if err != nil {
		return nil, err
	}

	policy := domain.NewAttendancePolicy(cmd.MinistryID, cmd.SiteID, cmd.Name, cmd.Rules, cmd.EffectiveFrom, cmd.ApprovedBy)
	policy.RaiseEvent(&domain.PolicyCreated{
		BaseEvent: domain.BaseEvent{
			ID: policy.Identity(), Type: "inwp.attendance.v1.policy.created", Version: "1.0.0",
			Time: time.Now().UTC(), Src: "/ministries/" + cmd.MinistryID.String() + "/services/attendance-service",
			Mtd: cmd.MinistryID, StID: cmd.SiteID,
		},
		PolicyID: policy.Identity(), EffectiveFrom: cmd.EffectiveFrom,
	})
	if err := h.policyRepo.Save(policy); err != nil {
		return nil, err
	}

	if existing != nil {
		existing.Supersede(policy.Identity(), cmd.EffectiveFrom.Add(-time.Second))
		existing.RaiseEvent(&domain.PolicySuperseded{
			BaseEvent: domain.BaseEvent{
				ID: existing.Identity(), Type: "inwp.attendance.v1.policy.superseded", Version: "1.0.0", Time: time.Now().UTC(),
				Mtd: cmd.MinistryID, StID: cmd.SiteID,
			},
			PolicyID: existing.Identity(), SupersededBy: policy.Identity(), SupersededAt: time.Now().UTC(),
		})
		if err := h.policyRepo.Save(existing); err != nil {
			return nil, err
		}
		for _, event := range existing.DomainEvents() {
			if err := h.eventPub.Publish(event); err != nil {
				return nil, err
			}
		}
		existing.ClearEvents()
	}

	for _, event := range policy.DomainEvents() {
		if err := h.eventPub.Publish(event); err != nil {
			return nil, err
		}
	}
	policy.ClearEvents()
	return policy, nil
}
