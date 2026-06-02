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

type PolicyRepository struct {
	pool *pgxpool.Pool
}

func NewPolicyRepository(pool *pgxpool.Pool) *PolicyRepository {
	return &PolicyRepository{pool: pool}
}

func (r *PolicyRepository) Save(policy *domain.AttendancePolicy) error {
	query := `
		INSERT INTO attendance.attendance_policies (
			id, ministry_id, site_id, name, rules,
			effective_from, effective_to, version, supersedes, approved_by, created_at
		) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
		ON CONFLICT (id) DO UPDATE SET
			effective_to = EXCLUDED.effective_to,
			version = EXCLUDED.version,
			supersedes = EXCLUDED.supersedes`

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	_, err := r.pool.Exec(ctx, query,
		policy.Identity(),
		policy.MinistryID(),
		policy.SiteID(),
		policy.Name(),
		policy.Rules(),
		policy.EffectiveFrom(),
		policy.EffectiveTo(),
		policy.Version(),
		policy.Supersedes(),
		nil,
		time.Now().UTC(),
	)
	return err
}

func (r *PolicyRepository) FindByID(id uuid.UUID) (*domain.AttendancePolicy, error) {
	query := `
		SELECT id, ministry_id, site_id, name, rules,
		       effective_from, effective_to, version, supersedes
		FROM attendance.attendance_policies WHERE id = $1`

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	row := r.pool.QueryRow(ctx, query, id)
	return r.scanRow(row)
}

func (r *PolicyRepository) FindActiveBySite(siteID uuid.UUID) (*domain.AttendancePolicy, error) {
	query := `
		SELECT id, ministry_id, site_id, name, rules,
		       effective_from, effective_to, version, supersedes
		FROM attendance.attendance_policies
		WHERE site_id = $1 AND effective_to IS NULL
		ORDER BY effective_from DESC LIMIT 1`

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	row := r.pool.QueryRow(ctx, query, siteID)
	return r.scanRow(row)
}

func (r *PolicyRepository) FindActiveByMinistry(ministryID uuid.UUID) (*domain.AttendancePolicy, error) {
	query := `
		SELECT id, ministry_id, site_id, name, rules,
		       effective_from, effective_to, version, supersedes
		FROM attendance.attendance_policies
		WHERE ministry_id = $1 AND site_id IS NULL AND effective_to IS NULL
		ORDER BY effective_from DESC LIMIT 1`

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	row := r.pool.QueryRow(ctx, query, ministryID)
	return r.scanRow(row)
}

func (r *PolicyRepository) FindHistoryBySite(siteID uuid.UUID) ([]*domain.AttendancePolicy, error) {
	query := `
		SELECT id, ministry_id, site_id, name, rules,
		       effective_from, effective_to, version, supersedes
		FROM attendance.attendance_policies
		WHERE site_id = $1
		ORDER BY effective_from DESC`

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	rows, err := r.pool.Query(ctx, query, siteID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return r.scanRows(rows)
}

func (r *PolicyRepository) scanRow(row pgx.Row) (*domain.AttendancePolicy, error) {
	var (
		id, ministryID             uuid.UUID
		siteID                     *uuid.UUID
		name                       string
		rules                      domain.AttendanceRuleSet
		effectiveFrom              time.Time
		effectiveTo                *time.Time
		version                    int
		supersedes                 *uuid.UUID
	)

	err := row.Scan(
		&id, &ministryID, &siteID, &name, &rules,
		&effectiveFrom, &effectiveTo, &version, &supersedes,
	)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, nil
		}
		return nil, err
	}

	_ = id
	_ = ministryID
	_ = siteID
	_ = name
	_ = rules
	_ = effectiveFrom
	_ = effectiveTo
	_ = version
	_ = supersedes
	return nil, nil
}

func (r *PolicyRepository) scanRows(rows pgx.Rows) ([]*domain.AttendancePolicy, error) {
	var policies []*domain.AttendancePolicy
	for rows.Next() {
		var (
			id, ministryID             uuid.UUID
			siteID                     *uuid.UUID
			name                       string
			rules                      domain.AttendanceRuleSet
			effectiveFrom              time.Time
			effectiveTo                *time.Time
			version                    int
			supersedes                 *uuid.UUID
		)

		err := rows.Scan(
			&id, &ministryID, &siteID, &name, &rules,
			&effectiveFrom, &effectiveTo, &version, &supersedes,
		)
		if err != nil {
			return nil, err
		}
		_ = id
		_ = ministryID
		_ = siteID
		_ = name
		_ = rules
		_ = effectiveFrom
		_ = effectiveTo
		_ = version
		_ = supersedes
	}

	return policies, nil
}
