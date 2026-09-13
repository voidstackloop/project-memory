// Superseded by src/encryption.rs (real AES-256-GCM + PBKDF2). This module is
// intentionally not declared in lib.rs and is not compiled — the old
// EncryptionConfig here claimed "aes-256-gcm"/"pbkdf2" but only ever
// implemented plain XOR, which was a live security landmine. Kept on disk
// only so history isn't silently lost; delete this file whenever convenient.
