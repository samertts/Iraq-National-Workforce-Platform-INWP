package postgres

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

type ClockEventRepository struct {
	pool *pgxpool.Pool
}

func NewClockEventRepository(pool *pgxpool.Pool) *ClockEventRepository {
	return &ClockEventRepository{pool: pool}
}

func (r *ClockEventRepository) Save(event *domain.ClockEvent) error {
	query := `
		INSERT INTO attendance.clock_events (
			id, employee_id, ministry_id, site_id, device_id,
			event_type, event_time, recorded_at,
			biometric_match, latitude, longitude, ip_address,
			sync_id, source_node_id, sync_status, version
		) VALUES (
			$1, $2, $3, $4, $5,
			$6, $7, $8,
			$9, $10, $11, $12,
			$13, $14, $15, $16
		)`

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	sm := event.SyncMetadata()
	_, err := r.pool.Exec(ctx, query,
		event.Identity(),
		uuid.UUID(event.EmployeeID()),
		event.MinistryID(),
		event.SiteID(),
		uuid.UUID(event.DeviceID()),
		string(event.EventType()),
		event.EventTime().DeviceTime,
		event.RecordedAt(),
		nil,
		nil,
		nil,
		nil,
		sm.SyncID,
		sm.SourceNodeID,
		string(sm.Status),
		sm.Version,
	)
	return err
}

func (r *ClockEventRepository) SaveBatch(events []*domain.ClockEvent) error {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	tx, err := r.pool.Begin(ctx)
	if err != nil {
		return err
	}
	defer tx.Rollback(ctx)

	for _, event := range events {
		sm := event.SyncMetadata()
		_, err := tx.Exec(ctx, `
			INSERT INTO attendance.clock_events (
				id, employee_id, ministry_id, site_id, device_id,
				event_type, event_time, recorded_at,
				sync_id, source_node_id, sync_status, version
			) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
			ON CONFLICT (id, recorded_at) DO NOTHING`,
			event.Identity(),
			uuid.UUID(event.EmployeeID()),
			event.MinistryID(),
			event.SiteID(),
			uuid.UUID(event.DeviceID()),
			string(event.EventType()),
			event.EventTime().DeviceTime,
			event.RecordedAt(),
			sm.SyncID,
			sm.SourceNodeID,
			string(sm.Status),
			sm.Version,
		)
		if err != nil {
			return err
		}
	}

	return tx.Commit(ctx)
}

func (r *ClockEventRepository) FindByID(id uuid.UUID) (*domain.ClockEvent, error) {
	query := `
		SELECT id, employee_id, ministry_id, site_id, device_id,
		       event_type, event_time, recorded_at,
		       biometric_match, latitude, longitude, ip_address,
		       sync_id, source_node_id, sync_status, version
		FROM attendance.clock_events
		WHERE id = $1`

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	row := r.pool.QueryRow(ctx, query, id)
	return r.scanRow(row)
}

func (r *ClockEventRepository) FindByEmployee(employeeID domain.EmployeeID, from, to time.Time) ([]*domain.ClockEvent, error) {
	query := `
		SELECT id, employee_id, ministry_id, site_id, device_id,
		       event_type, event_time, recorded_at,
		       biometric_match, latitude, longitude, ip_address,
		       sync_id, source_node_id, sync_status, version
		FROM attendance.clock_events
		WHERE employee_id = $1 AND event_time >= $2 AND event_time <= $3
		ORDER BY event_time ASC`

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	rows, err := r.pool.Query(ctx, query, uuid.UUID(employeeID), from, to)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return r.scanRows(rows)
}

func (r *ClockEventRepository) FindByDevice(deviceID domain.DeviceID, from, to time.Time) ([]*domain.ClockEvent, error) {
	query := `
		SELECT id, employee_id, ministry_id, site_id, device_id,
		       event_type, event_time, recorded_at,
		       biometric_match, latitude, longitude, ip_address,
		       sync_id, source_node_id, sync_status, version
		FROM attendance.clock_events
		WHERE device_id = $1 AND event_time >= $2 AND event_time <= $3
		ORDER BY event_time ASC`

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	rows, err := r.pool.Query(ctx, query, uuid.UUID(deviceID), from, to)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return r.scanRows(rows)
}

func (r *ClockEventRepository) FindBySite(siteID uuid.UUID, from, to time.Time) ([]*domain.ClockEvent, error) {
	query := `
		SELECT id, employee_id, ministry_id, site_id, device_id,
		       event_type, event_time, recorded_at,
		       biometric_match, latitude, longitude, ip_address,
		       sync_id, source_node_id, sync_status, version
		FROM attendance.clock_events
		WHERE site_id = $1 AND event_time >= $2 AND event_time <= $3
		ORDER BY event_time ASC`

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	rows, err := r.pool.Query(ctx, query, siteID, from, to)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return r.scanRows(rows)
}

func (r *ClockEventRepository) FindUnsynced(nodeID uuid.UUID) ([]*domain.ClockEvent, error) {
	query := `
		SELECT id, employee_id, ministry_id, site_id, device_id,
		       event_type, event_time, recorded_at,
		       biometric_match, latitude, longitude, ip_address,
		       sync_id, source_node_id, sync_status, version
		FROM attendance.clock_events
		WHERE sync_status = 'local_only'
		ORDER BY recorded_at ASC`

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	rows, err := r.pool.Query(ctx, query)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return r.scanRows(rows)
}

func (r *ClockEventRepository) ExistsDuplicate(employeeID domain.EmployeeID, deviceID domain.DeviceID, eventType domain.ClockEventType, eventTime time.Time) (bool, error) {
	query := `
		SELECT EXISTS(
			SELECT 1 FROM attendance.clock_events
			WHERE employee_id = $1
			  AND device_id = $2
			  AND event_type = $3
			  AND event_time BETWEEN $4 - INTERVAL '30 seconds' AND $4 + INTERVAL '30 seconds'
		)`

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	var exists bool
	err := r.pool.QueryRow(ctx, query,
		uuid.UUID(employeeID),
		uuid.UUID(deviceID),
		string(eventType),
		eventTime,
	).Scan(&exists)

	return exists, err
}

func (r *ClockEventRepository) scanRow(row pgx.Row) (*domain.ClockEvent, error) {
	var (
		id, employeeID, ministryID, siteID, deviceID uuid.UUID
		eventTypeStr                                   string
		eventTime, recordedAt                          time.Time
		biometricMatch                                 *float64
		latitude, longitude                            *float64
		ipAddress                                      *string
		syncID, sourceNodeID                           uuid.UUID
		syncStatusStr                                  string
		version                                        int64
	)

	err := row.Scan(
		&id, &employeeID, &ministryID, &siteID, &deviceID,
		&eventTypeStr, &eventTime, &recordedAt,
		&biometricMatch, &latitude, &longitude, &ipAddress,
		&syncID, &sourceNodeID, &syncStatusStr, &version,
	)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, nil
		}
		return nil, err
	}

	event := &domain.ClockEvent{}
	return event, nil
}

func (r *ClockEventRepository) scanRows(rows pgx.Rows) ([]*domain.ClockEvent, error) {
	var events []*domain.ClockEvent
	for rows.Next() {
		var (
			id, employeeID, ministryID, siteID, deviceID uuid.UUID
			eventTypeStr                                  string
			eventTime, recordedAt                         time.Time
			biometricMatch                                *float64
			latitude, longitude                           *float64
			ipAddress                                     *string
			syncID, sourceNodeID                          uuid.UUID
			syncStatusStr                                 string
			version                                       int64
		)

		err := rows.Scan(
			&id, &employeeID, &ministryID, &siteID, &deviceID,
			&eventTypeStr, &eventTime, &recordedAt,
			&biometricMatch, &latitude, &longitude, &ipAddress,
			&syncID, &sourceNodeID, &syncStatusStr, &version,
		)
		if err != nil {
			return nil, err
		}
		_ = eventTypeStr
		_ = eventTime
		_ = recordedAt
		_ = syncID
		_ = sourceNodeID
		_ = syncStatusStr
		_ = version
		_ = biometricMatch
		_ = latitude
		_ = longitude
		_ = ipAddress
		_ = ministryID
		_ = siteID
		_ = deviceID
		_ = employeeID
	}

	return events, nil
}
