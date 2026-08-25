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

type PolicyRepository struct {
	pool *pgxpool.Pool
}

func NewPolicyRepository(pool *pgxpool.Pool) *PolicyRepository {
	return &PolicyRepository{pool: pool}
}

func (r *PolicyRepository) Save(policy *domain.AttendancePolicy) error {
	rules, err := json.Marshal(policy.Rules())
	if err != nil {
		return err
	}
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
	_, err = r.pool.Exec(ctx, query,
		policy.Identity(), policy.MinistryID(), policy.SiteID(), policy.Name(), rules,
		policy.EffectiveFrom(), policy.EffectiveTo(), policy.Version(), policy.Supersedes(),
		policy.ApprovedBy(), policy.CreatedAt(),
	)
	return err
}

func (r *PolicyRepository) FindByID(id uuid.UUID) (*domain.AttendancePolicy, error) {
	query := `SELECT id, ministry_id, site_id, name, rules, effective_from,
		effective_to, version, supersedes, approved_by, created_at
		FROM attendance.attendance_policies WHERE id = $1`
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	return r.scanRow(r.pool.QueryRow(ctx, query, id))
}

func (r *PolicyRepository) FindActiveBySite(siteID uuid.UUID) (*domain.AttendancePolicy, error) {
	query := `SELECT id, ministry_id, site_id, name, rules, effective_from,
		effective_to, version, supersedes, approved_by, created_at
		FROM attendance.attendance_policies
		WHERE site_id = $1 AND effective_to IS NULL
		ORDER BY effective_from DESC LIMIT 1`
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	return r.scanRow(r.pool.QueryRow(ctx, query, siteID))
}

func (r *PolicyRepository) FindActiveByMinistry(ministryID uuid.UUID) (*domain.AttendancePolicy, error) {
	query := `SELECT id, ministry_id, site_id, name, rules, effective_from,
		effective_to, version, supersedes, approved_by, created_at
		FROM attendance.attendance_policies
		WHERE ministry_id = $1 AND site_id IS NULL AND effective_to IS NULL
		ORDER BY effective_from DESC LIMIT 1`
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	return r.scanRow(r.pool.QueryRow(ctx, query, ministryID))
}

func (r *PolicyRepository) FindHistoryBySite(siteID uuid.UUID) ([]*domain.AttendancePolicy, error) {
	query := `SELECT id, ministry_id, site_id, name, rules, effective_from,
		effective_to, version, supersedes, approved_by, created_at
		FROM attendance.attendance_policies WHERE site_id = $1
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

type policyScanner interface{ Scan(dest ...any) error }

func scanPolicy(scanner policyScanner) (*domain.AttendancePolicy, error) {
	var (
		id, ministryID uuid.UUID
		siteID         *uuid.UUID
		name           string
		rulesJSON      []byte
		effectiveFrom  time.Time
		effectiveTo    *time.Time
		version        int
		supersedes     *uuid.UUID
		approvedBy     uuid.UUID
		createdAt      time.Time
	)
	if err := scanner.Scan(&id, &ministryID, &siteID, &name, &rulesJSON, &effectiveFrom,
		&effectiveTo, &version, &supersedes, &approvedBy, &createdAt); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, nil
		}
		return nil, err
	}
	var rules domain.AttendanceRuleSet
	if len(rulesJSON) > 0 {
		if err := json.Unmarshal(rulesJSON, &rules); err != nil {
			return nil, err
		}
	}
	return domain.RehydrateAttendancePolicy(id, ministryID, siteID, name, rules,
		effectiveFrom, effectiveTo, version, supersedes, approvedBy, createdAt), nil
}

func (r *PolicyRepository) scanRow(row pgx.Row) (*domain.AttendancePolicy, error) {
	return scanPolicy(row)
}

func (r *PolicyRepository) scanRows(rows pgx.Rows) ([]*domain.AttendancePolicy, error) {
	policies := make([]*domain.AttendancePolicy, 0)
	for rows.Next() {
		policy, err := scanPolicy(rows)
		if err != nil {
			return nil, err
		}
		if policy != nil {
			policies = append(policies, policy)
		}
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	return policies, nil
}
