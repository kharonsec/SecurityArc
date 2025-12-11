# SecureArc Security Model

## Threat Model

SecureArc is designed to protect against:

1. **Brute-Force Attacks**: Automated password guessing attempts
2. **Casual Attackers**: Unauthorized users attempting to access archives
3. **Key Recovery**: Attempts to extract encryption keys from archives

## Security Guarantees

### Encryption

- **AES-256-GCM**: Provides authenticated encryption with 256-bit keys
- **ChaCha20-Poly1305**: Provides authenticated encryption with 256-bit keys
- Both algorithms provide confidentiality, integrity, and authenticity

### Key Derivation

- **Argon2id**: Memory-hard function resistant to GPU and ASIC attacks
  - Recommended: 64 MB memory, 3 iterations, 4 threads
  - Makes brute-force attacks computationally expensive
- **PBKDF2-SHA256**: Legacy support with high iteration counts (≥10,000)

### Self-Destruct Mechanism

The self-destruct mechanism provides protection by:

1. **Attempt Tracking**: HMAC-protected counter prevents tampering
2. **Automatic Destruction**: After N failed attempts, key material is destroyed
3. **Rollback Prevention**: Counter is cryptographically bound to header

## Limitations

### Known Limitations

1. **File Copying**: Sophisticated attackers may create backup copies before attempting decryption
   - **Mitigation**: Strong encryption ensures brute-force remains infeasible even with unlimited attempts

2. **Custom Tools**: Attackers may use custom tools that bypass the reference implementation
   - **Mitigation**: Format specification is open, but encryption remains strong

3. **Side-Channel Attacks**: Timing or power analysis attacks are not explicitly mitigated
   - **Mitigation**: Use of well-audited cryptographic libraries (ring, RustCrypto)

4. **Header Backup**: Users may export header backups, allowing recovery
   - **Mitigation**: This is a feature, not a bug - allows recovery mechanisms

### Threat Scenarios

#### Scenario 1: Lost Password
- **Risk**: User forgets password after multiple failed attempts
- **Mitigation**: Recovery key slots can be configured with separate passwords

#### Scenario 2: Accidental Destruction
- **Risk**: User or automated system triggers self-destruct accidentally
- **Mitigation**: Header backups can be exported before distribution

#### Scenario 3: Malicious Destruction
- **Risk**: Attacker intentionally triggers destruction
- **Mitigation**: This is the intended behavior - prevents unauthorized access

## Security Recommendations

1. **Strong Passwords**: Use passwords with high entropy (≥128 bits)
2. **Appropriate Max Attempts**: Balance security (lower) vs usability (higher)
3. **Header Backups**: Export security headers before distribution
4. **Recovery Keys**: Configure recovery key slots for critical archives
5. **Regular Updates**: Keep cryptographic libraries up-to-date

## Cryptographic Analysis

### Key Strength

- Master keys: 256 bits (2^256 possible keys)
- Derived keys: 256 bits (from password via KDF)
- Brute-force resistance: Computationally infeasible with strong passwords

### Integrity Protection

- HMAC-SHA256: 256-bit security level
- Prevents tampering with attempt counter
- Binds counter to header cryptographically

### Compression Security

- Compression algorithms are applied before encryption
- No known plaintext attacks on compressed+encrypted data
- Standard compression algorithms (LZMA2, Zstd, Brotli) are well-studied

