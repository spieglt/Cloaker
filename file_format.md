# Cloaker File Format

This document outlines the file format for files encrypted with Cloaker.
There are two versions of the file format: the current version (Cloaker 2.0) and a legacy version (Cloaker 1.0).

  * Cloaker verison 1.1 uses Cloaker 1.0 File Format, without a magic signature.
  * Cloaker verison 3.1 uses Cloaker 1.0 File Format, with a magic signature.
  * Cloaker verison 4.0 uses Cloaker 2.0 File Format.

## Cloaker 2.0 File Format

The file is structured as follows:

| Part      | Size (bytes) | Description                                                                 |
| --------- | ------------ | --------------------------------------------------------------------------- |
| Signature | 4            | A magic number to identify the file as a Cloaker 2.0 encrypted file. The value is `[0xC1, 0x0A, 0x6B, 0xED]`. |
| Salt      | 16           | A salt used for key derivation with the Argon2id13 algorithm.               |
| Header    | 24           | The header for the `xchacha20poly1305` secret stream.                       |
| Encrypted Data | variable     | The rest of the file contains the encrypted data, split into chunks.        |

Hash for password derived using libsodium / sodiumoxide https://doc.libsodium.org/password_hashing/default_phf crypto_pwhash_ALG_ARGON2ID13: version 1.3 of the Argon2id algorithm, available since libsodium 1.0.13.

Encrypted Data is generated with libsodium / sodiumoxide streaming encryption. See https://doc.libsodium.org/secret-key_cryptography/secretstream crypto_secretstream_xchacha20poly1305 the `crypto_secretstream_*()` API was introduced in libsodium 1.0.14.

## Cloaker 1.0 File Format (Legacy)

The legacy file format is similar to the 2.0 one, but with a few key differences:

| Part      | Size (bytes) | Description                                                                 |
| --------- | ------------ | --------------------------------------------------------------------------- |
| Signature | 4 (optional) | A magic number to identify the file as a Cloaker 1.0 encrypted file. The value is `[0xC1, 0x0A, 0x4B, 0xED]`. This signature is **optional**. |
| Salt      | 16           | A salt used for key derivation with an older pwhash algorithm. (scryptsalsa208sha256?)             |
| Header    | 24           | The header for the `xchacha20poly1305` secret stream.                       |
| Encrypted Data | variable     | The rest of the file contains the encrypted data, split into chunks.        |

Hash for password derived using libsodium / sodiumoxide https://doc.libsodium.org/password_hashing#scrypt

The primary differences in the legacy format are:

*   The signature has a different value and is optional.
*   A different key derivation algorithm (`pwhash`) is used.
