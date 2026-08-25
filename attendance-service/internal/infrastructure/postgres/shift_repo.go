package postgres

import (
	"context"
	"encoding/json"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

type ShiftRepository struct {
	pool *pgxpool.Pool
}

func NewShiftRepository(pool *pgxpool.Pool) *ShiftRepository {
	return &ShiftRepository{pool: pool}
}

func (r *ShiftRepository) Save(shift *domain.Shift) error {
	overtimePolicy, err := json.Marshal(shift.OvertimePolicy())
	if err != nil {
		return err
	}
	query := `
		INSERT INTO attendance.shifts (
			id, ministry_id, site_id, name, start_time, end_time,
			grace_period, break_duration, overtime_policy, is_active, version
		) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
		ON CONFLICT (id) DO UPDATE SET
			name = EXCLUDED.name, start_time = EXCLUDED.start_time,
			end_time = EXCLUDED.end_time, grace_period = EXCLUDED.grace_period,
			break_duration = EXCLUDED.break_duration, overtime_policy = EXCLUDED.overtime_policy,
			is_active = EXCLUDED.is_active, version = EXCLUDED.version`

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	_, err = r.pool.Exec(ctx, query,
		shift.Identity(), shift.MinistryID(), shift.SiteID(), shift.Name(),
		shift.StartTime(), shift.EndTime(), int64(shift.GracePeriod().Seconds()),
		int64(shift.BreakDuration().Seconds()), overtimePolicy, shift.IsActive(), shift.Version(),
	)
	return err
}

func (r *ShiftRepository) FindByID(id uuid.UUID) (*domain.Shift, error) {
	query := `SELECT id, ministry_id, site_id, name, start_time, end_time,
		grace_period, break_duration, overtime_policy, is_active, version
		FROM attendance.shifts WHERE id = $1`
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	return r.scanRow(r.pool.QueryRow(ctx, query, id))
}

func (r *ShiftRepository) FindBySite(siteID uuid.UUID) ([]*domain.Shift, error) {
	query := `SELECT id, ministry_id, site_id, name, start_time, end_time,
		grace_period, break_duration, overtime_policy, is_active, version
		FROM attendance.shifts WHERE site_id = $1 ORDER BY start_time ASC`
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	rows, err := r.pool.Query(ctx, query, siteID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	return r.scanRows(rows)
}

func (r *ShiftRepository) FindActiveBySite(siteID uuid.UUID) (*domain.Shift, error) {
	query := `SELECT id, ministry_id, site_id, name, start_time, end_time,
		grace_period, break_duration, overtime_policy, is_active, version
		FROM attendance.shifts WHERE site_id = $1 AND is_active = true LIMIT 1`
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	return r.scanRow(r.pool.QueryRow(ctx, query, siteID))
}

func (r *ShiftRepository) Delete(id uuid.UUID) error {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	_, err := r.pool.Exec(ctx, `DELETE FROM attendance.shifts WHERE id = $1`, id)
	return err
}

type shiftScanner interface{ Scan(dest ...any) error }

func scanShift(scanner shiftScanner) (*domain.Shift, error) {
	var (
		id, ministryID, siteID uuid.UUID
		name                   string
		startTime, endTime     time.Time
		gracePeriodSec         int64
		breakDurationSec       int64
		overtimePolicyJSON     []byte
		isActive               bool
		version                int64
	)
	if err := scanner.Scan(&id, &ministryID, &siteID, &name, &startTime, &endTime,
		&gracePeriodSec, &breakDurationSec, &overtimePolicyJSON, &isActive, &version); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, nil
		}
		return nil, err
	}
	var overtimePolicy domain.OvertimePolicy
	if len(overtimePolicyJSON) > 0 {
		if err := json.Unmarshal(overtimePolicyJSON, &overtimePolicy); err != nil {
			return nil, err
		}
	}
	return domain.RehydrateShift(id, ministryID, siteID, name, startTime, endTime,
		time.Duration(gracePeriodSec)*time.Second, time.Duration(breakDurationSec)*time.Second,
		overtimePolicy, isActive, version), nil
}

func (r *ShiftRepository) scanRow(row pgx.Row) (*domain.Shift, error) {
	return scanShift(row)
}

func (r *ShiftRepository) scanRows(rows pgx.Rows) ([]*domain.Shift, error) {
	shifts := make([]*domain.Shift, 0)
	for rows.Next() {
		shift, err := scanShift(rows)
		if err != nil {
			return nil, err
		}
		if shift != nil {
			shifts = append(shifts, shift)
		}
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	return shifts, nil
}
