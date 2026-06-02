package main

import (
	"context"
	"fmt"
	"log"
	"net"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/julienschmidt/httprouter"
	"github.com/nats-io/nats.go"
	"github.com/rs/zerolog"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/application"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/infrastructure/eventbus"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/infrastructure/postgres"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/infrastructure/sync"
	grpciface "github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/interfaces/grpc"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/interfaces/rest"
)

func main() {
	logger := zerolog.New(os.Stdout).With().Timestamp().Logger()

	cfg := loadConfig()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	pgPool, err := pgxpool.New(ctx, cfg.DatabaseURL)
	if err != nil {
		logger.Fatal().Err(err).Msg("failed to connect to postgres")
	}
	defer pgPool.Close()

	var nc *nats.Conn
	if cfg.NATSURL != "" {
		nc, err = nats.Connect(cfg.NATSURL)
		if err != nil {
			logger.Warn().Err(err).Msg("nats not available, running without event bus")
		} else {
			defer nc.Close()
		}
	}

	var eventPub application.EventPublisher
	if nc != nil {
		eventPub = eventbus.NewNATSPublisher(nc)
	} else {
		eventPub = &eventbus.NoopPublisher{}
	}

	eventRepo := postgres.NewClockEventRepository(pgPool)
	shiftRepo := postgres.NewShiftRepository(pgPool)
	policyRepo := postgres.NewPolicyRepository(pgPool)
	exceptionRepo := postgres.NewExceptionRepository(pgPool)
	syncOutbox := sync.NewOutbox(pgPool)

	duplicateDetector := &basicDuplicateDetector{repo: eventRepo}

	clockInHandler := application.NewClockInHandler(
		eventRepo, shiftRepo, policyRepo,
		duplicateDetector, nil, eventPub, syncOutbox,
	)
	clockOutHandler := application.NewClockOutHandler(
		eventRepo, duplicateDetector, nil, eventPub, syncOutbox,
	)
	createShiftHandler := application.NewCreateShiftHandler(shiftRepo, eventPub)
	setPolicyHandler := application.NewSetAttendancePolicyHandler(policyRepo, eventPub)
	justifyExceptionH := application.NewJustifyExceptionHandler(exceptionRepo, eventPub)
	resolveExceptionH := application.NewResolveExceptionHandler(exceptionRepo, eventPub)

	router := httprouter.New()
	restHandler := rest.NewHandler(
		clockInHandler, clockOutHandler,
		createShiftHandler, setPolicyHandler,
		justifyExceptionH, resolveExceptionH,
		eventRepo, shiftRepo, policyRepo, exceptionRepo,
	)
	restHandler.RegisterRoutes(router)

	httpServer := &http.Server{
		Addr:         fmt.Sprintf(":%s", cfg.HTTPPort),
		Handler:      router,
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 15 * time.Second,
		IdleTimeout:  60 * time.Second,
	}

	grpcServer := grpc.NewServer(
		grpc.Creds(insecure.NewCredentials()),
		grpciface.WithTimeout(30*time.Second),
	)
	attendanceServer := grpciface.NewAttendanceServer(
		clockInHandler, clockOutHandler, eventRepo,
	)
	grpciface.RegisterGRPCServices(grpcServer, attendanceServer)

	grpcListener, err := net.Listen("tcp", fmt.Sprintf(":%s", cfg.GRPCPort))
	if err != nil {
		logger.Fatal().Err(err).Msg("failed to listen on gRPC port")
	}

	go func() {
		logger.Info().Str("port", cfg.HTTPPort).Msg("starting HTTP server")
		if err := httpServer.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			logger.Fatal().Err(err).Msg("HTTP server failed")
		}
	}()

	go func() {
		logger.Info().Str("port", cfg.GRPCPort).Msg("starting gRPC server")
		if err := grpcServer.Serve(grpcListener); err != nil {
			logger.Fatal().Err(err).Msg("gRPC server failed")
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	logger.Info().Msg("shutting down servers...")

	grpcServer.GracefulStop()

	shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer shutdownCancel()

	if err := httpServer.Shutdown(shutdownCtx); err != nil {
		logger.Error().Err(err).Msg("HTTP server forced shutdown")
	}

	logger.Info().Msg("servers stopped")
}

type config struct {
	DatabaseURL string
	NATSURL     string
	HTTPPort    string
	GRPCPort    string
	NodeID      string
}

func loadConfig() config {
	return config{
		DatabaseURL: getEnv("DATABASE_URL", "postgres://inwp:inwp@localhost:5432/inwp?sslmode=disable"),
		NATSURL:     getEnv("NATS_URL", "nats://localhost:4222"),
		HTTPPort:    getEnv("HTTP_PORT", "8080"),
		GRPCPort:    getEnv("GRPC_PORT", "9090"),
		NodeID:      getEnv("NODE_ID", "edge-node-01"),
	}
}

func getEnv(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

type basicDuplicateDetector struct {
	repo *postgres.ClockEventRepository
}

func (d *basicDuplicateDetector) IsDuplicate(employeeID domain.EmployeeID, deviceID domain.DeviceID, eventType domain.ClockEventType, eventTime time.Time) (bool, error) {
	return d.repo.ExistsDuplicate(employeeID, deviceID, eventType, eventTime)
}

func (d *basicDuplicateDetector) FindNearDuplicates(employeeID domain.EmployeeID, timeWindow time.Duration) ([]*domain.ClockEvent, error) {
	log.Printf("FindNearDuplicates called for employee %v with window %v", employeeID, timeWindow)
	return nil, nil
}
