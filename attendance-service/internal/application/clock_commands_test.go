package application

import (
	"encoding/json"
	"errors"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

type fakeClockEventRepo struct{ saved []*domain.ClockEvent }

func (f *fakeClockEventRepo) Save(event *domain.ClockEvent) error {
	f.saved = append(f.saved, event)
	return nil
}
func (f *fakeClockEventRepo) SaveBatch(events []*domain.ClockEvent) error {
	f.saved = append(f.saved, events...)
	return nil
}
func (f *fakeClockEventRepo) FindByID(uuid.UUID) (*domain.ClockEvent, error) { return nil, nil }
func (f *fakeClockEventRepo) FindByEmployee(domain.EmployeeID, time.Time, time.Time) ([]*domain.ClockEvent, error) {
	return nil, nil
}
func (f *fakeClockEventRepo) FindByDevice(domain.DeviceID, time.Time, time.Time) ([]*domain.ClockEvent, error) {
	return nil, nil
}
func (f *fakeClockEventRepo) FindBySite(uuid.UUID, time.Time, time.Time) ([]*domain.ClockEvent, error) {
	return nil, nil
}
func (f *fakeClockEventRepo) FindUnsynced(uuid.UUID) ([]*domain.ClockEvent, error) { return nil, nil }
func (f *fakeClockEventRepo) ExistsDuplicate(domain.EmployeeID, domain.DeviceID, domain.ClockEventType, time.Time) (bool, error) {
	return false, nil
}

type fakeDuplicateDetector struct{ duplicate bool }

func (f *fakeDuplicateDetector) IsDuplicate(domain.EmployeeID, domain.DeviceID, domain.ClockEventType, time.Time) (bool, error) {
	return f.duplicate, nil
}
func (f *fakeDuplicateDetector) FindNearDuplicates(domain.EmployeeID, time.Duration) ([]*domain.ClockEvent, error) {
	return nil, nil
}

type fakePublisher struct{ events []domain.DomainEvent }

func (f *fakePublisher) Publish(event domain.DomainEvent) error {
	f.events = append(f.events, event)
	return nil
}

type fakeSyncQueue struct {
	payload  []byte
	metadata domain.SyncMetadata
}

func (f *fakeSyncQueue) Enqueue(_ uuid.UUID, metadata domain.SyncMetadata, payload []byte) error {
	f.metadata, f.payload = metadata, payload
	return nil
}

func validClockInCommand() ClockInCommand {
	return ClockInCommand{
		EmployeeID: domain.EmployeeID(uuid.New()), MinistryID: uuid.New(), SiteID: uuid.New(), DeviceID: domain.DeviceID(uuid.New()),
		EventTime: time.Now().UTC().Add(-time.Minute), Timezone: "Asia/Baghdad", NodeID: uuid.New(),
	}
}

func TestClockInWritesCanonicalPayloadToOutbox(t *testing.T) {
	repo, queue, publisher := &fakeClockEventRepo{}, &fakeSyncQueue{}, &fakePublisher{}
	handler := NewClockInHandler(repo, nil, nil, &fakeDuplicateDetector{}, nil, publisher, queue)
	event, err := handler.Handle(validClockInCommand())
	if err != nil {
		t.Fatalf("Handle() error = %v", err)
	}
	if len(repo.saved) != 1 || len(publisher.events) != 1 {
		t.Fatalf("expected one persisted event and one domain event")
	}
	var payload domain.SyncPayload
	if err := json.Unmarshal(queue.payload, &payload); err != nil {
		t.Fatalf("invalid outbox payload: %v", err)
	}
	if payload.ID != event.Identity() || payload.EmployeeID != event.EmployeeID() || payload.EventType != domain.ClockIn {
		t.Fatalf("payload does not match event")
	}
}

func TestClockInRejectsDuplicate(t *testing.T) {
	dup := &fakeDuplicateDetector{duplicate: true}
	handler := NewClockInHandler(&fakeClockEventRepo{}, nil, nil, dup, nil, &fakePublisher{}, &fakeSyncQueue{})
	_, err := handler.Handle(validClockInCommand())
	if !errors.Is(err, ErrDuplicateEvent) {
		t.Fatalf("expected ErrDuplicateEvent, got %v", err)
	}
}

func TestClockInRejectsUnconfiguredBiometricVerification(t *testing.T) {
	handler := NewClockInHandler(&fakeClockEventRepo{}, nil, nil, &fakeDuplicateDetector{}, nil, &fakePublisher{}, &fakeSyncQueue{})
	cmd := validClockInCommand()
	cmd.BiometricData = []byte("biometric")
	_, err := handler.Handle(cmd)
	if !errors.Is(err, ErrBiometricUnavailable) {
		t.Fatalf("expected ErrBiometricUnavailable, got %v", err)
	}
}

func TestClockInRejectsFutureEvent(t *testing.T) {
	handler := NewClockInHandler(&fakeClockEventRepo{}, nil, nil, &fakeDuplicateDetector{}, nil, &fakePublisher{}, &fakeSyncQueue{})
	cmd := validClockInCommand()
	cmd.EventTime = time.Now().UTC().Add(10 * time.Minute)
	_, err := handler.Handle(cmd)
	if !errors.Is(err, ErrEventInFuture) {
		t.Fatalf("expected ErrEventInFuture, got %v", err)
	}
}
