package grpc

import (
	"context"
	"time"

	"github.com/google/uuid"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/application"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
	"google.golang.org/protobuf/types/known/timestamppb"
)

type AttendanceServer struct {
	clockInHandler  *application.ClockInHandler
	clockOutHandler *application.ClockOutHandler
	eventRepo       domain.ClockEventRepository
}

func NewAttendanceServer(
	clockInHandler *application.ClockInHandler,
	clockOutHandler *application.ClockOutHandler,
	eventRepo domain.ClockEventRepository,
) *AttendanceServer {
	return &AttendanceServer{
		clockInHandler:  clockInHandler,
		clockOutHandler: clockOutHandler,
		eventRepo:       eventRepo,
	}
}

type ClockInRequest struct {
	EmployeeId string
	DeviceId   string
	SiteId     string
	MinistryId string
	EventTime  *timestamppb.Timestamp
}

type ClockInResponse struct {
	EventId   string
	EventType string
	CreatedAt *timestamppb.Timestamp
}

type ClockOutRequest struct {
	EmployeeId string
	DeviceId   string
	SiteId     string
	MinistryId string
	EventTime  *timestamppb.Timestamp
}

type ClockOutResponse struct {
	EventId   string
	EventType string
	CreatedAt *timestamppb.Timestamp
}

func (s *AttendanceServer) ClockIn(ctx context.Context, req *ClockInRequest) (*ClockInResponse, error) {
	cmd := application.ClockInCommand{
		EmployeeID: domain.EmployeeID(uuid.MustParse(req.EmployeeId)),
		MinistryID: uuid.MustParse(req.MinistryId),
		SiteID:     uuid.MustParse(req.SiteId),
		DeviceID:   domain.DeviceID(uuid.MustParse(req.DeviceId)),
		EventTime:  req.EventTime.AsTime(),
		Timezone:   "Asia/Baghdad",
		NodeID:     uuid.New(),
	}

	event, err := s.clockInHandler.Handle(cmd)
	if err != nil {
		return nil, err
	}

	return &ClockInResponse{
		EventId:   event.Identity().String(),
		EventType: string(event.EventType()),
		CreatedAt: timestamppb.New(event.RecordedAt()),
	}, nil
}

func (s *AttendanceServer) ClockOut(ctx context.Context, req *ClockOutRequest) (*ClockOutResponse, error) {
	cmd := application.ClockOutCommand{
		EmployeeID: domain.EmployeeID(uuid.MustParse(req.EmployeeId)),
		MinistryID: uuid.MustParse(req.MinistryId),
		SiteID:     uuid.MustParse(req.SiteId),
		DeviceID:   domain.DeviceID(uuid.MustParse(req.DeviceId)),
		EventTime:  req.EventTime.AsTime(),
		Timezone:   "Asia/Baghdad",
		NodeID:     uuid.New(),
	}

	event, err := s.clockOutHandler.Handle(cmd)
	if err != nil {
		return nil, err
	}

	return &ClockOutResponse{
		EventId:   event.Identity().String(),
		EventType: string(event.EventType()),
		CreatedAt: timestamppb.New(event.RecordedAt()),
	}, nil
}

type GetEventsRequest struct {
	EmployeeId string
	SiteId     string
	From       *timestamppb.Timestamp
	To         *timestamppb.Timestamp
}

type EventResponse struct {
	Id         string
	EmployeeId string
	EventType  string
	EventTime  *timestamppb.Timestamp
	RecordedAt *timestamppb.Timestamp
}

type GetEventsResponse struct {
	Events []*EventResponse
}

func (s *AttendanceServer) GetEvents(ctx context.Context, req *GetEventsRequest) (*GetEventsResponse, error) {
	from := req.From.AsTime()
	to := req.To.AsTime()

	var events []*domain.ClockEvent
	var err error

	if req.EmployeeId != "" {
		events, err = s.eventRepo.FindByEmployee(domain.EmployeeID(uuid.MustParse(req.EmployeeId)), from, to)
	} else if req.SiteId != "" {
		events, err = s.eventRepo.FindBySite(uuid.MustParse(req.SiteId), from, to)
	}

	if err != nil {
		return nil, err
	}

	resp := &GetEventsResponse{}
	for _, e := range events {
		resp.Events = append(resp.Events, &EventResponse{
			Id:         e.Identity().String(),
			EmployeeId: uuid.UUID(e.EmployeeID()).String(),
			EventType:  string(e.EventType()),
			EventTime:  timestamppb.New(e.EventTime().DeviceTime),
			RecordedAt: timestamppb.New(e.RecordedAt()),
		})
	}

	return resp, nil
}

func RegisterGRPCServices(s *grpc.Server, server *AttendanceServer) {
	reflection.Register(s)
}

func WithTimeout(d time.Duration) grpc.ServerOption {
	return grpc.UnaryInterceptor(func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
		ctx, cancel := context.WithTimeout(ctx, d)
		defer cancel()
		return handler(ctx, req)
	})
}
