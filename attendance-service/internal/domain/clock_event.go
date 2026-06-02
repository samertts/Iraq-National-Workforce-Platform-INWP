package domain

import (
	"time"

	"github.com/google/uuid"
)

type ClockEvent struct {
	id             uuid.UUID
	employeeID     EmployeeID
	ministryID     uuid.UUID
	siteID         uuid.UUID
	deviceID       DeviceID
	eventType      ClockEventType
	eventTime      OfflineTimestamp
	recordedAt     time.Time
	biometricMatch *float64
	latitude       *float64
	longitude      *float64
	ipAddress      *string
	syncMetadata   SyncMetadata
	domainEvents   []DomainEvent
}

func NewClockEvent(
	employeeID EmployeeID,
	ministryID uuid.UUID,
	siteID uuid.UUID,
	deviceID DeviceID,
	eventType ClockEventType,
	eventTime OfflineTimestamp,
	biometricMatch *float64,
	latitude *float64,
	longitude *float64,
	ipAddress *string,
	nodeID uuid.UUID,
) *ClockEvent {
	now := time.Now().UTC()
	return &ClockEvent{
		id:         uuid.New(),
		employeeID: employeeID,
		ministryID: ministryID,
		siteID:     siteID,
		deviceID:   deviceID,
		eventType:  eventType,
		eventTime:  eventTime,
		recordedAt: now,
		biometricMatch: biometricMatch,
		latitude:   latitude,
		longitude:  longitude,
		ipAddress:  ipAddress,
		syncMetadata: SyncMetadata{
			SyncID:       uuid.New(),
			SourceNodeID: nodeID,
			Status:       SyncStatusLocalOnly,
			Version:      1,
		},
		domainEvents: make([]DomainEvent, 0),
	}
}

func (c *ClockEvent) Identity() uuid.UUID        { return c.id }
func (c *ClockEvent) Version() int64              { return c.syncMetadata.Version }
func (c *ClockEvent) DomainEvents() []DomainEvent { return c.domainEvents }
func (c *ClockEvent) ClearEvents()                { c.domainEvents = nil }
func (c *ClockEvent) MinistryID() uuid.UUID       { return c.ministryID }
func (c *ClockEvent) SiteID() uuid.UUID            { return c.siteID }
func (c *ClockEvent) EmployeeID() EmployeeID      { return c.employeeID }
func (c *ClockEvent) DeviceID() DeviceID           { return c.deviceID }
func (c *ClockEvent) EventType() ClockEventType   { return c.eventType }
func (c *ClockEvent) EventTime() OfflineTimestamp  { return c.eventTime }
func (c *ClockEvent) RecordedAt() time.Time        { return c.recordedAt }
func (c *ClockEvent) SyncMetadata() SyncMetadata   { return c.syncMetadata }

func (c *ClockEvent) RaiseEvent(event DomainEvent) {
	c.domainEvents = append(c.domainEvents, event)
}

type ClockEventRepository interface {
	Save(event *ClockEvent) error
	SaveBatch(events []*ClockEvent) error
	FindByID(id uuid.UUID) (*ClockEvent, error)
	FindByEmployee(employeeID EmployeeID, from, to time.Time) ([]*ClockEvent, error)
	FindByDevice(deviceID DeviceID, from, to time.Time) ([]*ClockEvent, error)
	FindBySite(siteID uuid.UUID, from, to time.Time) ([]*ClockEvent, error)
	FindUnsynced(nodeID uuid.UUID) ([]*ClockEvent, error)
	ExistsDuplicate(employeeID EmployeeID, deviceID DeviceID, eventType ClockEventType, eventTime time.Time) (bool, error)
}

type DuplicateDetectionService interface {
	IsDuplicate(employeeID EmployeeID, deviceID DeviceID, eventType ClockEventType, eventTime time.Time) (bool, error)
	FindNearDuplicates(employeeID EmployeeID, timeWindow time.Duration) ([]*ClockEvent, error)
}

type BiometricVerificationService interface {
	Verify(employeeID EmployeeID, deviceID DeviceID, biometricData []byte) (BiometricResult, error)
	EnrollTemplate(employeeID EmployeeID, deviceID DeviceID, template []byte) error
}

type BiometricResult struct {
	Matched      bool    `json:"matched"`
	Confidence   float64 `json:"confidence"`
	TemplateHash []byte  `json:"template_hash"`
}

type AttendanceCalculationService interface {
	CalculateWorkedHours(employeeID EmployeeID, date time.Time) (float64, error)
	CalculateOvertime(employeeID EmployeeID, from, to time.Time) (float64, error)
	DetectMissingClocks(siteID uuid.UUID, date time.Time) ([]*AttendanceException, error)
	VerifyBreakCompliance(employeeID EmployeeID, date time.Time) (bool, error)
}
