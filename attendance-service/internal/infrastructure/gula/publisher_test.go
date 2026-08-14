package gula

import (
	"io"
	"net/http"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

type fakeClient struct {
	requests int
	body     string
}

func (f *fakeClient) Do(req *http.Request) (*http.Response, error) {
	f.requests++
	data, _ := io.ReadAll(req.Body)
	f.body = string(data)
	return &http.Response{StatusCode: 202, Body: io.NopCloser(strings.NewReader("{}"))}, nil
}

type testEvent struct {
	domain.BaseEvent
	Name string `json:"name"`
}

func TestPublisherBuildsCanonicalEnvelope(t *testing.T) {
	client := &fakeClient{}
	ministry := uuid.New()
	event := testEvent{BaseEvent: domain.BaseEvent{ID: uuid.New(), Type: "inwp.attendance.v1.clock-in.created", Version: "v1", Time: time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC), Src: "attendance-service", Mtd: ministry}, Name: "clock-in"}
	publisher := Publisher{BaseURL: "https://gula.example", AccessToken: "token", Client: client, MaxRetries: 1}
	if err := publisher.Publish(event); err != nil {
		t.Fatal(err)
	}
	if client.requests != 1 {
		t.Fatalf("expected one request, got %d", client.requests)
	}
	if !strings.Contains(client.body, `"event_type":"workforce.attendance.recorded"`) {
		t.Fatal("missing canonical event type")
	}
	if !strings.Contains(client.body, ministry.String()) {
		t.Fatal("missing tenant")
	}
	if !strings.Contains(client.body, `"schema_version":1`) {
		t.Fatal("missing schema version")
	}
}
