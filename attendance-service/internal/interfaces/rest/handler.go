package rest

import (
	"encoding/json"
	"errors"
	"net/http"
	"time"

	"github.com/google/uuid"
	"github.com/julienschmidt/httprouter"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/application"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

type Handler struct {
	clockInHandler     *application.ClockInHandler
	clockOutHandler    *application.ClockOutHandler
	createShiftHandler *application.CreateShiftHandler
	setPolicyHandler   *application.SetAttendancePolicyHandler
	justifyExceptionH  *application.JustifyExceptionHandler
	resolveExceptionH  *application.ResolveExceptionHandler
	eventRepo          domain.ClockEventRepository
	shiftRepo          domain.ShiftRepository
	policyRepo         domain.AttendancePolicyRepository
	exceptionRepo      domain.AttendanceExceptionRepository
}

func NewHandler(
	clockInHandler *application.ClockInHandler,
	clockOutHandler *application.ClockOutHandler,
	createShiftHandler *application.CreateShiftHandler,
	setPolicyHandler *application.SetAttendancePolicyHandler,
	justifyExceptionH *application.JustifyExceptionHandler,
	resolveExceptionH *application.ResolveExceptionHandler,
	eventRepo domain.ClockEventRepository,
	shiftRepo domain.ShiftRepository,
	policyRepo domain.AttendancePolicyRepository,
	exceptionRepo domain.AttendanceExceptionRepository,
) *Handler {
	return &Handler{
		clockInHandler:     clockInHandler,
		clockOutHandler:    clockOutHandler,
		createShiftHandler: createShiftHandler,
		setPolicyHandler:   setPolicyHandler,
		justifyExceptionH:  justifyExceptionH,
		resolveExceptionH:  resolveExceptionH,
		eventRepo:          eventRepo,
		shiftRepo:          shiftRepo,
		policyRepo:         policyRepo,
		exceptionRepo:      exceptionRepo,
	}
}

func (h *Handler) RegisterRoutes(r *httprouter.Router) {
	r.GET("/healthz", h.Health)
	r.GET("/readyz", h.Ready)
	r.POST("/api/v1/attendance/clock-in", h.ClockIn)
	r.POST("/api/v1/attendance/clock-out", h.ClockOut)
	r.GET("/api/v1/attendance/events", h.ListEvents)
	r.GET("/api/v1/attendance/events/:id", h.GetEvent)
	r.POST("/api/v1/shifts", h.CreateShift)
	r.GET("/api/v1/shifts", h.ListShifts)
	r.POST("/api/v1/policies", h.SetPolicy)
	r.GET("/api/v1/policies", h.ListPolicies)
	r.POST("/api/v1/exceptions/justify", h.JustifyException)
	r.POST("/api/v1/exceptions/resolve", h.ResolveException)
}

type clockInRequest struct {
	EmployeeID    string   `json:"employee_id"`
	MinistryID    string   `json:"ministry_id"`
	SiteID        string   `json:"site_id"`
	DeviceID      string   `json:"device_id"`
	EventTime     string   `json:"event_time"`
	Timezone      string   `json:"timezone"`
	BiometricData []byte   `json:"biometric_data,omitempty"`
	Latitude      *float64 `json:"latitude,omitempty"`
	Longitude     *float64 `json:"longitude,omitempty"`
}

type clockInResponse struct {
	EventID   string `json:"event_id"`
	EventType string `json:"event_type"`
	CreatedAt string `json:"created_at"`
}

func (h *Handler) Health(w http.ResponseWriter, _ *http.Request, _ httprouter.Params) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte(`{"status":"ok","service":"attendance-service"}`))
}

func (h *Handler) Ready(w http.ResponseWriter, _ *http.Request, _ httprouter.Params) {
	if h.eventRepo == nil {
		writeProblem(w, http.StatusServiceUnavailable, "not_ready", "event repository is not configured")
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte(`{"status":"ready"}`))
}

func (h *Handler) ClockIn(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	var req clockInRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"invalid request body"}`, http.StatusBadRequest)
		return
	}

	eventTime, err := time.Parse(time.RFC3339, req.EventTime)
	if err != nil {
		http.Error(w, `{"error":"invalid event_time format"}`, http.StatusBadRequest)
		return
	}

	employeeID, ok := parseUUIDField(w, "employee_id", req.EmployeeID)
	if !ok {
		return
	}
	ministryID, ok := parseUUIDField(w, "ministry_id", req.MinistryID)
	if !ok {
		return
	}
	siteID, ok := parseUUIDField(w, "site_id", req.SiteID)
	if !ok {
		return
	}
	deviceID, ok := parseUUIDField(w, "device_id", req.DeviceID)
	if !ok {
		return
	}

	nodeID := uuid.New()
	cmd := application.ClockInCommand{
		EmployeeID:    domain.EmployeeID(employeeID),
		MinistryID:    ministryID,
		SiteID:        siteID,
		DeviceID:      domain.DeviceID(deviceID),
		EventTime:     eventTime,
		Timezone:      req.Timezone,
		BiometricData: req.BiometricData,
		Latitude:      req.Latitude,
		Longitude:     req.Longitude,
		NodeID:        nodeID,
	}

	event, err := h.clockInHandler.Handle(cmd)
	if err != nil {
		writeHandlerError(w, err)
		return
	}

	resp := clockInResponse{
		EventID:   event.Identity().String(),
		EventType: string(event.EventType()),
		CreatedAt: event.RecordedAt().Format(time.RFC3339),
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(resp)
}

type clockOutRequest struct {
	EmployeeID    string   `json:"employee_id"`
	MinistryID    string   `json:"ministry_id"`
	SiteID        string   `json:"site_id"`
	DeviceID      string   `json:"device_id"`
	EventTime     string   `json:"event_time"`
	Timezone      string   `json:"timezone"`
	BiometricData []byte   `json:"biometric_data,omitempty"`
	Latitude      *float64 `json:"latitude,omitempty"`
	Longitude     *float64 `json:"longitude,omitempty"`
}

func (h *Handler) ClockOut(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	var req clockOutRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"invalid request body"}`, http.StatusBadRequest)
		return
	}

	eventTime, err := time.Parse(time.RFC3339, req.EventTime)
	if err != nil {
		http.Error(w, `{"error":"invalid event_time format"}`, http.StatusBadRequest)
		return
	}

	employeeID, ok := parseUUIDField(w, "employee_id", req.EmployeeID)
	if !ok {
		return
	}
	ministryID, ok := parseUUIDField(w, "ministry_id", req.MinistryID)
	if !ok {
		return
	}
	siteID, ok := parseUUIDField(w, "site_id", req.SiteID)
	if !ok {
		return
	}
	deviceID, ok := parseUUIDField(w, "device_id", req.DeviceID)
	if !ok {
		return
	}

	nodeID := uuid.New()
	cmd := application.ClockOutCommand{
		EmployeeID:    domain.EmployeeID(employeeID),
		MinistryID:    ministryID,
		SiteID:        siteID,
		DeviceID:      domain.DeviceID(deviceID),
		EventTime:     eventTime,
		Timezone:      req.Timezone,
		BiometricData: req.BiometricData,
		Latitude:      req.Latitude,
		Longitude:     req.Longitude,
		NodeID:        nodeID,
	}

	event, err := h.clockOutHandler.Handle(cmd)
	if err != nil {
		writeHandlerError(w, err)
		return
	}

	resp := clockInResponse{
		EventID:   event.Identity().String(),
		EventType: string(event.EventType()),
		CreatedAt: event.RecordedAt().Format(time.RFC3339),
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(resp)
}

type listEventsResponse struct {
	Events []eventResponse `json:"events"`
}

type eventResponse struct {
	ID         string `json:"id"`
	EmployeeID string `json:"employee_id"`
	EventType  string `json:"event_type"`
	EventTime  string `json:"event_time"`
	RecordedAt string `json:"recorded_at"`
}

func (h *Handler) ListEvents(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	employeeID := r.URL.Query().Get("employee_id")
	siteID := r.URL.Query().Get("site_id")
	fromStr := r.URL.Query().Get("from")
	toStr := r.URL.Query().Get("to")

	from := time.Now().UTC().Add(-24 * time.Hour)
	to := time.Now().UTC()

	if fromStr != "" {
		t, err := time.Parse(time.RFC3339, fromStr)
		if err != nil {
			writeProblem(w, http.StatusBadRequest, "invalid_request", "invalid from")
			return
		}
		from = t.UTC()
	}
	if toStr != "" {
		t, err := time.Parse(time.RFC3339, toStr)
		if err != nil {
			writeProblem(w, http.StatusBadRequest, "invalid_request", "invalid to")
			return
		}
		to = t.UTC()
	}
	if from.After(to) {
		writeProblem(w, http.StatusBadRequest, "invalid_request", "from must be before to")
		return
	}

	var events []*domain.ClockEvent

	if employeeID != "" && siteID != "" {
		writeProblem(w, http.StatusBadRequest, "invalid_request", "provide only one of employee_id or site_id")
		return
	}
	if employeeID != "" {
		eid, ok := parseUUIDField(w, "employee_id", employeeID)
		if !ok {
			return
		}
		found, err := h.eventRepo.FindByEmployee(domain.EmployeeID(eid), from, to)
		if err != nil {
			http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
			return
		}
		events = found
	} else if siteID != "" {
		sid, ok := parseUUIDField(w, "site_id", siteID)
		if !ok {
			return
		}
		found, err := h.eventRepo.FindBySite(sid, from, to)
		if err != nil {
			http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
			return
		}
		events = found
	} else {
		writeProblem(w, http.StatusBadRequest, "invalid_request", "employee_id or site_id required")
		return
	}

	resp := listEventsResponse{Events: make([]eventResponse, 0, len(events))}
	for _, e := range events {
		resp.Events = append(resp.Events, eventResponse{
			ID:         e.Identity().String(),
			EmployeeID: uuid.UUID(e.EmployeeID()).String(),
			EventType:  string(e.EventType()),
			EventTime:  e.EventTime().DeviceTime.Format(time.RFC3339),
			RecordedAt: e.RecordedAt().Format(time.RFC3339),
		})
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp)
}

func (h *Handler) GetEvent(w http.ResponseWriter, r *http.Request, ps httprouter.Params) {
	id, err := uuid.Parse(ps.ByName("id"))
	if err != nil {
		http.Error(w, `{"error":"invalid event id"}`, http.StatusBadRequest)
		return
	}

	event, err := h.eventRepo.FindByID(id)
	if err != nil {
		http.Error(w, `{"error":"event not found"}`, http.StatusNotFound)
		return
	}
	if event == nil {
		http.Error(w, `{"error":"event not found"}`, http.StatusNotFound)
		return
	}

	resp := eventResponse{
		ID:         event.Identity().String(),
		EmployeeID: uuid.UUID(event.EmployeeID()).String(),
		EventType:  string(event.EventType()),
		EventTime:  event.EventTime().DeviceTime.Format(time.RFC3339),
		RecordedAt: event.RecordedAt().Format(time.RFC3339),
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp)
}

type createShiftRequest struct {
	MinistryID       string                `json:"ministry_id"`
	SiteID           string                `json:"site_id"`
	Name             string                `json:"name"`
	StartTime        string                `json:"start_time"`
	EndTime          string                `json:"end_time"`
	GracePeriodMin   int                   `json:"grace_period_minutes"`
	BreakDurationMin int                   `json:"break_duration_minutes"`
	OvertimePolicy   domain.OvertimePolicy `json:"overtime_policy"`
}

func (h *Handler) CreateShift(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	var req createShiftRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"invalid request body"}`, http.StatusBadRequest)
		return
	}

	startTime, err := time.Parse("15:04", req.StartTime)
	if err != nil {
		writeProblem(w, http.StatusBadRequest, "invalid_request", "invalid start_time")
		return
	}
	endTime, err := time.Parse("15:04", req.EndTime)
	if err != nil {
		writeProblem(w, http.StatusBadRequest, "invalid_request", "invalid end_time")
		return
	}
	ministryID, ok := parseUUIDField(w, "ministry_id", req.MinistryID)
	if !ok {
		return
	}
	siteID, ok := parseUUIDField(w, "site_id", req.SiteID)
	if !ok {
		return
	}

	cmd := application.CreateShiftCommand{
		MinistryID:     ministryID,
		SiteID:         siteID,
		Name:           req.Name,
		StartTime:      startTime,
		EndTime:        endTime,
		GracePeriod:    time.Duration(req.GracePeriodMin) * time.Minute,
		BreakDuration:  time.Duration(req.BreakDurationMin) * time.Minute,
		ApplicableDays: []time.Weekday{time.Sunday, time.Monday, time.Tuesday, time.Wednesday, time.Thursday},
		OvertimePolicy: req.OvertimePolicy,
	}

	shift, err := h.createShiftHandler.Handle(cmd)
	if err != nil {
		writeHandlerError(w, err)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(map[string]string{
		"shift_id": shift.Identity().String(),
		"name":     shift.Name(),
	})
}

func (h *Handler) ListShifts(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	siteID := r.URL.Query().Get("site_id")
	if siteID == "" {
		writeProblem(w, http.StatusBadRequest, "invalid_request", "site_id required")
		return
	}
	sid, ok := parseUUIDField(w, "site_id", siteID)
	if !ok {
		return
	}

	shifts, err := h.shiftRepo.FindBySite(sid)
	if err != nil {
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
		return
	}

	type shiftResponse struct {
		ID        string `json:"id"`
		Name      string `json:"name"`
		StartTime string `json:"start_time"`
		EndTime   string `json:"end_time"`
		IsActive  bool   `json:"is_active"`
	}

	resp := make([]shiftResponse, 0, len(shifts))
	for _, s := range shifts {
		resp = append(resp, shiftResponse{
			ID:        s.Identity().String(),
			Name:      s.Name(),
			StartTime: s.StartTime().Format("15:04"),
			EndTime:   s.EndTime().Format("15:04"),
			IsActive:  s.IsActive(),
		})
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp)
}

type setPolicyRequest struct {
	MinistryID    string                   `json:"ministry_id"`
	SiteID        string                   `json:"site_id"`
	Name          string                   `json:"name"`
	Rules         domain.AttendanceRuleSet `json:"rules"`
	EffectiveFrom string                   `json:"effective_from"`
	ApprovedBy    string                   `json:"approved_by"`
}

func (h *Handler) SetPolicy(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	var req setPolicyRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"invalid request body"}`, http.StatusBadRequest)
		return
	}

	effectiveFrom, err := time.Parse(time.RFC3339, req.EffectiveFrom)
	if err != nil {
		writeProblem(w, http.StatusBadRequest, "invalid_request", "invalid effective_from")
		return
	}
	ministryID, ok := parseUUIDField(w, "ministry_id", req.MinistryID)
	if !ok {
		return
	}
	siteID, ok := parseUUIDField(w, "site_id", req.SiteID)
	if !ok {
		return
	}
	approvedBy, ok := parseUUIDField(w, "approved_by", req.ApprovedBy)
	if !ok {
		return
	}

	cmd := application.SetAttendancePolicyCommand{
		MinistryID:    ministryID,
		SiteID:        &siteID,
		Name:          req.Name,
		Rules:         req.Rules,
		EffectiveFrom: effectiveFrom,
		ApprovedBy:    approvedBy,
	}

	policy, err := h.setPolicyHandler.Handle(cmd)
	if err != nil {
		writeHandlerError(w, err)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(map[string]string{
		"policy_id": policy.Identity().String(),
		"name":      policy.Name(),
	})
}

func (h *Handler) ListPolicies(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	siteID := r.URL.Query().Get("site_id")
	if siteID == "" {
		http.Error(w, `{"error":"site_id required"}`, http.StatusBadRequest)
		return
	}

	sid, ok := parseUUIDField(w, "site_id", siteID)
	if !ok {
		return
	}
	policies, err := h.policyRepo.FindHistoryBySite(sid)
	if err != nil {
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
		return
	}

	type policyResponse struct {
		ID            string `json:"id"`
		Name          string `json:"name"`
		EffectiveFrom string `json:"effective_from"`
		EffectiveTo   string `json:"effective_to,omitempty"`
	}

	resp := make([]policyResponse, 0, len(policies))
	for _, p := range policies {
		pr := policyResponse{
			ID:            p.Identity().String(),
			Name:          p.Name(),
			EffectiveFrom: p.EffectiveFrom().Format(time.RFC3339),
		}
		if et := p.EffectiveTo(); et != nil {
			pr.EffectiveTo = et.Format(time.RFC3339)
		}
		resp = append(resp, pr)
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp)
}

type justifyRequest struct {
	ExceptionID string                   `json:"exception_id"`
	Reason      string                   `json:"reason"`
	Type        domain.JustificationType `json:"type"`
}

func (h *Handler) JustifyException(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	var req justifyRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"invalid request body"}`, http.StatusBadRequest)
		return
	}

	exceptionID, ok := parseUUIDField(w, "exception_id", req.ExceptionID)
	if !ok {
		return
	}
	cmd := application.JustifyExceptionCommand{
		ExceptionID: exceptionID,
		Reason:      req.Reason,
		Type:        req.Type,
	}

	if err := h.justifyExceptionH.Handle(cmd); err != nil {
		writeHandlerError(w, err)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "justified"})
}

type resolveRequest struct {
	ExceptionID string `json:"exception_id"`
	ResolvedBy  string `json:"resolved_by"`
}

func (h *Handler) ResolveException(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	var req resolveRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"invalid request body"}`, http.StatusBadRequest)
		return
	}

	exceptionID, ok := parseUUIDField(w, "exception_id", req.ExceptionID)
	if !ok {
		return
	}
	resolvedBy, ok := parseUUIDField(w, "resolved_by", req.ResolvedBy)
	if !ok {
		return
	}
	cmd := application.ResolveExceptionCommand{
		ExceptionID: exceptionID,
		ResolvedBy:  resolvedBy,
	}

	if err := h.resolveExceptionH.Handle(cmd); err != nil {
		writeHandlerError(w, err)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "resolved"})
}

func parseUUIDField(w http.ResponseWriter, field, value string) (uuid.UUID, bool) {
	parsed, err := uuid.Parse(value)
	if err != nil {
		writeProblem(w, http.StatusBadRequest, "invalid_request", "invalid "+field)
		return uuid.Nil, false
	}
	return parsed, true
}

func writeProblem(w http.ResponseWriter, status int, code, detail string) {
	w.Header().Set("Content-Type", "application/problem+json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(map[string]any{
		"type":   "https://inwp.local/problems/" + code,
		"title":  http.StatusText(status),
		"status": status,
		"detail": detail,
	})
}

func writeHandlerError(w http.ResponseWriter, err error) {
	status := http.StatusInternalServerError
	switch {
	case errors.Is(err, application.ErrExceptionNotFound):
		status = http.StatusNotFound
	case errors.Is(err, application.ErrDuplicateEvent):
		status = http.StatusConflict
	case errors.Is(err, application.ErrEventInFuture),
		errors.Is(err, application.ErrBiometricMismatch),
		errors.Is(err, application.ErrBiometricUnavailable):
		status = http.StatusUnprocessableEntity
	}
	writeProblem(w, status, "operation_failed", err.Error())
}
