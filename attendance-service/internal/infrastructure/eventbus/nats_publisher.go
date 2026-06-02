package eventbus

import (
	"encoding/json"
	"time"

	"github.com/google/uuid"
	"github.com/nats-io/nats.go"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

type NATSPublisher struct {
	conn *nats.Conn
}

func NewNATSPublisher(conn *nats.Conn) *NATSPublisher {
	return &NATSPublisher{conn: conn}
}

type CloudEvent struct {
	SpecVersion     string          `json:"specversion"`
	ID              string          `json:"id"`
	Source          string          `json:"source"`
	Type            string          `json:"type"`
	DataContentType string          `json:"datacontenttype"`
	Subject         string          `json:"subject"`
	Time            string          `json:"time"`
	DataSchema      string          `json:"dataschema"`
	MinistryID      string          `json:"ministry_id"`
	SiteID          *string         `json:"site_id,omitempty"`
	DeviceID        *string         `json:"device_id,omitempty"`
	UserID          *string         `json:"user_id,omitempty"`
	OfflineGenerated bool           `json:"offline_generated"`
	Data            json.RawMessage `json:"data"`
}

func (p *NATSPublisher) Publish(event domain.DomainEvent) error {
	if p.conn == nil {
		return nil
	}

	payload, err := json.Marshal(event)
	if err != nil {
		return err
	}

	var siteID, deviceID, userID *string
	if sid := event.SiteID(); sid != nil {
		s := sid.String()
		siteID = &s
	}

	ce := CloudEvent{
		SpecVersion:      "1.0",
		ID:               event.EventID().String(),
		Source:           event.Source(),
		Type:             event.EventType(),
		DataContentType:  "application/json",
		Subject:          "event:" + event.EventID().String(),
		Time:             event.OccurredAt().Format(time.RFC3339Nano),
		DataSchema:       event.EventType() + ":v1",
		MinistryID:       event.MinistryID().String(),
		SiteID:           siteID,
		DeviceID:         deviceID,
		UserID:           userID,
		OfflineGenerated: false,
		Data:             payload,
	}

	data, err := json.Marshal(ce)
	if err != nil {
		return err
	}

	subject := event.EventType()
	return p.conn.Publish(subject, data)
}

type NoopPublisher struct{}

func (p *NoopPublisher) Publish(event domain.DomainEvent) error {
	return nil
}
