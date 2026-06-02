package rest

import (
	"encoding/json"
	"net/http"
	"time"

	"github.com/google/uuid"
	"github.com/julienschmidt/httprouter"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/application"
	"github.com/samertts/Iraq-National-Workforce-Platform-INWP/attendance-service/internal/domain"
)

type Handler struct {
	clockInHandler      *application.ClockInHandler
	clockOutHandler     *application.ClockOutHandler
	createShiftHandler  *application.CreateShiftHandler
	setPolicyHandler    *application.SetAttendancePolicyHandler
	justifyExceptionH   *application.JustifyExceptionHandler
	resolveExceptionH   *application.ResolveExceptionHandler
	eventRepo           domain.ClockEventRepository
	shiftRepo           domain.ShiftRepository
	policyRepo          domain.AttendancePolicyRepository
	exceptionRepo       domain.AttendanceExceptionRepository
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
		clockInHandler:    clockInHandler,
		clockOutHandler:   clockOutHandler,
		createShiftHandler: createShiftHandler,
		setPolicyHandler:  setPolicyHandler,
		justifyExceptionH: justifyExceptionH,
		resolveExceptionH: resolveExceptionH,
		eventRepo:         eventRepo,
		shiftRepo:         shiftRepo,
		policyRepo:        policyRepo,
		exceptionRepo:     exceptionRepo,
	}
}

func (h *Handler) RegisterRoutes(r *httprouter.Router) {
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

	nodeID := uuid.New()

	cmd := application.ClockInCommand{
		EmployeeID:    domain.EmployeeID(uuid.MustParse(req.EmployeeID)),
		MinistryID:    uuid.MustParse(req.MinistryID),
		SiteID:        uuid.MustParse(req.SiteID),
		DeviceID:      domain.DeviceID(uuid.MustParse(req.DeviceID)),
		EventTime:     eventTime,
		Timezone:      req.Timezone,
		BiometricData: req.BiometricData,
		Latitude:      req.Latitude,
		Longitude:     req.Longitude,
		NodeID:        nodeID,
	}

	event, err := h.clockInHandler.Handle(cmd)
	if err != nil {
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
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

	nodeID := uuid.New()

	cmd := application.ClockOutCommand{
		EmployeeID:    domain.EmployeeID(uuid.MustParse(req.EmployeeID)),
		MinistryID:    uuid.MustParse(req.MinistryID),
		SiteID:        uuid.MustParse(req.SiteID),
		DeviceID:      domain.DeviceID(uuid.MustParse(req.DeviceID)),
		EventTime:     eventTime,
		Timezone:      req.Timezone,
		BiometricData: req.BiometricData,
		Latitude:      req.Latitude,
		Longitude:     req.Longitude,
		NodeID:        nodeID,
	}

	event, err := h.clockOutHandler.Handle(cmd)
	if err != nil {
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
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
	ID         string  `json:"id"`
	EmployeeID string  `json:"employee_id"`
	EventType  string  `json:"event_type"`
	EventTime  string  `json:"event_time"`
	RecordedAt string  `json:"recorded_at"`
}

func (h *Handler) ListEvents(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	employeeID := r.URL.Query().Get("employee_id")
	siteID := r.URL.Query().Get("site_id")
	fromStr := r.URL.Query().Get("from")
	toStr := r.URL.Query().Get("to")

	from := time.Now().Add(-24 * time.Hour)
	to := time.Now()

	if fromStr != "" {
		if t, err := time.Parse(time.RFC3339, fromStr); err == nil {
			from = t
		}
	}
	if toStr != "" {
		if t, err := time.Parse(time.RFC3339, toStr); err == nil {
			to = t
		}
	}

	var events []*domain.ClockEvent

	if employeeID != "" {
		eid := domain.EmployeeID(uuid.MustParse(employeeID))
		found, err := h.eventRepo.FindByEmployee(eid, from, to)
		if err != nil {
			http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
			return
		}
		events = found
	} else if siteID != "" {
		found, err := h.eventRepo.FindBySite(uuid.MustParse(siteID), from, to)
		if err != nil {
			http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
			return
		}
		events = found
	} else {
		http.Error(w, `{"error":"employee_id or site_id required"}`, http.StatusBadRequest)
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
	MinistryID     string           `json:"ministry_id"`
	SiteID         string           `json:"site_id"`
	Name           string           `json:"name"`
	StartTime      string           `json:"start_time"`
	EndTime        string           `json:"end_time"`
	GracePeriodMin int              `json:"grace_period_minutes"`
	BreakDurationMin int            `json:"break_duration_minutes"`
	OvertimePolicy domain.OvertimePolicy `json:"overtime_policy"`
}

func (h *Handler) CreateShift(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	var req createShiftRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"invalid request body"}`, http.StatusBadRequest)
		return
	}

	startTime, _ := time.Parse("15:04", req.StartTime)
	endTime, _ := time.Parse("15:04", req.EndTime)

	cmd := application.CreateShiftCommand{
		MinistryID:     uuid.MustParse(req.MinistryID),
		SiteID:         uuid.MustParse(req.SiteID),
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
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
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
		http.Error(w, `{"error":"site_id required"}`, http.StatusBadRequest)
		return
	}

	shifts, err := h.shiftRepo.FindBySite(uuid.MustParse(siteID))
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
			ID:   s.Identity().String(),
			Name: s.Name(),
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

	effectiveFrom, _ := time.Parse(time.RFC3339, req.EffectiveFrom)
	siteID := uuid.MustParse(req.SiteID)

	cmd := application.SetAttendancePolicyCommand{
		MinistryID:    uuid.MustParse(req.MinistryID),
		SiteID:        &siteID,
		Name:          req.Name,
		Rules:         req.Rules,
		EffectiveFrom: effectiveFrom,
		ApprovedBy:    uuid.MustParse(req.ApprovedBy),
	}

	policy, err := h.setPolicyHandler.Handle(cmd)
	if err != nil {
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
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

	policies, err := h.policyRepo.FindHistoryBySite(uuid.MustParse(siteID))
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
			ID:   p.Identity().String(),
			Name: p.Name(),
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

	cmd := application.JustifyExceptionCommand{
		ExceptionID: uuid.MustParse(req.ExceptionID),
		Reason:      req.Reason,
		Type:        req.Type,
	}

	if err := h.justifyExceptionH.Handle(cmd); err != nil {
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
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

	cmd := application.ResolveExceptionCommand{
		ExceptionID: uuid.MustParse(req.ExceptionID),
		ResolvedBy:  uuid.MustParse(req.ResolvedBy),
	}

	if err := h.resolveExceptionH.Handle(cmd); err != nil {
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "resolved"})
}
