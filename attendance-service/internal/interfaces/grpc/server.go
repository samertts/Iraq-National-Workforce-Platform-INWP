package grpc

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/application"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
	attendancepb "github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/proto"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/reflection"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/timestamppb"
)

type AttendanceServer struct {
	attendancepb.UnimplementedAttendanceServiceServer
	clockInHandler  *application.ClockInHandler
	clockOutHandler *application.ClockOutHandler
	eventRepo       domain.ClockEventRepository
}

func NewAttendanceServer(
	clockInHandler *application.ClockInHandler,
	clockOutHandler *application.ClockOutHandler,
	eventRepo domain.ClockEventRepository,
) *AttendanceServer {
	return &AttendanceServer{clockInHandler: clockInHandler, clockOutHandler: clockOutHandler, eventRepo: eventRepo}
}

func (s *AttendanceServer) ClockIn(ctx context.Context, req *attendancepb.ClockRequest) (*attendancepb.ClockResponse, error) {
	cmd, err := clockCommand(req)
	if err != nil {
		return nil, err
	}
	event, err := s.clockInHandler.Handle(application.ClockInCommand{
		EmployeeID: cmd.EmployeeID, MinistryID: cmd.MinistryID, SiteID: cmd.SiteID,
		DeviceID: cmd.DeviceID, EventTime: cmd.EventTime, Timezone: cmd.Timezone,
		BiometricData: cmd.BiometricData, Latitude: cmd.Latitude, Longitude: cmd.Longitude,
		IPAddress: cmd.IPAddress, NodeID: cmd.NodeID,
	})
	if err != nil {
		return nil, mapError(err)
	}
	return clockResponse(event), nil
}

func (s *AttendanceServer) ClockOut(ctx context.Context, req *attendancepb.ClockRequest) (*attendancepb.ClockResponse, error) {
	cmd, err := clockCommand(req)
	if err != nil {
		return nil, err
	}
	event, err := s.clockOutHandler.Handle(application.ClockOutCommand{
		EmployeeID: cmd.EmployeeID, MinistryID: cmd.MinistryID, SiteID: cmd.SiteID,
		DeviceID: cmd.DeviceID, EventTime: cmd.EventTime, Timezone: cmd.Timezone,
		BiometricData: cmd.BiometricData, Latitude: cmd.Latitude, Longitude: cmd.Longitude,
		IPAddress: cmd.IPAddress, NodeID: cmd.NodeID,
	})
	if err != nil {
		return nil, mapError(err)
	}
	return clockResponse(event), nil
}

func (s *AttendanceServer) GetEvents(ctx context.Context, req *attendancepb.GetEventsRequest) (*attendancepb.GetEventsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "request is required")
	}
	from, to := time.Now().UTC().Add(-24*time.Hour), time.Now().UTC()
	if req.From != nil {
		if err := req.From.CheckValid(); err != nil {
			return nil, status.Error(codes.InvalidArgument, "invalid from timestamp")
		}
		from = req.From.AsTime()
	}
	if req.To != nil {
		if err := req.To.CheckValid(); err != nil {
			return nil, status.Error(codes.InvalidArgument, "invalid to timestamp")
		}
		to = req.To.AsTime()
	}

	var events []*domain.ClockEvent
	var err error
	switch {
	case req.EmployeeId != "" && req.SiteId == "":
		id, parseErr := uuid.Parse(req.EmployeeId)
		if parseErr != nil {
			return nil, status.Error(codes.InvalidArgument, "invalid employee_id")
		}
		events, err = s.eventRepo.FindByEmployee(domain.EmployeeID(id), from, to)
	case req.SiteId != "" && req.EmployeeId == "":
		id, parseErr := uuid.Parse(req.SiteId)
		if parseErr != nil {
			return nil, status.Error(codes.InvalidArgument, "invalid site_id")
		}
		events, err = s.eventRepo.FindBySite(id, from, to)
	default:
		return nil, status.Error(codes.InvalidArgument, "exactly one of employee_id or site_id is required")
	}
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	response := &attendancepb.GetEventsResponse{Events: make([]*attendancepb.Event, 0, len(events))}
	for _, event := range events {
		response.Events = append(response.Events, &attendancepb.Event{
			Id: event.Identity().String(), EmployeeId: uuid.UUID(event.EmployeeID()).String(),
			MinistryId: event.MinistryID().String(), SiteId: event.SiteID().String(),
			DeviceId: uuid.UUID(event.DeviceID()).String(), EventType: string(event.EventType()),
			EventTime: timestamppb.New(event.EventTime().DeviceTime), RecordedAt: timestamppb.New(event.RecordedAt()),
			SyncStatus: string(event.SyncMetadata().Status),
		})
	}
	return response, nil
}

type clockCommandData struct {
	EmployeeID    domain.EmployeeID
	MinistryID    uuid.UUID
	SiteID        uuid.UUID
	DeviceID      domain.DeviceID
	EventTime     time.Time
	Timezone      string
	BiometricData []byte
	Latitude      *float64
	Longitude     *float64
	IPAddress     *string
	NodeID        uuid.UUID
}

func clockCommand(req *attendancepb.ClockRequest) (clockCommandData, error) {
	if req == nil {
		return clockCommandData{}, status.Error(codes.InvalidArgument, "request is required")
	}
	employeeID, err := uuid.Parse(req.EmployeeId)
	if err != nil {
		return clockCommandData{}, status.Error(codes.InvalidArgument, "invalid employee_id")
	}
	deviceID, err := uuid.Parse(req.DeviceId)
	if err != nil {
		return clockCommandData{}, status.Error(codes.InvalidArgument, "invalid device_id")
	}
	siteID, err := uuid.Parse(req.SiteId)
	if err != nil {
		return clockCommandData{}, status.Error(codes.InvalidArgument, "invalid site_id")
	}
	ministryID, err := uuid.Parse(req.MinistryId)
	if err != nil {
		return clockCommandData{}, status.Error(codes.InvalidArgument, "invalid ministry_id")
	}
	if req.EventTime == nil || req.EventTime.CheckValid() != nil {
		return clockCommandData{}, status.Error(codes.InvalidArgument, "valid event_time is required")
	}
	nodeID := uuid.New()
	if req.NodeId != "" {
		nodeID, err = uuid.Parse(req.NodeId)
		if err != nil {
			return clockCommandData{}, status.Error(codes.InvalidArgument, "invalid node_id")
		}
	}
	var ipAddress *string
	if req.IpAddress != "" {
		ipAddress = &req.IpAddress
	}
	return clockCommandData{
		EmployeeID: domain.EmployeeID(employeeID), MinistryID: ministryID, SiteID: siteID,
		DeviceID: domain.DeviceID(deviceID), EventTime: req.EventTime.AsTime(), Timezone: req.Timezone,
		BiometricData: req.BiometricData, Latitude: req.Latitude, Longitude: req.Longitude,
		IPAddress: ipAddress, NodeID: nodeID,
	}, nil
}

func clockResponse(event *domain.ClockEvent) *attendancepb.ClockResponse {
	return &attendancepb.ClockResponse{
		EventId: event.Identity().String(), EventType: string(event.EventType()),
		EventTime: timestamppb.New(event.EventTime().DeviceTime), RecordedAt: timestamppb.New(event.RecordedAt()),
		SyncStatus: string(event.SyncMetadata().Status),
	}
}

func mapError(err error) error {
	switch {
	case errors.Is(err, application.ErrDuplicateEvent):
		return status.Error(codes.AlreadyExists, err.Error())
	case errors.Is(err, application.ErrEventInFuture), errors.Is(err, application.ErrBiometricMismatch), errors.Is(err, application.ErrBiometricUnavailable):
		return status.Error(codes.InvalidArgument, err.Error())
	default:
		return status.Error(codes.Internal, err.Error())
	}
}

func RegisterGRPCServices(s *grpc.Server, server *AttendanceServer) {
	attendancepb.RegisterAttendanceServiceServer(s, server)
	reflection.Register(s)
}

func WithTimeout(d time.Duration) grpc.ServerOption {
	return grpc.UnaryInterceptor(func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
		ctx, cancel := context.WithTimeout(ctx, d)
		defer cancel()
		return handler(ctx, req)
	})
}
