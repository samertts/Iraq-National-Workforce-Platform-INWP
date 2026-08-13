package gula

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

type Envelope struct {
	EventID        string          `json:"event_id"`
	EventType      string          `json:"event_type"`
	SchemaVersion  int             `json:"schema_version"`
	SourceService  string          `json:"source_service"`
	TenantID       string          `json:"tenant_id"`
	OccurredAt     string          `json:"occurred_at"`
	ActorID        string          `json:"actor_id"`
	EntityID       string          `json:"entity_id"`
	CorrelationID  string          `json:"correlation_id"`
	IdempotencyKey string          `json:"idempotency_key"`
	Payload        json.RawMessage `json:"payload"`
}

type HTTPClient interface {
	Do(req *http.Request) (*http.Response, error)
}

type Publisher struct {
	BaseURL     string
	AccessToken string
	Client      HTTPClient
	MaxRetries  int
	Backoff     time.Duration
}

func (p *Publisher) Publish(event domain.DomainEvent) error {
	if event == nil {
		return fmt.Errorf("event is required")
	}
	if strings.TrimSpace(p.BaseURL) == "" || strings.TrimSpace(p.AccessToken) == "" {
		return fmt.Errorf("GULA base URL and access token are required")
	}
	payload, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("marshal workforce event: %w", err)
	}
	schemaVersion := 1
	if parts := strings.Split(event.EventVersion(), "v"); len(parts) > 1 {
		if parsed, parseErr := strconv.Atoi(parts[len(parts)-1]); parseErr == nil && parsed > 0 {
			schemaVersion = parsed
		}
	}
	eventType := "workforce.attendance.recorded"
	actorID := event.MinistryID().String()
	var actorPayload struct {
		EmployeeID string `json:"employee_id"`
	}
	if err := json.Unmarshal(payload, &actorPayload); err == nil && actorPayload.EmployeeID != "" {
		actorID = actorPayload.EmployeeID
	}
	envelope := Envelope{
		EventID:        event.EventID().String(),
		EventType:      eventType,
		SchemaVersion:  schemaVersion,
		SourceService:  "inwp",
		TenantID:       event.MinistryID().String(),
		OccurredAt:     event.OccurredAt().UTC().Format(time.RFC3339Nano),
		ActorID:        actorID,
		EntityID:       event.EventID().String(),
		CorrelationID:  event.EventID().String(),
		IdempotencyKey: event.EventID().String() + ":" + event.EventVersion(),
		Payload:        payload,
	}
	body, err := json.Marshal(envelope)
	if err != nil {
		return fmt.Errorf("marshal GULA envelope: %w", err)
	}
	client := p.Client
	if client == nil {
		client = http.DefaultClient
	}
	retries := p.MaxRetries
	if retries < 1 {
		retries = 1
	}
	backoff := p.Backoff
	if backoff <= 0 {
		backoff = time.Second
	}
	var lastErr error
	for attempt := 0; attempt < retries; attempt++ {
		req, reqErr := http.NewRequest(http.MethodPost, strings.TrimRight(p.BaseURL, "/")+"/integrations/events", bytes.NewReader(body))
		if reqErr != nil {
			return reqErr
		}
		req.Header.Set("Authorization", "Bearer "+p.AccessToken)
		req.Header.Set("Content-Type", "application/json")
		resp, doErr := client.Do(req)
		if doErr == nil && resp.StatusCode >= 200 && resp.StatusCode < 300 {
			_ = resp.Body.Close()
			return nil
		}
		if resp != nil {
			_ = resp.Body.Close()
			lastErr = fmt.Errorf("GULA returned HTTP %d", resp.StatusCode)
		} else {
			lastErr = doErr
		}
		if attempt+1 < retries {
			time.Sleep(backoff * time.Duration(1<<attempt))
		}
	}
	return fmt.Errorf("publish workforce event after %d attempts: %w", retries, lastErr)
}
