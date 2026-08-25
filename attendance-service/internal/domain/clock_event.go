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
		id:             uuid.New(),
		employeeID:     employeeID,
		ministryID:     ministryID,
		siteID:         siteID,
		deviceID:       deviceID,
		eventType:      eventType,
		eventTime:      eventTime,
		recordedAt:     now,
		biometricMatch: biometricMatch,
		latitude:       latitude,
		longitude:      longitude,
		ipAddress:      ipAddress,
		syncMetadata: SyncMetadata{
			SyncID:       uuid.New(),
			SourceNodeID: nodeID,
			Status:       SyncStatusLocalOnly,
			Version:      1,
		},
		domainEvents: make([]DomainEvent, 0),
	}
}

func (c *ClockEvent) Identity() uuid.UUID         { return c.id }
func (c *ClockEvent) Version() int64              { return c.syncMetadata.Version }
func (c *ClockEvent) DomainEvents() []DomainEvent { return c.domainEvents }
func (c *ClockEvent) ClearEvents()                { c.domainEvents = nil }
func (c *ClockEvent) MinistryID() uuid.UUID       { return c.ministryID }
func (c *ClockEvent) SiteID() uuid.UUID           { return c.siteID }
func (c *ClockEvent) EmployeeID() EmployeeID      { return c.employeeID }
func (c *ClockEvent) DeviceID() DeviceID          { return c.deviceID }
func (c *ClockEvent) EventType() ClockEventType   { return c.eventType }
func (c *ClockEvent) EventTime() OfflineTimestamp { return c.eventTime }
func (c *ClockEvent) RecordedAt() time.Time       { return c.recordedAt }
func (c *ClockEvent) SyncMetadata() SyncMetadata  { return c.syncMetadata }
func (c *ClockEvent) BiometricMatch() *float64    { return c.biometricMatch }
func (c *ClockEvent) Latitude() *float64          { return c.latitude }
func (c *ClockEvent) Longitude() *float64         { return c.longitude }
func (c *ClockEvent) IPAddress() *string          { return c.ipAddress }

// SyncPayload is the canonical JSON representation placed in the outbox.
type SyncPayload struct {
	ID             uuid.UUID        `json:"id"`
	EmployeeID     EmployeeID       `json:"employee_id"`
	MinistryID     uuid.UUID        `json:"ministry_id"`
	SiteID         uuid.UUID        `json:"site_id"`
	DeviceID       DeviceID         `json:"device_id"`
	EventType      ClockEventType   `json:"event_type"`
	EventTime      OfflineTimestamp `json:"event_time"`
	RecordedAt     time.Time        `json:"recorded_at"`
	BiometricMatch *float64         `json:"biometric_match,omitempty"`
	Latitude       *float64         `json:"latitude,omitempty"`
	Longitude      *float64         `json:"longitude,omitempty"`
	IPAddress      *string          `json:"ip_address,omitempty"`
	SyncMetadata   SyncMetadata     `json:"sync_metadata"`
}

func (c *ClockEvent) SyncPayload() SyncPayload {
	return SyncPayload{
		ID: c.id, EmployeeID: c.employeeID, MinistryID: c.ministryID,
		SiteID: c.siteID, DeviceID: c.deviceID, EventType: c.eventType,
		EventTime: c.eventTime, RecordedAt: c.recordedAt,
		BiometricMatch: c.biometricMatch, Latitude: c.latitude,
		Longitude: c.longitude, IPAddress: c.ipAddress,
		SyncMetadata: c.syncMetadata,
	}
}

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
