package sync

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/nats-io/nats.go"
)

type Relay struct {
	outbox   *Outbox
	conn     *nats.Conn
	interval time.Duration
}

func NewRelay(outbox *Outbox, conn *nats.Conn) *Relay {
	return &Relay{outbox: outbox, conn: conn, interval: 500 * time.Millisecond}
}

type syncEventEnvelope struct {
	EventID      string `json:"event_id"`
	NodeID       string `json:"node_id"`
	EventType    string `json:"event_type"`
	PartitionKey string `json:"partition_key"`
	Payload      []int  `json:"payload"`

	VersionVector struct {
		Versions       map[string]uint64 `json:"versions"`
		LocalTimestamp uint64            `json:"local_timestamp"`
	} `json:"version_vector"`
	LocalTimestamp uint64            `json:"local_timestamp"`
	Signature      []int             `json:"signature"`
	SigningKeyID   string            `json:"signing_key_id"`
	SchemaVersion  string            `json:"schema_version"`
	Metadata       map[string]string `json:"metadata"`
	CreatedAt      time.Time         `json:"created_at"`
}

func (r *Relay) Run(ctx context.Context) {
	if r == nil || r.outbox == nil || r.conn == nil {
		return
	}
	ticker := time.NewTicker(r.interval)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			r.process(ctx)
		}
	}
}

func (r *Relay) process(ctx context.Context) {
	_ = r.outbox.ResetStale(5 * time.Minute)
	entries, err := r.outbox.ClaimPending(100)
	if err != nil {
		return
	}
	for _, entry := range entries {
		if err := r.publish(ctx, entry); err != nil {
			_ = r.outbox.MarkFailed(entry.ID, err.Error())
			continue
		}
		_ = r.outbox.MarkSent(entry.ID)
	}
}

func bytesToInts(data []byte) []int {
	result := make([]int, len(data))
	for i, value := range data {
		result[i] = int(value)
	}
	return result
}

func (r *Relay) publish(ctx context.Context, entry OutboxEntry) error {
	if len(entry.Payload) == 0 {
		return fmt.Errorf("outbox entry %s has empty payload", entry.ID)
	}
	var payload struct {
		MinistryID string `json:"ministry_id"`
	}
	if err := json.Unmarshal(entry.Payload, &payload); err != nil {
		return fmt.Errorf("decode outbox payload: %w", err)
	}
	if payload.MinistryID == "" {
		return fmt.Errorf("outbox entry %s has no ministry_id", entry.ID)
	}
	createdAt := entry.CreatedAt.UTC()
	if createdAt.IsZero() {
		createdAt = time.Now().UTC()
	}
	event := syncEventEnvelope{
		EventID: entry.SyncID.String(), NodeID: entry.SourceNodeID.String(),
		EventType: "inwp.attendance.v1.clock_event", PartitionKey: payload.MinistryID,
		Payload: bytesToInts(entry.Payload),
		VersionVector: struct {
			Versions       map[string]uint64 `json:"versions"`
			LocalTimestamp uint64            `json:"local_timestamp"`
		}{Versions: map[string]uint64{entry.SourceNodeID.String(): 1}, LocalTimestamp: uint64(createdAt.UnixNano())},
		LocalTimestamp: uint64(createdAt.UnixNano()), Signature: []int{}, SigningKeyID: "",
		SchemaVersion: "1.0", Metadata: map[string]string{"entity_id": entry.EntityID.String(), "entity_type": entry.EntityType}, CreatedAt: createdAt,
	}
	data, err := json.Marshal(event)
	if err != nil {
		return err
	}
	if err := r.conn.Publish("inwp.sync.event.clock_event", data); err != nil {
		return err
	}
	flushCtx, cancel := context.WithTimeout(ctx, 10*time.Second)
	defer cancel()
	return r.conn.FlushWithContext(flushCtx)
}
