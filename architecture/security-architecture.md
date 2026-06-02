# INWP Security Architecture

> Zero-trust security architecture for national-scale deployment: PKI, MFA, RBAC, device identity, signed events, immutable audit, secrets management, and zero-trust networking.

---

## 1. Zero-Trust Model

```
                    NEVER TRUST, ALWAYS VERIFY
                    +---------------------------+
                    | Every request:            |
                    | 1. Who is it? (Identity)  |
                    | 2. What can they do? (Auth)|
                    | 3. Is it allowed? (Policy)|
                    | 4. Is it recorded? (Audit)|
                    +---------------------------+
                               |
        +----------------------+----------------------+
        |                      |                      |
   mTLS Verify            JWT Verify            Policy Evaluate
   - Certificate chain    - Signature            - RBAC roles
   - CRL check            - Expiry               - ABAC attributes
   - Validity period      - Claims               - Context
   - Issuer chain         - Audience             - Ministry/site
        |                      |                      |
        +----------------------+----------------------+
                               |
                         [AUDIT LOG]
                     Every decision recorded
```

---

## 2. Public Key Infrastructure (PKI)

### 2.1 Certificate Authority Hierarchy

```
            +-----------------------------------+
            | INWP ROOT CA (OFFLINE)            |
            | HSM: Luna SA 7 (or equivalent)    |
            | Algorithm: ECC P-384               |
            | Validity: 20 years                 |
            | Storage: Air-gapped, powered off   |
            | Access: 2-of-3 multi-party         |
            | Purpose: Sign intermediate CAs     |
            +---------------+-------------------+
                            |
          +-----------------+-------------------+
          |                 |                   |
+---------v--------+  +----v-----------v  +----v-----------+
| NATIONAL CA      |  | MINISTRY CA      |  | DEVICE CA      |
| (ONLINE)         |  | (PER MINISTRY)   |  | (PER TENANT)   |
| HSM: Luna SA 7   |  | Software HSM     |  | Software HSM   |
| Algorithm: P-384 |  | Algorithm: P-256 |  | Algorithm: P-256|
| Validity: 5 yrs  |  | Validity: 3 yrs  |  | Validity: 5 yrs |
| Purpose: Service  |  | Purpose: Admin   |  | Purpose: Device |
|          certs    |  |          certs   |  |          certs  |
+------------------+  +------------------+  +----------------+
     |                      |                      |
     v                      v                      v
+-----------+        +-------------+        +-------------+
| SERVICE   |        | MINISTRY    |        | BIOMETRIC   |
| CERTS     |        | ADMIN CERTS |        | TERMINAL    |
| (90 day)  |        | (30 day)    |        | CERTS       |
|           |        |             |        | (7 day)     |
| gRPC mTLS |        | Web Portal  |        | Device mTLS |
| JWT sign  |        | API access  |        | Auto-renew  |
+-----------+        +-------------+        +-------------+
```

### 2.2 Certificate Lifecycle

```
1. ISSUANCE
   - CSR generated on device/server
   - Private key NEVER leaves the originating HSM/TPM
   - CSR submitted to appropriate CA via authenticated API
   - CA verifies identity (attestation for devices, admin approval for users)
   - Certificate issued, signed, returned
   - Certificate logged in certificate transparency log

2. RENEWAL
   - Automatic: 30% of validity period remaining initiates renewal
   - Manual: admin triggers renewal via certificate management API
   - Renewal uses new keypair (key rotation)
   - Old certificate added to CRL

3. REVOCATION
   - Immediate revocation on compromise notification
   - CRL published within 1 hour of revocation
   - CRL distributed via sync engine to all nodes within 24h
   - Compromised certificate serial added to CRL

4. EXPIRY
   - Alert at T-30 days, T-7 days, T-1 day
   - Auto-renewal at T-7 days (if auto-renewal configured)
   - Expired certificates rejected at mTLS handshake
   - Services with expired certs enter degraded mode

5. AUDIT
   - All certificate operations logged to audit ledger
   - Certificate transparency logs published periodically
   - CRL distribution verified by receipt
```

### 2.3 Certificate Profile

| Field | Service Cert | Admin Cert | Device Cert |
|---|---|---|---|
| Algorithm | ECC P-384 | ECC P-256 | ECC P-256 |
| Validity | 90 days | 30 days | 7 days |
| Key Usage | digitalSignature, keyEncipherment | digitalSignature | digitalSignature |
| Extended Key Usage | serverAuth, clientAuth | clientAuth | clientAuth |
| SAN | service.inwp.iq, node_id | user@ministry.inwp.iq | device_id.site.inwp.iq |
| CRL Distribution | ca.inwp.iq/crl | ca.inwp.iq/crl | ca.inwp.iq/crl |

---

## 3. Multi-Factor Authentication (MFA)

### 3.1 MFA Methods

| Method | Factor Type | Security Level | Offline | User Experience |
|---|---|---|---|---|
| Password | Knowledge | Medium | N/A | Standard |
| TOTP (Google Auth) | Possession | High | Yes (synced seed) | Scan QR, enter 6-digit |
| WebAuthn/FIDO2 | Possession + Biometric | Very High | Yes (hardware key) | Touch/face/print |
| Smart Card (PKCS#11) | Possession | Very High | Yes | Insert card, enter PIN |
| SMS OTP | Possession | Medium | No | Receive SMS, enter code |
| Biometric (on-device) | Inherence | High | Yes | Fingerprint/face scan |

### 3.2 MFA Policy Configuration

```yaml
# Per-ministry MFA policy (stored in policy-engine)
mfa_policy:
  mohe:                              # Ministry of Education
    required: true
    methods: [password+totp, biometric+pin]
    min_factors: 2
    session_timeout: 15m
    remember_device: 7d             # Trusted device, skip MFA for 7 days
  moh:                              # Ministry of Health
    required: true
    methods: [password+totp, smartcard]
    min_factors: 2
  moo:                              # Ministry of Oil
    required: true
    methods: [smartcard+pin, fido2]
    min_factors: 2
  national_admins:                  # Override for national admin role
    required: true
    methods: [smartcard+pin+totp]   # 3-factor for national admins
    min_factors: 3
```

### 3.3 Offline MFA

```
Challenge: MFA must work when national DC is unreachable

Solution: Local MFA verification at edge nodes
- TOTP seeds are synced (encrypted) to edge nodes
- WebAuthn relies on local hardware key (no server needed)
- Smart card certificates validated locally (CRL cached)
- Biometric templates stored locally on device

Limitations:
- SMS/phone MFA not available offline
- Device trust level affects MFA policy (lower trust = stricter MFA)
- MFA policy changes sync with next connectivity
```

---

## 4. Role-Based Access Control (RBAC)

### 4.1 Role Definition

```yaml
roles:
  - name: national_admin
    scope: national
    permissions:
      - system:configure
      - system:audit:all
      - user:manage:all
      - ministry:manage:all
    constraints:
      mfa_required: true
      mfa_method: smartcard+pin+totp

  - name: ministry_admin
    scope: ministry
    permissions:
      - user:manage:{ministry}
      - policy:manage:{ministry}
      - report:view:{ministry}
    constraints:
      mfa_required: true

  - name: attendance_operator
    scope: site
    permissions:
      - attendance:clock-events:manage:{site}
      - attendance:exceptions:manage:{site}
      - attendance:shifts:view:{site}
```

### 4.2 Permission Inheritance

```
national_admin       -> All permissions across all ministries/regions/sites
ministry_admin       -> All permissions within specific ministry
site_admin           -> All permissions within specific site
hr_operator          -> User management, attendance/leave operations
attendance_operator  -> Clock events, exceptions, shift view only
leave_approver       -> Leave approval authority
viewer               -> Read-only dashboard access
```

### 4.3 Attribute-Based Access Control (ABAC)

Additional attributes evaluated at runtime:

| Attribute | Source | Example |
|---|---|---|
| ministry_id | JWT claim / request path | `mohe` |
| site_id | JWT claim / request path | `basra-univ-01` |
| device_id | mTLS certificate CN | `bio-scanner-42` |
| device_trust_level | Device registry | `trusted` |
| time_of_day | Request timestamp | `07:30` |
| day_of_week | Request timestamp | `Sunday` |
| network_zone | mTLS certificate SAN | `biometric-vlan` |
| auth_method | JWT claim | `password+totp` |
| ip_address | Request source IP | `192.168.50.42` |
| geo_location | GPS (mobile requests) | `33.3152, 44.3661` |

Example ABAC policy:
```rego
# Allow clock-in if:
# 1. User has attendance_operator role at this site
# 2. Device trust level >= TRUSTED
# 3. Time is within shift window + grace period
# 4. Device is on biometric VLAN
allow {
    input.role == "attendance_operator"
    input.device_trust_level == "trusted"
    time_within_shift_window(input.employee_id, input.time)
    input.network_zone == "biometric-vlan"
}
```

---

## 5. Device Identity

### 5.1 Device Enrollment Flow

```
1. PHYSICAL SETUP
   Device installed at site, connected to LAN
   Factory default: manufacturer keypair in TPM (optional)

2. DISCOVERY
   Device broadcasts presence via mDNS
   Device Gateway discovers and initiates enrollment

3. ATTESTATION
   Device generates ephemeral keypair for enrollment
   If TPM: device signs nonce with TPM attestation key
   If no TPM: device presents manufacturer certificate (if available)
   If no trust anchor: manual admin approval required

4. ENROLLMENT
   POST /device-gateway/v1/enroll { attestation_blob, device_info, public_key }
   Device Gateway validates attestation
   Device CA issues device certificate (7-day TTL, auto-renewable)
   Device registered in device_registry (initial trust: PROVISIONAL)

5. ACTIVATION
   Device connects with mTLS using issued certificate
   If attestation verified: trust level -> TRUSTED
   If manual approval: trust level -> TRUSTED (after admin approval)
   If attestation incomplete: trust level remains PROVISIONAL

6. OPERATION
   All device communication via mTLS
   Heartbeat every 60s
   Certificate auto-renewal at T-24h
   Trust score computed and updated
```

### 5.2 Device Trust Score Computation

```
Trust Score = weighted combination (0.0 - 1.0):

Score = 0.40 * attestation_recency
      + 0.20 * firmware_recency
      + 0.15 * heartbeat_regularity
      + 0.15 * anomaly_absence
      + 0.10 * certificate_validity

attestation_recency:
  1.0 if attested within 7 days
  0.8 if within 14 days
  0.5 if within 30 days
  0.1 if >30 days

firmware_recency:
  1.0 if running latest firmware
  0.7 if one version behind
  0.3 if two+ versions behind
  0.0 if firmware unknown

heartbeat_regularity:
  1.0 if all heartbeats within 60s
  0.9 if 90th percentile < 120s
  0.5 if 90th percentile < 300s
  0.1 if missed >10 heartbeats in 24h

anomaly_absence:
  1.0 if no anomalies in 30 days
  0.5 if 1-2 anomalies
  0.0 if 3+ anomalies

certificate_validity:
  1.0 if >70% validity remaining
  0.8 if >30% validity remaining
  0.3 if <30% validity remaining
  0.0 if expired
```

### 5.3 Device Trust Level Thresholds

| Trust Level | Score Range | Actions |
|---|---|---|
| TRUSTED | 0.8 - 1.0 | Full access |
| PROVISIONAL | 0.5 - 0.79 | Read-only, clock-in limited |
| DEGRADED | 0.3 - 0.49 | Read-only, alerts generated |
| SUSPENDED | 0.1 - 0.29 | Blocked, admin notification |
| REVOKED | 0.0 - 0.09 | All access blocked, cert revoked |

---

## 6. Signed Events

### 6.1 Event Signing

Every event produced by any service is signed with Ed25519:

```
1. Producer creates event payload (JSON)
2. Producer computes SHA-256 hash of canonical JSON payload
3. Producer signs hash with Ed25519 private key
4. Event published with: {payload, signature, signing_key_id}

Signature verification:
1. Consumer receives event
2. Consumer looks up public key for signing_key_id
3. Consumer computes SHA-256 of payload
4. Consumer verifies Ed25519 signature
5. If invalid: event rejected, security alert triggered
```

### 6.2 Key Management for Signing

```
Key Hierarchy:
  National Level:  National signing key (HSM-backed, 1-year rotation)
  Ministry Level:  Ministry signing key (Vault transit, 90-day rotation)
  Service Level:   Service signing key (Vault transit, 30-day rotation)
  Edge Node:       Node signing key (local, rotates on sync)

Key Distribution:
  - Public keys published to key registry (synced to all nodes)
  - Private keys stored in Vault (national) or encrypted local storage (edge)
  - Key rotation events published: inwp.security.v1.key.rotated
  - Grace period for signature verification: 2x key rotation interval
```

---

## 7. Immutable Audit

See detailed audit ledger architecture in section 19 of the main architecture document.

### 7.1 Audit Integrity Verification

```
Automated daily verification:
1. Walk hash chain from last verified checkpoint
2. Recompute each entry_hash
3. Verify: entry_hash == SHA-256(prev_hash || payload_hash || nonce)
4. Verify chain continuity: entry[i].prev_hash == entry[i-1].entry_hash
5. Verify periodic seals
6. Generate verification report

On integrity failure:
1. Identify corrupted entry position
2. Quarantine affected entries
3. Alert: audit.integrity.failure (SEVERITY: CATASTROPHIC)
4. Initiate incident response
5. Restore from replicated copy or backup
```

---

## 8. Secrets Management

### 8.1 Secrets Architecture

```
National DC:
  HashiCorp Vault (3-node HA cluster)
  +-- Transit Engine: encryption-as-a-service
  +-- PKI Engine: dynamic certificate issuance
  +-- KV Engine: service credentials, API keys
  +-- Identity Engine: Kubernetes auth, JWT auth
  +-- Audit: all secret access logged to file + audit ledger

  Secret types:
  - Database credentials (dynamic, short-lived)
  - Encryption keys (envelope encryption, per-tenant)
  - Service account keys (JWT signing, mTLS)
  - API tokens (SMS gateway, email, push notifications)
  - Private keys (event signing, sync batch signing)

Regional Hub:
  Vault Agent (caching token, secret refresh)
  - Authenticates to national Vault via JWT auth
  - Caches secrets locally with configurable TTL
  - Refreshes before TTL expiry
  - No persistent secret storage at regional level

Edge Node:
  Docker Secrets (mounted files)
  - Secrets injected at container start via Docker Compose
  - Secrets provisioned during bootstrap
  - Secrets rotated on each successful sync
  - No Vault client at edge (reduced dependency, offline capability)
```

### 8.2 Secret Rotation

| Secret Type | Rotation | Mechanism |
|---|---|---|
| Database passwords | 90 days | Dynamic secrets (Vault) or automated rotation |
| JWT signing keys | 30 days | Vault transit key rotation |
| mTLS certificates | Per certificate profile | Vault PKI auto-renewal |
| Encryption keys | 1 year (master), per-use (data) | Vault transit rewrap |
| API tokens | 90 days | Manual (via Vault UI) |
| SSH keys | 180 days | Automated rotation script |

### 8.3 No Hardcoded Secrets Rule

```
ENFORCED BY:
1. CI/CD pipeline: Gitleaks scan on every commit (fail on secret detection)
2. Code review: human inspection for hardcoded credentials
3. Runtime: application crashes if secrets not provided via expected mechanism
4. Audit: secret access patterns monitored for anomalies

SECRET INJECTION METHODS (in priority order):
1. Vault (national DC) - dynamic, audited, rotated
2. Kubernetes Secrets (national DC) - encrypted at rest via KMS
3. Docker Secrets (edge) - mounted as tmpfs files
4. Environment variables - ONLY for non-sensitive config (ports, log levels)

NEVER IN:
- Source code files
- Configuration files in git
- Docker image layers
- Log files
- Error messages
- Stack traces
```

---

## 9. Zero-Trust Networking

### 9.1 Network Segmentation

```
+-----------------------+        +-----------------------+
|   PUBLIC NETWORK      |        |   MANAGEMENT NETWORK   |
|   (Internet-facing)   |        |   (Admin access)       |
|                       |        |                        |
| - Web Portal          |        | - SSH bastion          |
| - API Gateway (443)   |        | - Vault UI             |
| - Mobile API (443)    |        | - Grafana              |
| - mTLS termination    |        | - K8s API (internal)   |
+-----------+-----------+        +-----------+------------+
            |                                |
            v                                v
+-----------+-----------+        +-----------+------------+
|   SERVICE NETWORK      |        |   DEVICE NETWORK       |
|   (Internal traffic)   |        |   (Biometric hardware) |
|                        |        |                        |
| - All microservices    |        | - Biometric terminals  |
| - PostgreSQL           |        | - Card readers         |
| - NATS                 |        | - Attendance kiosks    |
| - Redis                |        | - Restricted egress    |
| - mTLS required        |        | - VLAN isolated        |
+------------------------+        +------------------------+
```

### 9.2 Firewall Rules

```
PUBLIC -> SERVICE NETWORK:
  ALLOW: TCP/443 (HTTPS) from public to API Gateway
  ALLOW: TCP/443 (HTTPS) from public to Web Portal
  DENY: all other inbound

SERVICE NETWORK INTERNAL:
  ALLOW: TCP/5432 (PostgreSQL) between service pods
  ALLOW: TCP/4222 (NATS) between service pods
  ALLOW: TCP/6379 (Redis) between service pods
  ALLOW: TCP/8200 (Vault) from services to Vault
  ALLOW: TCP/9090 (Prometheus) from monitoring to services
  ALLOW: TCP/4318 (OTLP) from services to OpenTelemetry Collector
  DENY: all other inter-service

DEVICE NETWORK:
  ALLOW: TCP/8443 (sync) from device-gateway to service network
  ALLOW: TCP/8080 (device API) from devices to device-gateway
  DENY: all other egress from device VLAN
  DENY: all inbound from device VLAN to service network (except device-gateway)
```

### 9.3 mTLS Requirements

```
ALL service-to-service communication requires mTLS 1.3:

1. Connection establishment
   - Client presents certificate (issued by National CA or sub-CA)
   - Server presents certificate (issued by National CA or sub-CA)
   - Both verify peer certificate: chain, validity, CRL, purpose

2. Certificate validation
   - CRL check (cached, max 1h staleness)
   - Certificate validity period
   - Key usage (digitalSignature + clientAuth/serverAuth)
   - Issuer chain (must trace to INWP Root CA)

3. Connection rejection
   - Invalid certificate chain
   - Revoked certificate (in CRL)
   - Expired certificate
   - Wrong key usage
   - Unknown CA

4. Mutual authentication
   - Both sides identify each other
   - Service identity from certificate CN/SAN
   - Device identity from certificate CN/SAN
```

---

## 10. Security Incident Response

### 10.1 Incident Severity Levels

| Level | Definition | Response Time | Examples |
|---|---|---|---|
| SEV-0 | Critical system-wide compromise | Immediate (<15min) | Root CA compromise, audit tampering |
| SEV-1 | High-impact security breach | <1 hour | Service compromise, data exfiltration |
| SEV-2 | Medium-impact incident | <4 hours | Device compromise, policy violation |
| SEV-3 | Low-impact event | <24 hours | Suspicious activity, minor policy violation |
| SEV-4 | Informational | Next business day | Vulnerability report, scan findings |

### 10.2 Incident Response Playbooks

Stored in `security/incident-response/playbooks/`:

| Playbook | Description |
|---|---|
| `root-ca-compromise.md` | Root CA key compromise: re-issue all certificates |
| `service-compromise.md` | Service account compromise: rotate keys, audit logs |
| `device-compromise.md` | Device compromise: revoke cert, isolate, forensics |
| `data-breach.md` | Data exfiltration: contain, assess, notify |
| `audit-tampering.md` | Audit log tampering: restore from backup, investigate |
| `denial-of-service.md` | DoS/DDoS: rate limiting, traffic filtering, scaling |
| `insider-threat.md` | Insider abuse: account suspension, forensics, legal |
| `certificate-compromise.md` | Certificate/private key exposure: revoke, re-issue |
