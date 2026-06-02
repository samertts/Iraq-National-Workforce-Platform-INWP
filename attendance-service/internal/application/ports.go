package application

import (
	"github.com/google/uuid"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

type EventPublisher interface {
	Publish(event domain.DomainEvent) error
}

type SyncQueue interface {
	Enqueue(entityID uuid.UUID, metadata domain.SyncMetadata) error
}
