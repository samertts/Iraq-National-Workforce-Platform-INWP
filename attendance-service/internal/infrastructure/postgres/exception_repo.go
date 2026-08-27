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
	var justificationReason any
	var justificationType any
	var justificationSubmittedAt any
	if justification := exception.Justification(); justification != nil {
		justificationReason = justification.Reason
		justificationType = string(justification.Type)
		justificationSubmittedAt = justification.SubmittedAt
	}

	query := `
		INSERT INTO attendance.attendance_exceptions (
			id, employee_id, clock_event_id, exception_type, severity,
			description, occurred_at, detected_at,
			justification_reason, justification_type, justification_submitted_at,
			resolved_at, resolved_by, escalated_at
		) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
		ON CONFLICT (id) DO UPDATE SET
			clock_event_id = EXCLUDED.clock_event_id,
			exception_type = EXCLUDED.exception_type,
			severity = EXCLUDED.severity,
			description = EXCLUDED.description,
			occurred_at = EXCLUDED.occurred_at,
			justification_reason = EXCLUDED.justification_reason,
			justification_type = EXCLUDED.justification_type,
			justification_submitted_at = EXCLUDED.justification_submitted_at,
			resolved_at = EXCLUDED.resolved_at,
			resolved_by = EXCLUDED.resolved_by,
			escalated_at = EXCLUDED.escalated_at`

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	_, err := r.pool.Exec(ctx, query,
		exception.Identity(), uuid.UUID(exception.EmployeeID()), exception.ClockEventID(),
		string(exception.ExceptionType()), string(exception.Severity()), exception.Description(),
		exception.OccurredAt(), exception.DetectedAt(), justificationReason, justificationType,
		justificationSubmittedAt, exception.ResolvedAt(), exception.ResolvedBy(), exception.EscalatedAt(),
	)
	return err
}

const exceptionSelect = `
	SELECT id, employee_id, clock_event_id, exception_type, severity,
	       description, occurred_at, detected_at,
	       justification_reason, justification_type, justification_submitted_at,
	       resolved_at, resolved_by, escalated_at
`

func (r *ExceptionRepository) FindByID(id uuid.UUID) (*domain.AttendanceException, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	return scanException(r.pool.QueryRow(ctx, exceptionSelect+` FROM attendance.attendance_exceptions WHERE id = $1`, id))
}

func (r *ExceptionRepository) FindByEmployee(employeeID domain.EmployeeID, from, to time.Time) ([]*domain.AttendanceException, error) {
	query := exceptionSelect + ` FROM attendance.attendance_exceptions
		WHERE employee_id = $1 AND occurred_at >= $2 AND occurred_at <= $3
		ORDER BY occurred_at DESC`
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	rows, err := r.pool.Query(ctx, query, uuid.UUID(employeeID), from, to)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	return scanExceptions(rows)
}

func (r *ExceptionRepository) FindUnresolved(siteID uuid.UUID) ([]*domain.AttendanceException, error) {
	query := exceptionSelect + ` FROM attendance.attendance_exceptions e
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
	return scanExceptions(rows)
}

func (r *ExceptionRepository) FindEscalated(ministryID uuid.UUID) ([]*domain.AttendanceException, error) {
	query := exceptionSelect + ` FROM attendance.attendance_exceptions e
		LEFT JOIN attendance.clock_events c ON c.id = e.clock_event_id
		WHERE e.escalated_at IS NOT NULL AND e.resolved_at IS NULL
		  AND ($1 = '00000000-0000-0000-0000-000000000000'::uuid OR c.ministry_id = $1)
		ORDER BY e.escalated_at DESC`
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	rows, err := r.pool.Query(ctx, query, ministryID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	return scanExceptions(rows)
}

type exceptionScanner interface{ Scan(dest ...any) error }

func scanException(scanner exceptionScanner) (*domain.AttendanceException, error) {
	var (
		id, employeeID                       uuid.UUID
		clockEventID                         *uuid.UUID
		exceptionType, severity, description string
		occurredAt, detectedAt               time.Time
		justificationReason                  *string
		justificationType                    *string
		justificationAt                      *time.Time
		resolvedAt                           *time.Time
		resolvedBy                           *uuid.UUID
		escalatedAt                          *time.Time
	)
	if err := scanner.Scan(&id, &employeeID, &clockEventID, &exceptionType, &severity,
		&description, &occurredAt, &detectedAt, &justificationReason, &justificationType,
		&justificationAt, &resolvedAt, &resolvedBy, &escalatedAt); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, nil
		}
		return nil, err
	}

	var justification *domain.Justification
	if justificationReason != nil || justificationType != nil || justificationAt != nil {
		j := &domain.Justification{}
		if justificationReason != nil {
			j.Reason = *justificationReason
		}
		if justificationType != nil {
			j.Type = domain.JustificationType(*justificationType)
		}
		if justificationAt != nil {
			j.SubmittedAt = *justificationAt
		}
		justification = j
	}
	return domain.RehydrateAttendanceException(id, domain.EmployeeID(employeeID), clockEventID,
		domain.ExceptionType(exceptionType), domain.ExceptionSeverity(severity), description,
		occurredAt, detectedAt, justification, resolvedAt, resolvedBy, escalatedAt), nil
}

func scanExceptions(rows pgx.Rows) ([]*domain.AttendanceException, error) {
	exceptions := make([]*domain.AttendanceException, 0)
	for rows.Next() {
		exception, err := scanException(rows)
		if err != nil {
			return nil, err
		}
		if exception != nil {
			exceptions = append(exceptions, exception)
		}
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	return exceptions, nil
}
