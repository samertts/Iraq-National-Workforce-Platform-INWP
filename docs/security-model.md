# Security Model

See [architecture/overview.md §5 Security Architecture](../architecture/overview.md#5-security-architecture) for the complete zero-trust security architecture.

Key sections:
- Zero-trust model diagram and layers
- Certificate Authority hierarchy (Root CA → National CA → Ministry CA → Device CA)
- Authentication flows (user, service-to-service)
- Cryptographic inventory (Ed25519, X25519, AES-256-GCM, Argon2id)
- Security event monitoring
- Device trust model (§10.5)
