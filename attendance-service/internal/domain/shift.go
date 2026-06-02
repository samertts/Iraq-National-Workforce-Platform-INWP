package domain

import (
	"time"

	"github.com/google/uuid"
)

type Shift struct {
	id               uuid.UUID
	ministryID       uuid.UUID
	siteID           uuid.UUID
	name             string
	startTime        time.Time
	endTime          time.Time
	gracePeriod      time.Duration
	breakDuration    time.Duration
	applicableDays   []time.Weekday
	applicableGroups []uuid.UUID
	overtimePolicy   OvertimePolicy
	isActive         bool
	version          int64
	domainEvents     []DomainEvent
}

func NewShift(
	ministryID uuid.UUID,
	siteID uuid.UUID,
	name string,
	startTime time.Time,
	endTime time.Time,
	gracePeriod time.Duration,
	breakDuration time.Duration,
	applicableDays []time.Weekday,
	overtimePolicy OvertimePolicy,
) *Shift {
	return &Shift{
		id:             uuid.New(),
		ministryID:     ministryID,
		siteID:         siteID,
		name:           name,
		startTime:      startTime,
		endTime:        endTime,
		gracePeriod:    gracePeriod,
		breakDuration:  breakDuration,
		applicableDays: applicableDays,
		overtimePolicy: overtimePolicy,
		isActive:       true,
		version:        1,
		domainEvents:   make([]DomainEvent, 0),
	}
}

func (s *Shift) Identity() uuid.UUID        { return s.id }
func (s *Shift) Version() int64              { return s.version }
func (s *Shift) DomainEvents() []DomainEvent { return s.domainEvents }
func (s *Shift) ClearEvents()                { s.domainEvents = nil }
func (s *Shift) MinistryID() uuid.UUID       { return s.ministryID }
func (s *Shift) SiteID() uuid.UUID            { return s.siteID }
func (s *Shift) Name() string                 { return s.name }
func (s *Shift) StartTime() time.Time         { return s.startTime }
func (s *Shift) EndTime() time.Time           { return s.endTime }
func (s *Shift) GracePeriod() time.Duration   { return s.gracePeriod }
func (s *Shift) BreakDuration() time.Duration { return s.breakDuration }
func (s *Shift) IsActive() bool               { return s.isActive }
func (s *Shift) OvertimePolicy() OvertimePolicy { return s.overtimePolicy }

func (s *Shift) Deactivate() {
	s.isActive = false
	s.version++
}

func (s *Shift) Modify(
	name string,
	startTime time.Time,
	endTime time.Time,
	gracePeriod time.Duration,
	breakDuration time.Duration,
	overtimePolicy OvertimePolicy,
) {
	s.name = name
	s.startTime = startTime
	s.endTime = endTime
	s.gracePeriod = gracePeriod
	s.breakDuration = breakDuration
	s.overtimePolicy = overtimePolicy
	s.version++
}

func (s *Shift) RaiseEvent(event DomainEvent) {
	s.domainEvents = append(s.domainEvents, event)
}

type ShiftRepository interface {
	Save(shift *Shift) error
	FindByID(id uuid.UUID) (*Shift, error)
	FindBySite(siteID uuid.UUID) ([]*Shift, error)
	FindActiveBySite(siteID uuid.UUID) (*Shift, error)
	Delete(id uuid.UUID) error
}
