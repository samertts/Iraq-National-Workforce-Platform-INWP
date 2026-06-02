package sync

import (
	"context"
	"encoding/json"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

type Outbox struct {
	pool *pgxpool.Pool
}

func NewOutbox(pool *pgxpool.Pool) *Outbox {
	return &Outbox{pool: pool}
}

type OutboxEntry struct {
	ID           uuid.UUID       `json:"id"`
	EntityID     uuid.UUID       `json:"entity_id"`
	EntityType   string          `json:"entity_type"`
	SyncID       uuid.UUID       `json:"sync_id"`
	SourceNodeID uuid.UUID       `json:"source_node_id"`
	Status       string          `json:"status"`
	Payload      json.RawMessage `json:"payload"`
	CreatedAt    time.Time       `json:"created_at"`
}

func (o *Outbox) Enqueue(entityID uuid.UUID, metadata domain.SyncMetadata) error {
	query := `
		INSERT INTO sync.outbox (entity_id, entity_type, sync_id, source_node_id, status)
		VALUES ($1, 'clock_event', $2, $3, 'pending')`

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	_, err := o.pool.Exec(ctx, query,
		entityID,
		metadata.SyncID,
		metadata.SourceNodeID,
	)
	return err
}

func (o *Outbox) FetchPending(limit int) ([]OutboxEntry, error) {
	query := `
		SELECT id, entity_id, entity_type, sync_id, source_node_id, status, payload, created_at
		FROM sync.outbox
		WHERE status = 'pending'
		ORDER BY created_at ASC
		LIMIT $1
		FOR UPDATE SKIP LOCKED`

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	rows, err := o.pool.Query(ctx, query, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var entries []OutboxEntry
	for rows.Next() {
		var entry OutboxEntry
		if err := rows.Scan(
			&entry.ID, &entry.EntityID, &entry.EntityType,
			&entry.SyncID, &entry.SourceNodeID,
			&entry.Status, &entry.Payload, &entry.CreatedAt,
		); err != nil {
			return nil, err
		}
		entries = append(entries, entry)
	}

	return entries, nil
}

func (o *Outbox) MarkSent(id uuid.UUID) error {
	query := `UPDATE sync.outbox SET status = 'sent' WHERE id = $1`
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	_, err := o.pool.Exec(ctx, query, id)
	return err
}
