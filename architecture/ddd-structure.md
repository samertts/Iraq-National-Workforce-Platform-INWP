# INWP Domain-Driven Design Structure

> Complete DDD model for all core services: bounded contexts, aggregates, entities, value objects, domain events, domain services, repositories, invariants, and application services.

---

## Table of Contents

1. [Strategic Design](#1-strategic-design)
2. [platform-core (Shared Kernel)](#2-platform-core-shared-kernel)
3. [identity-service](#3-identity-service)
4. [attendance-service](#4-attendance-service)
5. [leave-service](#5-leave-service)
6. [audit-ledger](#6-audit-ledger)
7. [sync-engine](#7-sync-engine)
8. [notification-service](#8-notification-service)
9. [Context Maps](#9-context-maps)
10. [Event Storming Summary](#10-event-storming-summary)

---

## 1. Strategic Design

### 1.1 Bounded Context Map

```
+---------------------------+       +---------------------------+
|        Identity           |<------|       Attendance          |
|  (User, Role, Device,    |       |  (ClockEvent, Shift,      |
|   Session, Realm)        |       |   Policy, Exception)      |
+---------------------------+       +---------------------------+
         |                                     |
         | events                              | events
         v                                     v
+---------------------------+       +---------------------------+
|      Sync Engine          |<------|       Leave               |
|  (MerkleTree, Checkpoint, |       |  (LeaveRequest, Balance,  |
|   Conflict, Batch)        |       |   Approval, Accrual)      |
+---------------------------+       +---------------------------+
         |                                     |
         | events                              | events
         v                                     v
+---------------------------+       +---------------------------+
|      Audit Ledger         |       |    Notification           |
|  (LedgerEntry, HashChain, |       |  (Notification, Template, |
|   Seal, IntegrityProof)   |       |   Channel, Delivery)      |
+---------------------------+       +---------------------------+
         |
         | shared
         v
+---------------------------+
|      Platform Core        |
|  (Shared Kernel:          |
|   BaseEvent, Ministry,    |
|   Site, Employee, Device, |
|   Crypto, Validation)     |
+---------------------------+
```

### 1.2 Ubiquitous Language Glossary

| Term | Definition | Contexts |
|---|---|---|
| **Ministry** | A sovereign tenant within INWP (e.g., MoHE, MoH). Has its own realm, policies, encryption keys. | All |
| **Site** | A physical location (university, hospital, office). Belongs to exactly one ministry. | Identity, Attendance, Leave |
| **Employee** | A person employed by a ministry. Has a national ID, biometric binding, and employment record. | Identity, Attendance, Leave |
| **Device** | A hardware endpoint (biometric scanner, card reader, mobile) enrolled to a site. | Identity, Attendance |
| **Clock Event** | A timestamped record of an employee clocking in or out at a device. | Attendance |
| **Leave Request** | An application for time off, with approval chain and balance impact. | Leave |
| **Accrual Policy** | Rules governing how leave balance accumulates over time. | Leave |
| **Realm** | An isolated identity domain for one ministry with its own user directory, roles, and authentication policies. | Identity |
| **Trust Level** | A computed score (0.0–1.0) representing how trustworthy a device is. | Identity |
| **Sync Batch** | A signed set of events exchanged between two nodes during synchronization. | Sync |
| **Merkle Partition** | A logical shard of the sync data space (ministry + entity type + time bucket). | Sync |
| **Ledger Entry** | A single append-only record in the audit chain, cryptographically linked to its predecessor. | Audit |
| **Hash Chain** | The sequence of cryptographic hashes linking all ledger entries, forming a tamper-evident log. | Audit |

### 1.3 Aggregate Root Identification

| Aggregate Root | Context | Why Aggregate? | Concurrency |
|---|---|---|---|
| `User` | Identity | Owns credentials, profile, device bindings | Optimistic concurrency (version field) |
| `Device` | Identity | Owns enrollment state, certificate, trust level | Optimistic concurrency |
| `Realm` | Identity | Owns auth policies, role catalog | Rarely modified; pessimistic lock |
| `ClockEvent` | Attendance | Single immutable record; no children | Append-only; no conflict |
| `Shift` | Attendance | Owns shift rules, exceptions | Optimistic concurrency |
| `AttendancePolicy` | Attendance | Owns site-level attendance rules | Rarely modified; read replica |
| `LeaveRequest` | Leave | Owns approval chain, status, balance impact | Optimistic concurrency |
| `LeaveBalance` | Leave | Owns accrual history, deductions | Pessimistic (financial accuracy) |
| `AccrualPolicy` | Leave | Owns accrual rules | Rarely modified |
| `SyncBatch` | Sync | Owns merkle roots, event manifests | Append-only |
| `MerkleTree` | Sync | Owns node hierarchy, leaf hashes | Optimistic concurrency |
| `LedgerEntry` | Audit | Single immutable record | Append-only |
| `Notification` | Notification | Owns delivery state, retry count | Optimistic concurrency |
| `Template` | Notification | Owns content, channels | Rarely modified |

---

## 2. platform-core (Shared Kernel)

> **Strategic role**: Shared Kernel — domain concepts shared across all bounded contexts.
>
> **Language**: Go package `inwp/pkg/domain`
>
> **Key rule**: NO business logic. Only shared types, base interfaces, value objects, and validation.

### 2.1 Value Objects

```go
// --- Core Identifiers ---

type MinistryID valueObject {
    value: uuid.UUID
    // Invariant: must be a valid UUID v7
}

type SiteID valueObject {
    value: uuid.UUID
}

type EmployeeID valueObject {
    value: uuid.UUID
}

type DeviceID valueObject {
    value: uuid.UUID
}

type UserID valueObject {
    value: uuid.UUID
}

type ServiceID valueObject {
    name: string        // "attendance-service"
    instanceID: string  // unique instance identifier
}

// --- Geographic ---

type GeoLocation valueObject {
    latitude:  float64
    longitude: float64
    accuracy:  float64  // meters
}

// --- Time ---

type OfflineTimestamp valueObject {
    deviceTime:  time.Time   // time on the device when event occurred
    serverTime:  time.Time   // time when server received it (optional)
    timezone:    string      // IANA timezone, e.g. "Asia/Baghdad"
    // Invariant: deviceTime must not be zero
}

// --- Identity ---

type CredentialType enum {
    PASSWORD
    TOTP
    BIOMETRIC_FINGERPRINT
    BIOMETRIC_FACE
    SMART_CARD
    DEVICE_CERTIFICATE
}

type AuthMethod valueObject {
    primary:   CredentialType
    secondary: CredentialType?  // MFA
}

type TrustLevel enum {
    TRUSTED
    PROVISIONAL
    DEGRADED
    SUSPENDED
    REVOKED
}

// --- Security ---

type Signature valueObject {
    keyID:      string          // which key signed it
    algorithm:  string          // "Ed25519"
    value:      []byte
    signedAt:   time.Time
}

type EncryptedPayload valueObject {
    encryptedKey:   []byte      // wrapped CEK
    ciphertext:     []byte      // AES-256-GCM
    nonce:          []byte      // 12-byte GCM nonce
    algorithm:      string      // "AES-256-GCM"
}

// --- Sync ---

type SyncStatus enum {
    LOCAL_ONLY       // not yet synced
    SYNCED           // confirmed by at least one peer
    CONFLICTED       // conflict detected, awaiting resolution
    RESOLVED         // conflict resolved
}

type SyncMetadata valueObject {
    syncID:        uuid.UUID
    sourceNodeID:  uuid.UUID
    status:        SyncStatus
    lastSyncedAt:  time.Time?
    version:       int64        // monotonic version vector
}

// --- Base Event ---

type DomainEvent interface {
    EventID()     uuid.UUID
    EventType()   string        // "inwp.attendance.v1.clock-in.created"
    EventVersion() string       // "1.0.0"
    OccurredAt()  time.Time
    Source()      string        // "/ministries/{id}/sites/{id}/services/{name}"
    MinistryID()  uuid.UUID
    SiteID()      uuid.UUID?
    Payload()     []byte
    Signature()   Signature?
}
```

### 2.2 Domain Interfaces

```go
// Entity — anything with an identity
type Entity interface {
    Identity() uuid.UUID
}

// AggregateRoot — entity that owns other entities
type AggregateRoot interface {
    Entity
    Version() int64
    DomainEvents() []DomainEvent
    ClearEvents()
}

// ValueObject — immutable by value
type ValueObject interface {
    Equals(other ValueObject) bool
}

// HasMinistry — scoped to a ministry tenant
type HasMinistry interface {
    MinistryID() uuid.UUID
}

// HasSite — scoped to a site
type HasSite interface {
    MinistryID() uuid.UUID
    SiteID() uuid.UUID
}

// Auditable — can produce audit events
type Auditable interface {
    ToAuditEvent() DomainEvent
}
```

### 2.3 Shared Validation

```go
type ValidationRule interface {
    Validate(ctx context.Context) ValidationResult
}

type ValidationResult struct {
    Valid   bool
    Errors  []ValidationError
}

type ValidationError struct {
    Field   string
    Code    string     // "required", "invalid_format", "out_of_range"
    Message string
}
```

---

## 3. identity-service

> **Bounded Context**: Identity & Access Management
>
> **Domain**: User registration, authentication, authorization, device management, realm administration, session management.
>
> **Language**: Go package `internal/domain/`

### 3.1 Aggregates

#### Aggregate: `User`

```go
// User aggregate root
// Invariants:
//   - Must have exactly one primary credential
//   - Must belong to exactly one realm (ministry)
//   - NationalID must be unique within a ministry
//   - Cannot be deactivated if has active sessions
//   - Device bindings limited to 10 per user (configurable per ministry)
type User struct {
    id              UserID
    realmID         uuid.UUID          // ministry realm
    nationalID      string             // Iraqi national ID number
    fullName        PersonName         // value object
    email           EmailAddress?      // value object
    phone           PhoneNumber?       // value object
    employmentInfo  EmploymentInfo     // value object
    credentials     []Credential       // entity list
    deviceBindings  []DeviceBinding    // value object list
    roles           []RoleAssignment   // value object list
    status          UserStatus
    trustScore      float64            // 0.0–1.0
    version         int64
    domainEvents    []DomainEvent
}

type PersonName valueObject {
    firstName:  string
    fatherName: string
    lastName:   string
    // Invariant: 2–100 characters per component, no digits or special chars
}

type EmailAddress valueObject {
    value: string
    // Invariant: valid email per RFC 5321
}

type PhoneNumber valueObject {
    countryCode: string   // "+964"
    number:      string   // "7xx xxx xxxx"
    // Invariant: valid Iraqi phone number format
}

type EmploymentInfo valueObject {
    employeeID:     string           // ministry-specific employee code
    ministryID:     uuid.UUID
    siteID:         uuid.UUID
    department:     string
    position:       string
    employmentType: EmploymentType   // FULL_TIME, PART_TIME, CONTRACTOR
    joinedAt:       time.Time
}

type EmploymentType enum {
    FULL_TIME
    PART_TIME
    CONTRACTOR
    TEMPORARY
}

type UserStatus enum {
    ACTIVE
    INACTIVE        // temporarily disabled
    DEACTIVATED     // permanently disabled
    SUSPENDED       // security suspension
    PENDING_VERIFICATION
}

// Entity inside User aggregate
type Credential struct {
    id:            uuid.UUID
    credentialType: CredentialType
    identifier:    string             // username, cert serial, etc.
    hash:          []byte?            // for passwords (Argon2id)
    publicKey:     []byte?            // for certificates/biometric
    enrolledAt:    time.Time
    lastUsedAt:    time.Time?
    expiresAt:     time.Time?
    isPrimary:     bool
    // Invariant: exactly one credential must be isPrimary
    // Invariant: password credentials require hash
    // Invariant: certificate credentials require publicKey
}

type RoleAssignment valueObject {
    roleID:   uuid.UUID
    scope:    RoleScope
    assignedAt: time.Time
    assignedBy: uuid.UUID   // admin user ID
    expiresAt:  time.Time?
}

type RoleScope valueObject {
    type:  ScopeType    // NATIONAL, MINISTRY, SITE
    ministryID: uuid.UUID?
    siteID:     uuid.UUID?
}

type DeviceBinding valueObject {
    deviceID:     uuid.UUID
    boundAt:      time.Time
    trustLevel:   TrustLevel
    purpose:      string     // "attendance", "auth", "both"
}

// User Domain Events
type UserRegistered struct        { UserID, RealmID, RegisteredAt }
type UserVerified struct          { UserID, VerifiedAt }
type UserDeactivated struct       { UserID, Reason, DeactivatedAt }
type CredentialAdded struct       { UserID, CredentialType, AddedAt }
type CredentialRevoked struct     { UserID, CredentialID, RevokedAt }
type RoleAssigned struct          { UserID, RoleID, Scope }
type RoleRevoked struct           { UserID, RoleID }
type DeviceBound struct           { UserID, DeviceID, TrustLevel }
type DeviceUnbound struct         { UserID, DeviceID }
```

#### Aggregate: `Device`

```go
// Device aggregate root
// Invariants:
//   - Must have a valid attestation to be TRUSTED
//   - Certificate must not be expired
//   - Can only be enrolled to one site at a time
//   - TrustLevel degrades after 30 days without attestation refresh
type Device struct {
    id              DeviceID
    deviceType      DeviceType
    manufacturer    string
    model           string
    serialNumber    string
    ministryID      uuid.UUID
    siteID          uuid.UUID
    firmwareVersion string
    certificate     DeviceCertificate  // entity
    trustLevel      TrustLevel
    trustScore      float64            // computed 0.0–1.0
    lastSeenAt      time.Time?
    enrolledAt      time.Time
    retiredAt       time.Time?
    metadata        map[string]string  // vendor-specific
    allowedUsers    []UserID           // employees authorized on this device
    version         int64
    domainEvents    []DomainEvent
}

type DeviceType enum {
    FINGERPRINT_SCANNER
    FACE_RECOGNITION_TERMINAL
    CARD_READER
    PIN_PAD
    MOBILE_DEVICE
    TABLET
    IoT_SENSOR
}

type DeviceCertificate struct {
    serialNumber:   string
    subject:        string
    issuer:         string
    notBefore:      time.Time
    notAfter:       time.Time
    publicKey:      []byte
    certificatePEM: []byte
    // Invariant: notAfter - notBefore <= 365 days (auto-rotate)
    // Invariant: must be signed by Device CA
}

// Device Domain Events
type DeviceEnrolled struct       { DeviceID, SiteID, EnrolledAt }
type DeviceActivated struct      { DeviceID, ActivatedAt }
type DeviceSuspended struct      { DeviceID, Reason, SuspendedAt }
type DeviceRevoked struct        { DeviceID, Reason, RevokedAt }
type DeviceTrustChanged struct   { DeviceID, OldLevel, NewLevel }
type DeviceFirmwareUpdated struct { DeviceID, OldVersion, NewVersion }
type DeviceHeartbeatReceived struct { DeviceID, Timestamp }
```

#### Aggregate: `Realm`

```go
// Realm aggregate root
// Invariants:
//   - Realm name must be unique across the entire platform
//   - Authentication policies are versioned for audit
//   - Cannot delete realm with active users
type Realm struct {
    id              uuid.UUID
    ministryID      uuid.UUID
    name            string              // "mohe.inwp.iq"
    displayName     string              // "Ministry of Education"
    authPolicies    AuthPolicy          // value object
    passwordPolicy  PasswordPolicy      // value object
    sessionPolicy   SessionPolicy       // value object
    roleCatalog     []RoleDefinition    // entity list
    featureFlags    map[string]bool
    isActive        bool
    version         int64
    domainEvents    []DomainEvent
}

type AuthPolicy valueObject {
    allowedMethods:      []AuthMethod     // which auth methods are enabled
    mfaRequired:         bool
    mfaEnforcementScope: MFAScope         // ALL, ADMIN_ONLY, SENSITIVE_ACTIONS
    maxFailedAttempts:   int              // default 5
    lockoutDuration:     time.Duration    // default 30 minutes
    passwordlessAllowed: bool
}

type PasswordPolicy valueObject {
    minLength:          int               // default 12
    requireUppercase:   bool
    requireLowercase:   bool
    requireDigit:       bool
    requireSpecialChar: bool
    expiryDays:         int               // default 90, 0 = never expires
    passwordHistory:    int               // number of previous passwords remembered
    maxConcurrentSessions: int            // default 5 per user
}

type SessionPolicy valueObject {
    accessTokenTTL:     time.Duration     // default 15 minutes
    refreshTokenTTL:    time.Duration     // default 24 hours
    idleTimeout:        time.Duration     // default 30 minutes
    absoluteTimeout:    time.Duration     // default 12 hours
    offlineSessionTTL:  time.Duration     // default 24 hours
}

type RoleDefinition struct {
    id:             uuid.UUID
    name:           string               // "hr_admin"
    displayName:    string
    description:    string
    permissions:    []Permission          // list of permission codes
    scope:          RoleScope
    isSystem:       bool                  // system roles cannot be deleted
    createdAt:      time.Time
}

type Permission string  // "attendance:read", "attendance:write", "leave:approve", etc.

// Realm Domain Events
type RealmCreated struct       { RealmID, MinistryID }
type RealmDeactivated struct   { RealmID, DeactivatedAt }
type AuthPolicyUpdated struct  { RealmID, OldPolicy, NewPolicy }
type RoleCreated struct        { RealmID, RoleID, Name }
type RolePermissionUpdated struct { RealmID, RoleID, Added, Removed }
```

### 3.2 Domain Services

```go
// AuthenticationService — stateless auth logic
// Responsibilities: verify credentials, issue tokens, validate MFA
type AuthenticationService interface {
    Authenticate(ctx, realmID, identifier, credential) (Session, error)
    VerifyMFA(ctx, sessionID, mfaCode) (bool, error)
    RefreshSession(ctx, refreshToken) (Session, error)
    TerminateSession(ctx, sessionID) error
}

// DeviceTrustService — computes and manages device trust
// Responsibilities: score computation, attestation verification, trust decay
type DeviceTrustService interface {
    EvaluateTrust(ctx, device) (TrustLevel, error)
    VerifyAttestation(ctx, attestationBlob, expectedNonce) (bool, error)
    ApplyTrustDecay(ctx, device) (TrustLevel, error)  // called daily by cron
    EscalateUntrusted(ctx, device) (Alert, error)
}

// FederationService — cross-ministry identity federation
// Responsibilities: SAML bridging, SCIM provisioning, cross-realm token exchange
type FederationService interface {
    ProvisionUser(ctx, sourceRealm, targetRealm, user) (UserID, error)
    ExchangeToken(ctx, sourceToken, targetRealm) (Session, error)
    SyncDirectory(ctx, realmID) (SyncResult, error)
}
```

### 3.3 Repository Interfaces

```go
type UserRepository interface {
    Save(ctx, user *User) error
    FindByID(ctx, id UserID) (*User, error)
    FindByNationalID(ctx, realmID uuid.UUID, nationalID string) (*User, error)
    FindByCredential(ctx, realmID uuid.UUID, credentialType CredentialType, identifier string) (*User, error)
    FindByDevice(ctx, deviceID DeviceID) ([]*User, error)
    FindByRole(ctx, roleID uuid.UUID) ([]*User, error)
    Search(ctx, query UserSearchQuery) (Page[*User], error)
    Delete(ctx, id UserID) error  // soft delete
}

type DeviceRepository interface {
    Save(ctx, device *Device) error
    FindByID(ctx, id DeviceID) (*Device, error)
    FindBySerialNumber(ctx, serial string) (*Device, error)
    FindBySite(ctx, siteID uuid.UUID) ([]*Device, error)
    FindByTrustLevel(ctx, level TrustLevel) ([]*Device, error)
    Search(ctx, query DeviceSearchQuery) (Page[*Device], error)
}

type RealmRepository interface {
    Save(ctx, realm *Realm) error
    FindByID(ctx, id uuid.UUID) (*Realm, error)
    FindByMinistryID(ctx, ministryID uuid.UUID) (*Realm, error)
    FindAll(ctx) ([]*Realm, error)
}
```

### 3.4 Application Services (Use Cases)

| Use Case | Command | Description |
|---|---|---|
| Register User | `RegisterUserCommand { NationalID, FullName, MinistryID, SiteID, Department, Position }` | Creates user with PENDING_VERIFICATION status |
| Verify User | `VerifyUserCommand { UserID, VerifiedBy }` | Approves user after identity verification |
| Authenticate | `AuthenticateCommand { RealmID, Identifier, Credential, DeviceAttestation? }` | Authenticates and issues session |
| Enroll Device | `EnrollDeviceCommand { SerialNumber, DeviceType, Manufacturer, Model, SiteID, Attestation }` | Registers device with PROVISIONAL trust |
| Assign Role | `AssignRoleCommand { UserID, RoleID, Scope }` | Assigns role to user within scope |
| Create Realm | `CreateRealmCommand { MinistryID, Name, DisplayName }` | Creates new ministry realm |
| Update Auth Policy | `UpdateAuthPolicyCommand { RealmID, AuthPolicy }` | Updates authentication rules |

---

## 4. attendance-service

> **Bounded Context**: Attendance Tracking
>
> **Domain**: Clock-in/out events, shift management, attendance policies, overtime calculation, biometric verification.
>
> **Language**: Go package `internal/domain/`

### 4.1 Aggregates

#### Aggregate: `ClockEvent`

```go
// ClockEvent aggregate root
// IMMUTABLE — once created, never modified
// Invariants:
//   - eventTime must not be in the future (max 5min clock skew allowed)
//   - employeeID must reference an active user
//   - deviceID must reference a TRUSTED device
//   - duplicate detection: same employee + device + eventType + eventTime ± 30s
type ClockEvent struct {
    id              uuid.UUID
    employeeID      EmployeeID
    ministryID      uuid.UUID
    siteID          uuid.UUID
    deviceID        DeviceID
    eventType       ClockEventType
    eventTime       OfflineTimestamp
    recordedAt      time.Time           // server receive time
    biometricMatch  float64?            // 0.0–1.0 confidence
    latitude        float64?            // GPS latitude if mobile
    longitude       float64?            // GPS longitude if mobile
    ipAddress       string?
    syncMetadata    SyncMetadata
    // NO version — immutable aggregate
    domainEvents    []DomainEvent
}

type ClockEventType enum {
    CLOCK_IN
    CLOCK_OUT
    BREAK_START
    BREAK_END
    MANUAL_CORRECTION
}

// ClockEvent Domain Events
type ClockInCreated struct    { EventID, EmployeeID, DeviceID, EventTime }
type ClockOutCreated struct   { EventID, EmployeeID, DeviceID, EventTime }
type BreakStarted struct      { EventID, EmployeeID, EventTime }
type BreakEnded struct        { EventID, EmployeeID, EventTime }
type AttendanceCorrected struct { EventID, EmployeeID, CorrectedBy, OldEvent, NewEvent }
type AttendanceDisputed struct { EventID, EmployeeID, Reason, DisputedAt }
```

#### Aggregate: `Shift`

```go
// Shift aggregate root
// Defines a scheduled work period for a site or employee group
// Invariants:
//   - startTime must be before endTime
//   - shifts cannot overlap for the same employee/group
//   - max shift duration: 16 hours (configurable per ministry)
type Shift struct {
    id              uuid.UUID
    ministryID      uuid.UUID
    siteID          uuid.UUID
    name            string              // "Morning Shift", "Night Shift"
    startTime       time.Time           // e.g., 08:00 (recurring daily)
    endTime         time.Time           // e.g., 16:00
    gracePeriod     time.Duration       // late tolerance (default 15min)
    breakDuration   time.Duration       // mandated break time
    applicableDays  []time.Weekday      // Sunday–Thursday in Iraq
    applicableGroups []GroupID?         // which employee groups this applies to
    overtimePolicy  OvertimePolicy      // value object
    isActive        bool
    version         int64
    domainEvents    []DomainEvent
}

type OvertimePolicy valueObject {
    enabled:            bool
    rateMultiplier:     float64           // 1.5x standard
    thresholdHours:     int               // after 8 hours
    maxOvertimePerDay:  time.Duration     // max 4 hours
    requiresApproval:   bool
    doubleHolidayRate:  bool
}

// Shift Domain Events
type ShiftCreated struct     { ShiftID, SiteID, Name }
type ShiftModified struct    { ShiftID, ModifiedBy }
type ShiftDeactivated struct { ShiftID, DeactivatedAt }
```

#### Aggregate: `AttendancePolicy`

```go
// AttendancePolicy aggregate root
// Site or ministry-level rules governing attendance behaviour
// Invariants:
//   - Each site can have only ONE active policy
//   - Policy changes are versioned and require audit trail
type AttendancePolicy struct {
    id              uuid.UUID
    ministryID      uuid.UUID
    siteID          uuid.UUID?
    name            string
    rules           AttendanceRuleSet   // value object
    effectiveFrom   time.Time
    effectiveTo     time.Time?          // null = active indefinitely
    version         int
    supersedes      uuid.UUID?          // previous policy version
    approvedBy      uuid.UUID           // admin who approved
    createdAt       time.Time
    domainEvents    []DomainEvent
}

type AttendanceRuleSet valueObject {
    clockInWindow:         TimeRange          // e.g., 07:30–09:00
    lateThreshold:         time.Duration      // > 15min after shift start
    earlyDepartureThreshold: time.Duration    // > 15min before shift end
    autoApprovalWindow:    time.Duration      // auto-approve if discrepancy < 5min
    requireBiometric:      bool
    requireGPS:            bool               // for mobile clock-in
    maxDailyClocks:        int                // prevent abuse
    minHoursBetweenClocks: time.Duration      // minimum gap between in/out
    allowManualCorrection: bool
    correctionRequiresApproval: bool
    disputeWindow:         time.Duration      // 48 hours to dispute
}

// AttendancePolicy Domain Events
type PolicyCreated struct   { PolicyID, SiteID, EffectiveFrom }
type PolicyActivated struct { PolicyID, ActivatedAt }
type PolicySuperseded struct { PolicyID, SupersededBy, SupersededAt }
```

#### Aggregate: `AttendanceException`

```go
// AttendanceException aggregate root
// Represents irregularities in attendance (late, early departure, missing clock)
// Invariants:
//   - Created automatically by policy evaluation OR manually by admin
//   - Can be resolved with justification
//   - Escalates if unresolved beyond threshold
type AttendanceException struct {
    id              uuid.UUID
    employeeID      EmployeeID
    clockEventID    uuid.UUID?
    exceptionType   ExceptionType
    severity        ExceptionSeverity
    description     string
    occurredAt      time.Time
    detectedAt      time.Time
    justification   Justification?
    resolvedAt      time.Time?
    resolvedBy      uuid.UUID?
    escalatedAt     time.Time?
    domainEvents    []DomainEvent
}

type ExceptionType enum {
    LATE_CLOCK_IN
    EARLY_CLOCK_OUT
    MISSING_CLOCK_IN
    MISSING_CLOCK_OUT
    MISSING_BREAK
    EXCEEDED_MAX_HOURS
    DUPLICATE_CLOCK
    BIOMETRIC_MISMATCH
    GPS_OUT_OF_RANGE
}

type ExceptionSeverity enum {
    LOW     // informational, < 15min deviation
    MEDIUM  // requires justification
    HIGH    // requires admin review
    CRITICAL// potential fraud, immediate escalation
}

type Justification struct {
    reason:     string
    type:       JustificationType   // SICKNESS, OFFICIAL_DUTY, TECHNICAL_ISSUE, OTHER
    supportingDocs: []DocumentRef?
    submittedAt: time.Time
    approvedBy: uuid.UUID?
    approvedAt: time.Time?
}

// AttendanceException Domain Events
type ExceptionCreated struct    { ExceptionID, EmployeeID, ExceptionType }
type ExceptionJustified struct  { ExceptionID, JustificationType }
type ExceptionResolved struct   { ExceptionID, ResolvedBy }
type ExceptionEscalated struct  { ExceptionID, EscalatedTo }
```

### 4.2 Domain Services

```go
// AttendanceCalculationService
// Responsibilities: compute worked hours, overtime, break compliance
type AttendanceCalculationService interface {
    CalculateWorkedHours(ctx, employeeID, date) (WorkedHours, error)
    CalculateOvertime(ctx, employeeID, dateRange) (OvertimeSummary, error)
    DetectMissingClocks(ctx, siteID, date) ([]AttendanceException, error)
    VerifyBreakCompliance(ctx, employeeID, date) (BreakCompliance, error)
}

// DuplicateDetectionService
// Responsibilities: prevent duplicate clock events within time window
type DuplicateDetectionService interface {
    IsDuplicate(ctx, employeeID, deviceID, eventType, eventTime) (bool, error)
    FindNearDuplicates(ctx, employeeID, timeWindow) ([]ClockEvent, error)
}

// BiometricVerificationService
// Responsibilities: delegate biometric match to device, verify confidence
type BiometricVerificationService interface {
    Verify(ctx, employeeID, deviceID, biometricData) (BiometricResult, error)
    EnrollTemplate(ctx, employeeID, deviceID, template) error
}

type BiometricResult struct {
    Matched:     bool
    Confidence:  float64
    TemplateHash: []byte   // hash of matched template, NOT the template
}
```

### 4.3 Repository Interfaces

```go
type ClockEventRepository interface {
    Save(ctx, event *ClockEvent) error
    SaveBatch(ctx, events []*ClockEvent) error  // bulk insert for sync
    FindByID(ctx, id uuid.UUID) (*ClockEvent, error)
    FindByEmployee(ctx, employeeID EmployeeID, dateRange TimeRange) ([]*ClockEvent, error)
    FindByDevice(ctx, deviceID DeviceID, dateRange TimeRange) ([]*ClockEvent, error)
    FindBySite(ctx, siteID uuid.UUID, dateRange TimeRange) ([]*ClockEvent, error)
    FindUnsynced(ctx, nodeID uuid.UUID) ([]*ClockEvent, error)
    ExistsDuplicate(ctx, employeeID, deviceID, eventType, eventTime) (bool, error)
}

type ShiftRepository interface {
    Save(ctx, shift *Shift) error
    FindByID(ctx, id uuid.UUID) (*Shift, error)
    FindBySite(ctx, siteID uuid.UUID) ([]*Shift, error)
    FindActiveBySite(ctx, siteID uuid.UUID) (*Shift, error)
    Delete(ctx, id uuid.UUID) error
}

type AttendancePolicyRepository interface {
    Save(ctx, policy *AttendancePolicy) error
    FindByID(ctx, id uuid.UUID) (*AttendancePolicy, error)
    FindActiveBySite(ctx, siteID uuid.UUID) (*AttendancePolicy, error)
    FindActiveByMinistry(ctx, ministryID uuid.UUID) (*AttendancePolicy, error)
    FindHistoryBySite(ctx, siteID uuid.UUID) ([]*AttendancePolicy, error)
}

type AttendanceExceptionRepository interface {
    Save(ctx, exception *AttendanceException) error
    FindByID(ctx, id uuid.UUID) (*AttendanceException, error)
    FindByEmployee(ctx, employeeID EmployeeID, dateRange TimeRange) ([]*AttendanceException, error)
    FindUnresolved(ctx, siteID uuid.UUID) ([]*AttendanceException, error)
    FindEscalated(ctx, ministryID uuid.UUID) ([]*AttendanceException, error)
}
```

### 4.4 Application Services (Use Cases)

| Use Case | Command | Description |
|---|---|---|
| Clock In | `ClockInCommand { EmployeeID, DeviceID, SiteID, EventTime, Biometric?, GPS? }` | Records clock-in with duplicate detection |
| Clock Out | `ClockOutCommand { EmployeeID, DeviceID, SiteID, EventTime, Biometric?, GPS? }` | Records clock-out, triggers worked hours calc |
| Correct Attendance | `CorrectAttendanceCommand { EventID, CorrectedBy, NewEventTime, Reason }` | Admin correction with audit trail |
| Dispute Event | `DisputeEventCommand { EventID, EmployeeID, Reason }` | Employee disputes a recorded event |
| Create Shift | `CreateShiftCommand { SiteID, Name, StartTime, EndTime, GracePeriod, BreakDuration }` | Creates new shift schedule |
| Set Policy | `SetAttendancePolicyCommand { SiteID, Rules, EffectiveFrom }` | Activates new attendance rules |
| Justify Exception | `JustifyExceptionCommand { ExceptionID, Reason, Type, SupportingDocs }` | Employee provides justification |
| Resolve Exception | `ResolveExceptionCommand { ExceptionID, ResolvedBy, Resolution }` | Admin resolves an exception |
| Process Day | `ProcessAttendanceDayCommand { SiteID, Date }` | Batch process: calc hours, detect exceptions |

---

## 5. leave-service

> **Bounded Context**: Leave Management
>
> **Domain**: Leave requests, approvals, balance tracking, accrual policies, ministry-specific leave types.
>
> **Language**: Go package `internal/domain/`

### 5.1 Aggregates

#### Aggregate: `LeaveRequest`

```go
// LeaveRequest aggregate root
// Invariants:
//   - startDate must be before or equal to endDate
//   - cannot overlap with another approved leave for same employee
//   - duration must not exceed remaining balance for the leave type
//   - approval chain must be complete before status becomes APPROVED
//   - cancellation only allowed if status is PENDING or APPROVED (not IN_PROGRESS or COMPLETED)
type LeaveRequest struct {
    id              uuid.UUID
    employeeID      EmployeeID
    ministryID      uuid.UUID
    siteID          uuid.UUID
    leaveType       LeaveType
    startDate       time.Time           // date only
    endDate         time.Time           // date only
    durationDays    int                 // computed: business days
    reason          string
    supportingDocs  []DocumentRef?
    status          LeaveStatus
    approvalChain   []ApprovalStep      // value object list
    cancellation     Cancellation?
    syncMetadata    SyncMetadata
    version         int64
    domainEvents    []DomainEvent
}

type LeaveType enum {
    ANNUAL
    SICK
    MATERNITY
    PATERNITY
    HAJJ             // pilgrimage leave
    UMRAH
    EXAMINATION      // for employees pursuing education
    COMPASSIONATE    // bereavement
    UNPAID
    SPECIAL          // ministerial discretion
    COMPENSATORY     // overtime compensation
}

type LeaveStatus enum {
    DRAFT
    PENDING_APPROVAL
    APPROVED
    REJECTED
    CANCELLED
    IN_PROGRESS       // employee is currently on this leave
    COMPLETED         // leave period has passed
}

type ApprovalStep valueObject {
    stepOrder:      int
    approverID:     uuid.UUID
    role:           string              // "supervisor", "hr_manager", "ministry_admin"
    status:         ApprovalStatus      // PENDING, APPROVED, REJECTED, SKIPPED
    decidedAt:      time.Time?
    decision:       string?
    // Invariant: approval chain is sequential (step N must be resolved before step N+1)
    // Invariant: any REJECTED step terminates the chain
}

type Cancellation valueObject {
    cancelledAt:    time.Time
    cancelledBy:    uuid.UUID
    reason:         string
}

// LeaveRequest Domain Events
type LeaveRequestCreated struct       { RequestID, EmployeeID, LeaveType, StartDate, EndDate }
type LeaveRequestSubmitted struct     { RequestID, SubmittedAt }
type LeaveRequestApproved struct      { RequestID, ApprovedBy, StepOrder }
type LeaveRequestRejected struct      { RequestID, RejectedBy, Reason }
type LeaveRequestCancelled struct     { RequestID, CancelledBy, Reason }
type LeaveRequestExpired struct       { RequestID }  // auto-cancelled after 60 days pending
type LeaveRequestCompleted struct     { RequestID, CompletedAt }
type LeaveApprovalEscalated struct    { RequestID, EscalatedTo, DaysPending }
```

#### Aggregate: `LeaveBalance`

```go
// LeaveBalance aggregate root
// CRITICAL — financial-grade accuracy required
// Invariants:
//   - balance can never go negative (requires special override)
//   - accrual and deduction records form an auditable ledger
//   - balance is computed as: SUM(accruals) - SUM(deductions) + SUM(adjustments)
//   - concurrent modifications must use pessimistic locking (SELECT FOR UPDATE)
type LeaveBalance struct {
    id              uuid.UUID
    employeeID      EmployeeID
    ministryID      uuid.UUID
    leaveType       LeaveType
    currentBalance  float64            // days
    accruedToDate   float64            // total accrued
    takenToDate     float64            // total taken
    pendingDeduction float64           // approved but not yet taken
    fiscalYearStart  time.Time
    fiscalYearEnd    time.Time
    lastAccrualAt    time.Time?        // last time accrual was processed
    version          int64
    domainEvents     []DomainEvent

    // Transactions (entity list inside aggregate)
    transactions    []BalanceTransaction
}

type BalanceTransaction struct {
    id:              uuid.UUID
    type:            TransactionType   // ACCRUAL, DEDUCTION, ADJUSTMENT, EXPIRY
    amount:          float64
    referenceID:     uuid.UUID?        // LeaveRequestID, AccrualRuleID
    description:     string
    createdAt:       time.Time
    createdBy:       uuid.UUID         // system or admin
}

type TransactionType enum {
    ACCRUAL          // monthly/yearly accumulation
    DEDUCTION        // leave taken
    ADJUSTMENT       // manual correction by admin
    EXPIRY           // balance expired at fiscal year end
    FORFEITURE       // left without using
    TRANSFER_IN      // transferred from another ministry
    TRANSFER_OUT     // transferred to another ministry
}

// LeaveBalance Domain Events
type BalanceAccrued struct          { BalanceID, EmployeeID, LeaveType, Amount, NewBalance }
type BalanceDeducted struct         { BalanceID, EmployeeID, LeaveType, Amount, NewBalance }
type BalanceAdjusted struct         { BalanceID, AdjustedBy, OldBalance, NewBalance, Reason }
type BalanceExpired struct          { BalanceID, LeaveType, Amount }
type BalanceWarning struct          { BalanceID, EmployeeID, LeaveType, RemainingDays }
```

#### Aggregate: `AccrualPolicy`

```go
// AccrualPolicy aggregate root
// Defines how leave balance accumulates over time
// Invariants:
//   - Each leave type can have only ONE active accrual rule per ministry
//   - Rate must be positive
//   - Max accrual cap prevents infinite accumulation
type AccrualPolicy struct {
    id              uuid.UUID
    ministryID      uuid.UUID
    leaveType       LeaveType
    name            string
    accrualRules    AccrualRuleSet      // value object
    eligibilityRules EligibilityRuleSet  // value object
    effectiveFrom   time.Time
    effectiveTo     time.Time?
    version         int
    domainEvents    []DomainEvent
}

type AccrualRuleSet valueObject {
    frequency:      AccrualFrequency   // MONTHLY, YEARLY, PER_SERVICE_DAY
    rate:           float64            // e.g., 2.5 days per month
    maxAccrual:     float64            // e.g., 30 days max
    proRataFirstYear: bool             // partial accrual for new employees
    carryOverLimit: float64            // max days that carry to next year
    carryOverExpiry: int              // months into next year before expiry
    serviceYearThreshold: int         // years of service for rate increase
    serviceYearBonus: float64         // additional days after threshold
}

type EligibilityRuleSet valueObject {
    probationDays:      int            // 90 days before eligible
    minServiceMonths:   int            // minimum tenure
    excludesContractors: bool
    requiresActiveStatus: bool
    ministrySpecific:   map[string]bool // ministry-level overrides
}

type AccrualFrequency enum {
    MONTHLY       // e.g., 2.5 days per month
    YEARLY        // e.g., 30 days per year, granted on anniversary
    PER_SERVICE_DAY  // e.g., 1 day per 20 days worked (for daily wage workers)
}

// AccrualPolicy Domain Events
type AccrualPolicyCreated struct   { PolicyID, MinistryID, LeaveType }
type AccrualPolicyUpdated struct   { PolicyID, UpdatedBy }
type AccrualPolicyExpired struct   { PolicyID }
type AccrualProcessed struct      { PolicyID, FiscalPeriod, EmployeesCount, TotalAccrued }
```

### 5.2 Domain Services

```go
// LeaveCalculationService
// Responsibilities: compute balance, validate leave duration, check overlaps
type LeaveCalculationService interface {
    CalculateBalance(ctx, employeeID, leaveType) (LeaveBalance, error)
    ValidateLeaveDuration(ctx, employeeID, leaveType, startDate, endDate) (ValidationResult, error)
    CheckOverlap(ctx, employeeID, startDate, endDate) ([]LeaveRequest, error)
    ComputeBusinessDays(ctx, startDate, endDate) int
    ApplyHolidayCalendar(ctx, startDate, endDate, ministryID) (int, error) // exclude public holidays
}

// AccrualProcessingService
// Responsibilities: run accrual cycles, manage fiscal year transitions
type AccrualProcessingService interface {
    ProcessMonthlyAccrual(ctx, ministryID, fiscalPeriod) (AccrualResult, error)
    ProcessYearlyAccrual(ctx, ministryID, fiscalYear) (AccrualResult, error)
    ApplyCarryOver(ctx, employeeID, leaveType, fiscalYearEnd) error
    ExpireBalances(ctx, ministryID, fiscalYear) (ExpiryResult, error)
    HandleTransfer(ctx, employeeID, fromMinistry, toMinistry, leaveType, balance) error
}

// ApprovalWorkflowService
// Responsibilities: route approvals, escalate timeouts, enforce chain
type ApprovalWorkflowService interface {
    DetermineApprovalChain(ctx, employeeID, leaveType, durationDays) ([]ApprovalStep, error)
    ProcessApproval(ctx, requestID, approverID, decision, reason) error
    EscalateStaleRequests(ctx, thresholdDays) ([]LeaveRequest, error)
    AutoApproveEligible(ctx, employeeID, durationDays) (bool, error)
    NotifyApprover(ctx, requestID, approverID) error
}
```

### 5.3 Repository Interfaces

```go
type LeaveRequestRepository interface {
    Save(ctx, request *LeaveRequest) error
    FindByID(ctx, id uuid.UUID) (*LeaveRequest, error)
    FindByEmployee(ctx, employeeID EmployeeID, dateRange TimeRange) ([]*LeaveRequest, error)
    FindByApprover(ctx, approverID uuid.UUID, status LeaveStatus) ([]*LeaveRequest, error)
    FindByStatus(ctx, status LeaveStatus, ministryID uuid.UUID) ([]*LeaveRequest, error)
    FindOverlapping(ctx, employeeID, startDate, endDate) ([]*LeaveRequest, error)
    FindPendingApproval(ctx, approverID uuid.UUID) ([]*LeaveRequest, error)
    Search(ctx, query LeaveSearchQuery) (Page[*LeaveRequest], error)
}

type LeaveBalanceRepository interface {
    Save(ctx, balance *LeaveBalance) error
    FindByEmployeeAndType(ctx, employeeID EmployeeID, leaveType LeaveType) (*LeaveBalance, error)
    FindByEmployee(ctx, employeeID EmployeeID) ([]*LeaveBalance, error)
    FindByMinistry(ctx, ministryID uuid.UUID, leaveType LeaveType) ([]*LeaveBalance, error)
    LockForUpdate(ctx, employeeID EmployeeID, leaveType LeaveType) (*LeaveBalance, error) // pessimistic
}

type AccrualPolicyRepository interface {
    Save(ctx, policy *AccrualPolicy) error
    FindActiveByMinistryAndType(ctx, ministryID uuid.UUID, leaveType LeaveType) (*AccrualPolicy, error)
    FindByMinistry(ctx, ministryID uuid.UUID) ([]*AccrualPolicy, error)
    FindHistory(ctx, policyID uuid.UUID) ([]*AccrualPolicy, error)
}
```

### 5.4 Application Services (Use Cases)

| Use Case | Command | Description |
|---|---|---|
| Request Leave | `RequestLeaveCommand { EmployeeID, LeaveType, StartDate, EndDate, Reason, Documents }` | Creates leave request with pending status |
| Approve Leave | `ApproveLeaveCommand { RequestID, ApproverID, StepOrder, Decision }` | Approves/rejects at current approval step |
| Cancel Leave | `CancelLeaveCommand { RequestID, CancelledBy, Reason }` | Cancels an approved or pending request |
| Adjust Balance | `AdjustBalanceCommand { EmployeeID, LeaveType, Amount, Reason, AdjustedBy }` | Manual balance adjustment with audit |
| Process Accrual | `ProcessAccrualCommand { MinistryID, LeaveType, FiscalPeriod }` | Batch accrual processing |
| Get Balance | `GetBalanceQuery { EmployeeID, LeaveType }` | Returns current leave balance |
| Get Approvals | `GetPendingApprovalsQuery { ApproverID }` | Returns list of requests awaiting approval |

---

## 6. audit-ledger

> **Bounded Context**: Immutable Audit
>
> **Domain**: Append-only event ingestion, hash chain verification, tamper-evident sealing, compliance queries.
>
> **Language**: Go package `internal/domain/`

### 6.1 Aggregates

#### Aggregate: `LedgerEntry`

```go
// LedgerEntry aggregate root
// IMMUTABLE — append-only. Never updated or deleted.
// Invariants:
//   - prevHash links to the hash of the previous entry (or zero hash for genesis)
//   - payloadHash is SHA-256 of the serialized event payload
//   - entryHash = SHA-256(prevHash + payloadHash + timestamp + nonce)
//   - signature is Ed25519 of entryHash by the producing service
//   - entryHash is unique (no duplicate entries)
type LedgerEntry struct {
    id              uuid.UUID          // entry sequence ID (monotonic)
    prevHash        [32]byte           // SHA-256 of previous entry
    payloadHash     [32]byte           // SHA-256 of event payload
    entryHash       [32]byte           // SHA-256(prevHash || payloadHash || timestamp || nonce)
    nonce           [8]byte            // ensures uniqueness even for identical payloads
    timestamp       time.Time          // when the ledger entry was created
    sourceService   string             // "attendance-service"
    sourceNodeID    uuid.UUID          // which node produced this
    ministryID      uuid.UUID
    eventType       string             // "inwp.attendance.v1.clock-in.created"
    eventID         uuid.UUID          // reference to original event
    payload         []byte             // the full event payload (signed)
    signature       [64]byte           // Ed25519 signature of entryHash
    signingKeyID    string             // which key signed it
    // NO version — immutable
    domainEvents    []DomainEvent
}

// LedgerEntry Domain Events
type EntryAppended struct  { EntryID, EntryHash, EventType, Timestamp }
type IntegrityVerified struct { StartEntryID, EndEntryID, Valid bool }
type IntegrityFailure struct  { EntryID, ExpectedHash, ActualHash, DetectedAt }
```

#### Aggregate: `Seal`

```go
// Seal aggregate root
// Periodic cryptographic seal over a range of ledger entries
// Used for efficient integrity verification without checking every entry
// Invariants:
//   - Seals are created on a schedule (every 1000 entries or every hour, whichever comes first)
//   - Seal hash = SHA-256(merkle_root_of_range + previous_seal_hash + seal_time)
//   - Seals are published to a public bulletin board for external verification
type Seal struct {
    id              uuid.UUID
    startEntryID    uuid.UUID          // first entry in this seal range
    endEntryID      uuid.UUID          // last entry in this seal range
    entryCount      int
    merkleRoot      [32]byte           // Merkle root of all entries in range
    previousSealHash [32]byte          // hash of previous seal (zero for first)
    sealHash        [32]byte           // SHA-256(merkleRoot || previousSealHash || sealedAt)
    sealedAt        time.Time
    signature       [64]byte           // Ed25519 by audit-ledger service
    verifiedAt      time.Time?
    isVerified      bool
    domainEvents    []DomainEvent
}

// Seal Domain Events
type SealGenerated struct { SealID, StartEntryID, EndEntryID, MerkleRoot, SealedAt }
type SealVerified struct   { SealID, VerifiedAt, Valid bool }
type SealPublished struct  { SealID, BulletinBoardURL }
```

### 6.2 Domain Services

```go
// IntegrityVerificationService
// Responsibilities: verify hash chain integrity, validate seals, detect tampering
type IntegrityVerificationService interface {
    VerifyChain(ctx, fromEntryID, toEntryID) (ChainVerificationResult, error)
    VerifySeal(ctx, sealID) (bool, error)
    VerifyAllSeals(ctx) ([]SealVerificationResult, error)
    DetectAnomalies(ctx, fromTime, toTime) ([]Anomaly, error)
    RebuildChain(ctx, fromEntryID) error  // recovery from backup
}

type ChainVerificationResult struct {
    StartEntryID:  uuid.UUID
    EndEntryID:    uuid.UUID
    TotalEntries:  int
    Verified:      bool
    FailedEntries: []ChainBreak
    VerifiedAt:    time.Time
}

type ChainBreak struct {
    EntryID:      uuid.UUID
    ExpectedHash: [32]byte
    ActualHash:   [32]byte
    Position:     int
}

// RetentionService
// Responsibilities: manage data lifecycle, archive old entries, enforce retention policies
type RetentionService interface {
    ApplyRetentionPolicy(ctx, ministryID) (ArchiveResult, error)
    MarkForArchive(ctx, olderThan time.Time) ([]LedgerEntry, error)
    RestoreArchive(ctx, entryID) (*LedgerEntry, error)
    PurgeArchived(ctx, olderThan time.Time) error
}
```

### 6.3 Repository Interfaces

```go
type LedgerEntryRepository interface {
    Append(ctx, entry *LedgerEntry) error  // insert-only
    FindByID(ctx, id uuid.UUID) (*LedgerEntry, error)
    FindByEventID(ctx, eventID uuid.UUID) (*LedgerEntry, error)
    FindByRange(ctx, fromID, toID uuid.UUID) ([]*LedgerEntry, error)
    FindByMinistryAndTime(ctx, ministryID uuid.UUID, timeRange TimeRange) ([]*LedgerEntry, error)
    FindByEventType(ctx, eventType string, timeRange TimeRange) ([]*LedgerEntry, error)
    GetLatestEntry(ctx) (*LedgerEntry, error)
    GetEntryCount(ctx) (int64, error)
    GetEntryAtPosition(ctx, position int64) (*LedgerEntry, error)  // by sequence
}

type SealRepository interface {
    Save(ctx, seal *Seal) error
    FindByID(ctx, id uuid.UUID) (*Seal, error)
    GetLatestSeal(ctx) (*Seal, error)
    FindByRange(ctx, fromTime, toTime time.Time) ([]*Seal, error)
    FindUnverified(ctx) ([]*Seal, error)
}
```

### 6.4 Application Services (Use Cases)

| Use Case | Command | Description |
|---|---|---|
| Ingest Event | `IngestEventCommand { EventPayload, SourceService, SourceNode }` | Appends event to hash chain |
| Generate Seal | `GenerateSealCommand {}` | Creates cryptographic seal over recent entries |
| Verify Integrity | `VerifyIntegrityCommand { FromEntryID, ToEntryID }` | Full chain integrity check |
| Query Ledger | `QueryLedgerCommand { MinistryID, EventType, TimeRange, Cursor, Limit }` | Compliance queries |
| Publish Seal | `PublishSealCommand { SealID, BulletinBoardURL }` | Publishes seal for external verification |

---

## 7. sync-engine

> **Bounded Context**: Data Synchronization
>
> **Domain**: Merkle tree reconciliation, delta computation, conflict resolution, bandwidth management, peer discovery.
>
> **Language**: Rust crate `sync-engine/`

### 7.1 Aggregates

#### Aggregate: `MerkleTree`

```go
// MerkleTree aggregate root
// Represents a single partition's merkle tree (ministry + entity type + time bucket)
// Invariants:
//   - Tree is a complete binary merkle tree (leaf count padded to power of 2)
//   - Leaf hashes = SHA-256(recordID + recordVersion + lastModified)
//   - Node hashes = SHA-256(leftChild + rightChild)
//   - Root hash changes when ANY leaf changes
type MerkleTree struct {
    id              uuid.UUID
    partitionKey    PartitionKey         // value object
    nodeID          uuid.UUID            // which node owns this tree
    rootHash        [32]byte
    depth           int
    leafCount       int
    lastModifiedAt  time.Time
    lastVerifiedAt  time.Time?
    version         int64
    domainEvents    []DomainEvent
}

type PartitionKey valueObject {
    ministryID:     uuid.UUID
    entityType:     string               // "attendance.clock_events", "leave.requests", etc.
    timeBucket:     string               // "2026-05" (monthly granularity)
    // Invariant: timeBucket format is "YYYY-MM"
}

// MerkleTree Domain Events
type TreeRebuilt struct      { TreeID, PartitionKey, RootHash, LeafCount }
type LeafUpdated struct      { TreeID, RecordID, OldHash, NewHash }
type BranchCompared struct   { TreeID, RemoteNodeID, DifferingBranches []string }
type TreeVerified struct     { TreeID, RemoteNodeID, Match bool }
```

#### Aggregate: `SyncBatch`

```go
// SyncBatch aggregate root
// A signed, atomic set of events exchanged between two nodes
// Invariants:
//   - Batch is immutable after both nodes sign
//   - Conflict count must match actual number of conflicts detected
//   - Source and target signatures must be valid Ed25519
//   - Events must belong to a single partition
type SyncBatch struct {
    id              uuid.UUID            // sync_id
    sourceNodeID    uuid.UUID            // sender
    targetNodeID    uuid.UUID            // receiver
    partitionKey    PartitionKey
    direction       SyncDirection        // UPLOAD, DOWNLOAD, BIDIRECTIONAL
    events          []SyncedEvent        // value object list
    conflicts       []ConflictRecord     // entity list
    localMerkle     [32]byte             // merkle root AFTER applying changes
    remoteMerkle    [32]byte             // remote merkle root AFTER applying changes
    sourceSignature [64]byte
    targetSignature [64]byte
    byteCount       int64
    eventCount      int
    compressed      bool
    createdAt       time.Time
    committedAt     time.Time?
    domainEvents    []DomainEvent
}

type SyncDirection enum {
    UPLOAD          // edge → hub
    DOWNLOAD        // hub → edge
    BIDIRECTIONAL   // P2P LAN sync
}

type SyncedEvent valueObject {
    eventID:        uuid.UUID
    recordID:       uuid.UUID
    recordType:     string
    action:         SyncAction   // CREATE, UPDATE, DELETE
    payload:        []byte
    localTimestamp: time.Time
    version:        int64
}

type SyncAction enum {
    CREATE
    UPDATE
    DELETE
}

type ConflictRecord struct {
    id:              uuid.UUID
    recordID:        uuid.UUID
    recordType:      string
    localVersion:    int64
    remoteVersion:   int64
    localPayload:    []byte
    remotePayload:   []byte
    resolution:      ConflictResolution?
    resolvedAt:      time.Time?
    resolvedBy:      string?        // "auto-lww", "auto-ministry-author", "manual-{adminID}"
}

type ConflictResolution enum {
    LOCAL_WINS
    REMOTE_WINS
    MERGED
    MANUAL_REQUIRED
}

// SyncBatch Domain Events
type BatchCreated struct      { BatchID, SourceNodeID, TargetNodeID, PartitionKey, EventCount }
type BatchCommitted struct    { BatchID, CommittedAt, ConflictsResolved, ConflictsManual }
type ConflictDetected struct  { BatchID, RecordID, RecordType, LocalVersion, RemoteVersion }
type ConflictResolved struct  { BatchID, RecordID, Resolution, ResolvedBy }
```

#### Aggregate: `SyncNode`

```go
// SyncNode aggregate root
// Represents a peer node in the sync network (edge, relay, or hub)
// Invariants:
//   - Heartbeat must be received at least every 120s or node marked DEGRADED
//   - Node certificate must not be expired
//   - Capabilities determine what services this node runs
type SyncNode struct {
    id              uuid.UUID
    nodeType        NodeType             // EDGE, REGIONAL_RELAY, NATIONAL_HUB
    ministryID      uuid.UUID
    siteID          uuid.UUID?
    hostname        string
    ipAddresses     []string
    port            int
    certificate     DeviceCertificate    // entity
    capabilities    []string             // ["attendance", "leave", "sync"]
    status          NodeStatus
    lastHeartbeatAt time.Time?
    heartbeatInterval time.Duration
    bandwidthBudget  BandwidthBudget?    // value object (for edge nodes)
    version         int64
    domainEvents    []DomainEvent
}

type NodeType enum {
    EDGE
    REGIONAL_RELAY
    NATIONAL_HUB
}

type NodeStatus enum {
    ONLINE
    DEGRADED          // heartbeat delayed, high latency
    OFFLINE           // no heartbeat > 120s
    DECOMMISSIONED
}

type BandwidthBudget valueObject {
    dailyUploadLimit:   int64            // bytes per day
    dailyDownloadLimit: int64
    priorityLevels:     map[string]int   // entity types and their priority
    // Invariant: cumulative usage must not exceed limit within 24h sliding window
}

// SyncNode Domain Events
type NodeRegistered struct     { NodeID, NodeType, SiteID }
type NodeOnline struct         { NodeID, OnlineAt, IPAddress }
type NodeOffline struct        { NodeID, OfflineAt, Duration }
type NodeDegraded struct       { NodeID, Reason }
type NodeBandwidthExceeded struct { NodeID, EntityType, Usage, Limit }
```

### 7.2 Domain Services

```go
// ReconciliationService
// Responsibilities: merkle tree diff, delta computation, conflict detection
type ReconciliationService interface {
    ComputeDiff(ctx, localTree, remoteRoot) (DiffResult, error)
    RequestMissingLeaves(ctx, remoteNodeID, branches) ([]LeafHash, error)
    BuildDelta(ctx, missingLeaves) (Delta, error)
    DetectConflicts(ctx, localEvents, remoteEvents) ([]ConflictRecord, error)
}

type DiffResult struct {
    PartitionKey:     PartitionKey
    RemoteRoot:       [32]byte
    LocalRoot:        [32]byte
    DifferingBranches: []string   // branch paths that differ
    Match:            bool
}

// ConflictResolutionService
// Responsibilities: apply conflict resolution strategies, escalate manual cases
type ConflictResolutionService interface {
    ResolveAuto(ctx, conflict, strategy ResolutionStrategy) (ConflictResolution, error)
    EscalateManual(ctx, conflict) error
    ApplyResolution(ctx, batchID, conflict) error
    BatchResolve(ctx, conflicts, strategy ResolutionStrategy) ([]ConflictRecord, error)
}

type ResolutionStrategy enum {
    LWW_TIMESTAMP         // last-writer-wins by local_timestamp
    MINISTRY_AUTHOR_WINS  // ministry HR records override terminal
    SERVICE_MERGE         // service-specific merge (e.g., accrual math)
    MANUAL                // admin must decide
}

// CompressionService
// Responsibilities: delta compression, decompression, bandwidth tracking
type CompressionService interface {
    CompressDelta(ctx, events []SyncedEvent) (CompressedPayload, error)
    DecompressDelta(ctx, payload CompressedPayload) ([]SyncedEvent, error)
    EstimateBandwidth(ctx, nodeID, partitionKey) (BandwidthEstimate, error)
    TrackUsage(ctx, nodeID, bytes, direction SyncDirection) error
}

// DiscoveryService
// Responsibilities: LAN peer discovery via mDNS, regional relay assignment
type DiscoveryService interface {
    DiscoverPeersLAN(ctx) ([]SyncNode, error)
    GetRegionalRelay(ctx, nodeID) (*SyncNode, error)
    AdvertisePresence(ctx) error
    HealthCheckPeers(ctx, peerIDs) (map[uuid.UUID]NodeStatus, error)
}
```

### 7.3 Repository Interfaces

```go
type MerkleTreeRepository interface {
    Save(ctx, tree *MerkleTree) error
    FindByPartition(ctx, nodeID uuid.UUID, partitionKey PartitionKey) (*MerkleTree, error)
    FindAllByNode(ctx, nodeID uuid.UUID) ([]*MerkleTree, error)
    UpdateLeaf(ctx, treeID uuid.UUID, recordID uuid.UUID, hash [32]byte) error
    GetLeaves(ctx, treeID uuid.UUID) ([]LeafRecord, error)
    Delete(ctx, treeID uuid.UUID) error
}

type SyncBatchRepository interface {
    Save(ctx, batch *SyncBatch) error
    FindByID(ctx, id uuid.UUID) (*SyncBatch, error)
    FindByPartition(ctx, partitionKey PartitionKey, since time.Time) ([]*SyncBatch, error)
    FindUncommitted(ctx, nodeID uuid.UUID) ([]*SyncBatch, error)
    FindByNode(ctx, nodeID uuid.UUID, limit int) ([]*SyncBatch, error)
    UpdateSignatures(ctx, batchID uuid.UUID, sourceSig, targetSig [64]byte) error
}

type SyncNodeRepository interface {
    Save(ctx, node *SyncNode) error
    FindByID(ctx, id uuid.UUID) (*SyncNode, error)
    FindBySite(ctx, siteID uuid.UUID) ([]*SyncNode, error)
    FindOnlineByType(ctx, nodeType NodeType) ([]*SyncNode, error)
    FindByMinistry(ctx, ministryID uuid.UUID) ([]*SyncNode, error)
    UpdateHeartbeat(ctx, nodeID uuid.UUID) error
    MarkOffline(ctx, nodeID uuid.UUID) error
}
```

### 7.4 Application Services (Use Cases)

| Use Case | Command | Description |
|---|---|---|
| Sync Partition | `SyncPartitionCommand { SourceNodeID, TargetNodeID, PartitionKey }` | Full sync cycle for one partition |
| Discover Peers | `DiscoverPeersCommand { NodeID }` | LAN discovery of other edge nodes |
| Resolve Conflict | `ResolveConflictCommand { BatchID, ConflictID, Resolution }` | Manual conflict resolution |
| Register Node | `RegisterNodeCommand { NodeType, SiteID, Hostname, Capabilities }` | Register a sync peer |
| Set Bandwidth | `SetBandwidthBudgetCommand { NodeID, UploadLimit, DownloadLimit }` | Configure bandwidth caps |
| Verify Merkle | `VerifyMerkleTreeCommand { NodeID, PartitionKey, RemoteRoot }` | Verify against remote merkle root |

---

## 8. notification-service

> **Bounded Context**: Notification Delivery
>
> **Domain**: Multi-channel notification dispatch, template management, delivery guarantees, ministry communication policies.
>
> **Language**: Go package `internal/domain/`

### 8.1 Aggregates

#### Aggregate: `Notification`

```go
// Notification aggregate root
// Invariants:
//   - Must have at least one delivery channel
//   - Retry count resets on channel change
//   - Notifications expire (configurable per ministry, default 7 days)
//   - Delivery status transitions: PENDING → SENDING → SENT/DELIVERED, or PENDING → FAILED
type Notification struct {
    id              uuid.UUID
    ministryID      uuid.UUID
    eventID         uuid.UUID?           // originating event
    recipientID     uuid.UUID            // user or group
    channels        []DeliveryChannel    // value object list
    title           string
    body            string
    templateID      uuid.UUID?
    locale          string               // "ar-IQ", "ku-IQ", "en-IQ"
    priority        NotificationPriority // HIGH (immediate), NORMAL (queued), LOW (batch)
    status          NotificationStatus
    deliveries      []Delivery           // entity list
    expiresAt       time.Time
    createdAt       time.Time
    version         int64
    domainEvents    []DomainEvent
}

type DeliveryChannel valueObject {
    channelType:    ChannelType          // PUSH, SMS, EMAIL, IN_APP, ON_SCREEN
    destination:    string               // device token, phone number, email
    isPrimary:      bool
    // Invariant: at least one channel must be isPrimary
}

type ChannelType enum {
    PUSH            // mobile push notification
    SMS             // text message
    EMAIL           // email
    IN_APP          // in-app notification center
    ON_SCREEN       // on-device display (biometric terminal screen)
}

type NotificationPriority enum {
    HIGH            // immediate delivery, bypass batching (e.g., security alerts)
    NORMAL          // queued, delivered within minutes
    LOW             // batched, delivered in next batch window
}

type NotificationStatus enum {
    PENDING
    SENDING
    PARTIALLY_DELIVERED
    DELIVERED       // at least one channel confirmed
    FAILED          // all channels failed after max retries
    EXPIRED
}

type Delivery struct {
    id:              uuid.UUID
    channelType:     ChannelType
    destination:     string
    status:          DeliveryStatus    // PENDING, SENT, DELIVERED, FAILED, READ
    attemptCount:    int
    maxAttempts:     int               // default 3
    lastAttemptAt:   time.Time?
    deliveredAt:     time.Time?
    readAt:          time.Time?
    errorCode:       string?
    errorMessage:    string?
}

type DeliveryStatus enum {
    PENDING
    SENT              // dispatched to channel provider
    DELIVERED         // confirmed delivered to device
    READ              // user opened/interacted
    FAILED
    BOUNCED           // invalid address/token
}

// Notification Domain Events
type NotificationCreated struct   { NotificationID, RecipientID, Priority, Channels }
type NotificationSent struct      { NotificationID, ChannelType, SentAt }
type NotificationDelivered struct { NotificationID, ChannelType, DeliveredAt }
type NotificationRead struct      { NotificationID, ChannelType, ReadAt, ReadBy }
type NotificationFailed struct    { NotificationID, ChannelType, Error, AttemptCount }
type NotificationExpired struct   { NotificationID }
```

#### Aggregate: `Template`

```go
// Template aggregate root
// Invariants:
//   - Template variables use {{mustache}} syntax
//   - Templates are versioned; breaking changes create new template
//   - Each locale has its own translation of the template
type Template struct {
    id              uuid.UUID
    ministryID      uuid.UUID
    name            string               // "leave-approved", "clock-in-reminder"
    eventType       string               // associated event type
    versions        []TemplateVersion    // entity list
    isActive        bool
    createdAt       time.Time
    domainEvents    []DomainEvent
}

type TemplateVersion struct {
    version:        int
    content:        map[string]LocalizedContent  // locale → content
    variables:      []string             // ["employee_name", "leave_type", "start_date"]
    channelSupport: map[ChannelType]bool // which channels this version supports
    createdBy:      uuid.UUID
    createdAt:      time.Time
    supersededAt:   time.Time?
}

type LocalizedContent struct {
    title:       string
    body:        string
    smsBody:     string?        // shortened for SMS
    pushData:    map[string]any?  // key-value pairs for push payload
}

// Template Domain Events
type TemplateCreated struct   { TemplateID, Name, EventType }
type TemplateVersioned struct { TemplateID, Version, CreatedBy }
type TemplateActivated struct { TemplateID, ActivatedAt }
type TemplateDeactivated struct { TemplateID, DeactivatedAt }
```

### 8.2 Domain Services

```go
// DeliveryOrchestrationService
// Responsibilities: channel selection, retry logic, fallback, batching
type DeliveryOrchestrationService interface {
    Dispatch(ctx, notification *Notification) error
    RetryFailed(ctx, notificationID) error
    SelectOptimalChannel(ctx, notification, userPreferences) (DeliveryChannel, error)
    BatchDeliver(ctx, notifications []*Notification) (BatchResult, error)
    FallbackChannel(ctx, notification, failedChannel) (DeliveryChannel, error)
}

// TemplateRenderingService
// Responsibilities: template variable substitution, localization, channel adaptation
type TemplateRenderingService interface {
    Render(ctx, templateID, locale, variables, channelType) (RenderedContent, error)
    ValidateTemplate(ctx, templateContent) (ValidationResult, error)
    PreviewTemplate(ctx, templateID, version, locale, variables) (RenderedContent, error)
}

type RenderedContent struct {
    Title:      string
    Body:       string
    SMSBody:    string?
    PushData:   map[string]any?
}

// ChannelAdapterService
// Responsibilities: abstract different channel providers (Firebase, SMS gateway, SMTP)
type ChannelAdapterService interface {
    SendPush(ctx, deviceTokens, title, body, data) (DeliveryResult, error)
    SendSMS(ctx, phoneNumber, message) (DeliveryResult, error)
    SendEmail(ctx, to, subject, body) (DeliveryResult, error)
    SendOnScreen(ctx, deviceID, message) (DeliveryResult, error)
}

type DeliveryResult struct {
    Success:     bool
    ProviderID:  string
    Error:       string?
}
```

### 8.3 Repository Interfaces

```go
type NotificationRepository interface {
    Save(ctx, notification *Notification) error
    FindByID(ctx, id uuid.UUID) (*Notification, error)
    FindByRecipient(ctx, recipientID uuid.UUID, status NotificationStatus, limit int) ([]*Notification, error)
    FindPendingDelivery(ctx, limit int) ([]*Notification, error)
    FindExpired(ctx) ([]*Notification, error)
    UpdateDeliveryStatus(ctx, notificationID uuid.UUID, deliveryID uuid.UUID, status DeliveryStatus) error
    MarkExpired(ctx, id uuid.UUID) error
}

type TemplateRepository interface {
    Save(ctx, template *Template) error
    FindByID(ctx, id uuid.UUID) (*Template, error)
    FindActiveByEventType(ctx, eventType string, ministryID uuid.UUID) (*Template, error)
    FindByName(ctx, name string, ministryID uuid.UUID) (*Template, error)
    FindByMinistry(ctx, ministryID uuid.UUID) ([]*Template, error)
}
```

### 8.4 Application Services (Use Cases)

| Use Case | Command | Description |
|---|---|---|
| Send Notification | `SendNotificationCommand { RecipientID, EventID, Channels, Title, Body, Priority }` | Creates and dispatches a notification |
| Create Template | `CreateTemplateCommand { Name, EventType, MinistryID, Content, Variables }` | Creates a notification template |
| Render Notification | `RenderNotificationCommand { NotificationID }` | Renders with template variables |
| Retry Delivery | `RetryDeliveryCommand { NotificationID, ChannelType }` | Retries failed delivery |
| Batch Dispatch | `BatchDispatchCommand { Priority, BatchSize }` | Sends batched low-priority notifications |
| Mark Read | `MarkNotificationReadCommand { NotificationID, ChannelType, ReadAt }` | Marks notification as read |

---

## 9. Context Maps

### 9.1 Partnership (Shared Kernel)

```
platform-core ◄────► identity-service
platform-core ◄────► attendance-service
platform-core ◄────► leave-service
platform-core ◄────► audit-ledger
platform-core ◄────► sync-engine
platform-core ◄────► notification-service
```

All contexts depend on platform-core's `MinistryID`, `EmployeeID`, `SiteID`, `DeviceID`, `DomainEvent`, and `SyncMetadata`.

### 9.2 Customer-Supplier

```
identity-service ────> attendance-service
    [User, Device]       [read-only references to employees/devices]

identity-service ────> leave-service
    [User]               [read-only references to employees]

identity-service ────> notification-service
    [User]               [read-only references to recipients]

sync-engine ────> audit-ledger
    [SyncBatch]          [every sync committed → audit event]
```

### 9.3 Conformist

```
attendance-service ────> identity-service
    [conforms to User aggregate for employee verification]

leave-service ────> identity-service
    [conforms to User aggregate for employee verification]

attendance-service ────> sync-engine
    [conforms to sync protocol for data exchange]
```

### 9.4 Event-Driven (via events)

```
attendance-service ────[events]────> notification-service
    clock-in.created → notify supervisor of late arrival

leave-service ────[events]────> notification-service
    request.approved → notify employee

identity-service ────[events]────> sync-engine
    user.registered → update sync data set

all services ────[events]────> audit-ledger
    all domain events → immutable audit trail
```

### 9.5 Aggregate Reference Map

| Reference | Source Context | Target Context | How Resolved |
|---|---|---|---|
| `EmployeeID` | Attendance, Leave | Identity | Eventual consistency via local cache; identity events keep it fresh |
| `DeviceID` | Attendance | Identity | Synchronous gRPC call for trust check; cached for offline |
| `UserID` | Notification | Identity | Local replica of user contact info |
| `MinistryID` | All | platform-core | Value object; no resolution needed |
| `SiteID` | All | platform-core | Value object; no resolution needed |

---

## 10. Event Storming Summary

### 10.1 Core Domain Events (by Domain)

```
IDENTITY DOMAIN:
    UserRegistered → UserVerified → CredentialAdded → RoleAssigned
    DeviceEnrolled → DeviceActivated → DeviceTrustChanged → DeviceSuspended

ATTENDANCE DOMAIN:
    ClockInCreated → AttendanceExceptionCreated (if late)
    ClockOutCreated → WorkedHoursCalculated
    AttendanceExceptionCreated → ExceptionJustified → ExceptionResolved
    AttendanceExceptionEscalated → ManualReviewCompleted

LEAVE DOMAIN:
    LeaveRequestCreated → LeaveRequestSubmitted → LeaveRequestApproved
    → BalanceDeducted → LeaveRequestCompleted
    LeaveRequestRejected → BalanceUnchanged

SYNC DOMAIN:
    SyncInitiated → MerkleExchangeCompleted → DeltaTransferred
    → ConflictDetected → ConflictResolved → BatchCommitted

AUDIT DOMAIN:
    EntryAppended → SealGenerated → SealPublished → SealVerified
```

### 10.2 Command ↔ Event Flow

```
[ClockInCommand]
    → ClockInCreated (domain event)
    → [attendance-service handles]
        → Save ClockEvent to repository
        → Publish event to outbox
        → [notify sync-engine]
        → [notify notification-service if supervisor wants alerts]
        → [publish to audit-ledger]
```

### 10.3 Saga: Leave Request Approval

```
Step 1: RequestLeaveCommand
    → LeaveRequestCreated event
    → LeaveRequest saved with status PENDING_APPROVAL

Step 2: DetermineApprovalChain
    → ApprovalSteps generated

Step 3: ApproveLeaveCommand (Step 1 approve)
    → LeaveRequestApproved event
    → Notification sent to Step 2 approver

Step 4: ApproveLeaveCommand (Step 2 approve)
    → LeaveRequestApproved event
    → Request status changes to APPROVED
    → BalanceDeducted
    → Notification sent to employee
    → Notification sent to payroll service
    → SyncBatch created for this partition

Step 5: (Compensation) RejectLeaveCommand at any step
    → LeaveRequestRejected event
    → Balance NOT deducted
    → Notification sent to employee
```

---

## Architecture Consistency Notes

All DDD structures above conform to the foundational architecture defined in `architecture/overview.md`:

| Architectural Rule | DDD Implementation |
|---|---|
| Offline-first | Every aggregate carries `SyncMetadata` value object; repositories support `FindUnsynced()` queries |
| Event-driven | Every aggregate produces domain events; event sourcing via `event_outbox` per service |
| Zero-trust | All authentication/authorization flows defined in identity-service aggregates |
| Immutable audit | `ClockEvent` and `LedgerEntry` are immutable aggregates (no UPDATE allowed) |
| PostgreSQL only | All repositories designed for PostgreSQL-specific features (RLS, partitioning, JSONB) |
| Multi-tenancy | Every aggregate carries `ministryID`; RLS enforced at repository level |
| No monolithic | Each bounded context is a separate Go module with its own domain package |
| Distributed | Sync-engine aggregates (`MerkleTree`, `SyncBatch`, `SyncNode`) handle all distribution concerns |
