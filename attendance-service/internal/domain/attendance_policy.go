package domain

import (
	"time"

	"github.com/google/uuid"
)

type AttendancePolicy struct {
	id              uuid.UUID
	ministryID      uuid.UUID
	siteID          *uuid.UUID
	name            string
	rules           AttendanceRuleSet
	effectiveFrom   time.Time
	effectiveTo     *time.Time
	version         int
	supersedes      *uuid.UUID
	approvedBy      uuid.UUID
	createdAt       time.Time
	domainEvents    []DomainEvent
}

func NewAttendancePolicy(
	ministryID uuid.UUID,
	siteID *uuid.UUID,
	name string,
	rules AttendanceRuleSet,
	effectiveFrom time.Time,
	approvedBy uuid.UUID,
) *AttendancePolicy {
	return &AttendancePolicy{
		id:            uuid.New(),
		ministryID:    ministryID,
		siteID:        siteID,
		name:          name,
		rules:         rules,
		effectiveFrom: effectiveFrom,
		version:       1,
		approvedBy:    approvedBy,
		createdAt:     time.Now().UTC(),
		domainEvents:  make([]DomainEvent, 0),
	}
}

func (p *AttendancePolicy) Identity() uuid.UUID        { return p.id }
func (p *AttendancePolicy) Version() int64              { return int64(p.version) }
func (p *AttendancePolicy) DomainEvents() []DomainEvent { return p.domainEvents }
func (p *AttendancePolicy) ClearEvents()                { p.domainEvents = nil }
func (p *AttendancePolicy) MinistryID() uuid.UUID       { return p.ministryID }
func (p *AttendancePolicy) SiteID() *uuid.UUID          { return p.siteID }
func (p *AttendancePolicy) Name() string                { return p.name }
func (p *AttendancePolicy) Rules() AttendanceRuleSet    { return p.rules }
func (p *AttendancePolicy) EffectiveFrom() time.Time    { return p.effectiveFrom }
func (p *AttendancePolicy) EffectiveTo() *time.Time     { return p.effectiveTo }
func (p *AttendancePolicy) Supersedes() *uuid.UUID      { return p.supersedes }

func (p *AttendancePolicy) Supersede(newPolicyID uuid.UUID, effectiveTo time.Time) {
	p.effectiveTo = &effectiveTo
	p.supersedes = &newPolicyID
	p.version++
}

func (p *AttendancePolicy) RaiseEvent(event DomainEvent) {
	p.domainEvents = append(p.domainEvents, event)
}

type AttendancePolicyRepository interface {
	Save(policy *AttendancePolicy) error
	FindByID(id uuid.UUID) (*AttendancePolicy, error)
	FindActiveBySite(siteID uuid.UUID) (*AttendancePolicy, error)
	FindActiveByMinistry(ministryID uuid.UUID) (*AttendancePolicy, error)
	FindHistoryBySite(siteID uuid.UUID) ([]*AttendancePolicy, error)
}
