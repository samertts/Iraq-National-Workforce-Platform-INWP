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

type ExceptionRepository struct {
	pool *pgxpool.Pool
}

func NewExceptionRepository(pool *pgxpool.Pool) *ExceptionRepository {
	return &ExceptionRepository{pool: pool}
}

func (r *ExceptionRepository) Save(exception *domain.AttendanceException) error {
	query := `
		INSERT INTO attendance.attendance_exceptions (
			id, employee_id, clock_event_id, exception_type, severity,
			description, occurred_at, detected_at,
			justification_reason, justification_type, justification_submitted_at,
			resolved_at, resolved_by, escalated_at
		) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
		ON CONFLICT (id) DO UPDATE SET
			justification_reason = EXCLUDED.justification_reason,
			justification_type = EXCLUDED.justification_type,
			justification_submitted_at = EXCLUDED.justification_submitted_at,
			resolved_at = EXCLUDED.resolved_at,
			resolved_by = EXCLUDED.resolved_by,
			escalated_at = EXCLUDED.escalated_at`

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	_, err := r.pool.Exec(ctx, query,
		exception.Identity(),
		uuid.UUID(exception.EmployeeID()),
		nil,
		string(exception.ExceptionType()),
		string(exception.Severity()),
		exception.Description(),
		time.Now(),
		time.Now(),
		nil, nil, nil,
		nil, nil, nil,
	)
	return err
}

func (r *ExceptionRepository) FindByID(id uuid.UUID) (*domain.AttendanceException, error) {
	query := `
		SELECT id, employee_id, clock_event_id, exception_type, severity,
		       description, occurred_at, detected_at
		FROM attendance.attendance_exceptions WHERE id = $1`

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	row := r.pool.QueryRow(ctx, query, id)
	return r.scanRow(row)
}

func (r *ExceptionRepository) FindByEmployee(employeeID domain.EmployeeID, from, to time.Time) ([]*domain.AttendanceException, error) {
	query := `
		SELECT id, employee_id, clock_event_id, exception_type, severity,
		       description, occurred_at, detected_at
		FROM attendance.attendance_exceptions
		WHERE employee_id = $1 AND occurred_at >= $2 AND occurred_at <= $3
		ORDER BY occurred_at DESC`

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	rows, err := r.pool.Query(ctx, query, uuid.UUID(employeeID), from, to)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return r.scanRows(rows)
}

func (r *ExceptionRepository) FindUnresolved(siteID uuid.UUID) ([]*domain.AttendanceException, error) {
	query := `
		SELECT e.id, e.employee_id, e.clock_event_id, e.exception_type, e.severity,
		       e.description, e.occurred_at, e.detected_at
		FROM attendance.attendance_exceptions e
		JOIN attendance.clock_events c ON c.id = e.clock_event_id
		WHERE c.site_id = $1 AND e.resolved_at IS NULL
		ORDER BY e.severity, e.occurred_at ASC`

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	rows, err := r.pool.Query(ctx, query, siteID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return r.scanRows(rows)
}

func (r *ExceptionRepository) FindEscalated(ministryID uuid.UUID) ([]*domain.AttendanceException, error) {
	query := `
		SELECT e.id, e.employee_id, e.clock_event_id, e.exception_type, e.severity,
		       e.description, e.occurred_at, e.detected_at
		FROM attendance.attendance_exceptions e
		WHERE e.escalated_at IS NOT NULL AND e.resolved_at IS NULL
		ORDER BY e.escalated_at DESC`

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	rows, err := r.pool.Query(ctx, query)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return r.scanRows(rows)
}

func (r *ExceptionRepository) scanRow(row pgx.Row) (*domain.AttendanceException, error) {
	var (
		id, employeeID    uuid.UUID
		clockEventID      *uuid.UUID
		exceptionTypeStr  string
		severityStr       string
		description       string
		occurredAt, detectedAt time.Time
	)

	err := row.Scan(
		&id, &employeeID, &clockEventID, &exceptionTypeStr, &severityStr,
		&description, &occurredAt, &detectedAt,
	)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, nil
		}
		return nil, err
	}

	_ = id
	_ = employeeID
	_ = clockEventID
	_ = exceptionTypeStr
	_ = severityStr
	_ = description
	_ = occurredAt
	_ = detectedAt
	return nil, nil
}

func (r *ExceptionRepository) scanRows(rows pgx.Rows) ([]*domain.AttendanceException, error) {
	var exceptions []*domain.AttendanceException
	for rows.Next() {
		var (
			id, employeeID    uuid.UUID
			clockEventID      *uuid.UUID
			exceptionTypeStr  string
			severityStr       string
			description       string
			occurredAt, detectedAt time.Time
		)

		err := rows.Scan(
			&id, &employeeID, &clockEventID, &exceptionTypeStr, &severityStr,
			&description, &occurredAt, &detectedAt,
		)
		if err != nil {
			return nil, err
		}
		_ = id
		_ = employeeID
		_ = clockEventID
		_ = exceptionTypeStr
		_ = severityStr
		_ = description
		_ = occurredAt
		_ = detectedAt
	}

	return exceptions, nil
}
