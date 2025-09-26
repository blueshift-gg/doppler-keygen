# Doppler Keygen

A multi-threaded Solana keypair generator that searches for public keys where the first 8 bytes form a valid 32-bit immediate value with proper sign extension.

## The Pattern

This tool mines for Solana public keys where the first 8 bytes can be used as a 32-bit immediate value in assembly or VM contexts. The pattern ensures compatibility with sign extension from i32 to i64:

### How It Works

The first 4 bytes (0-3) of the public key are interpreted as a little-endian i32 value. Bytes 4-7 must match the sign extension pattern:

- **Positive values** (bit 31 clear, byte 3 < 0x80):
  - Bytes 4-7 must all be `0x00`
  - Example: `12 34 56 78 | 00 00 00 00` → i32 = 0x78563412 → i64 = 0x0000000078563412

- **Negative values** (bit 31 set, byte 3 ≥ 0x80):
  - Bytes 4-7 must all be `0xFF`
  - Example: `12 34 56 80 | FF FF FF FF` → i32 = 0x80563412 → i64 = 0xFFFFFFFF80563412

This ensures that when the first 4 bytes are read as an i32 and sign-extended to i64, the result matches exactly what's stored in the full 8 bytes of the public key.

## Features

- Automatically detects and mines both positive and negative patterns
- Multi-threaded for maximum performance
- Real-time progress reporting
- Batch generation support for multiple keys
- Converts keypairs to assembly constants format
- Automatically saves matching keypairs to JSON files

## Installation

Install directly from GitHub using cargo:

```bash
cargo install --git https://github.com/blueshift-gg/doppler-keygen
```

## Usage

### Grind for vanity keys

```bash
# Find a single vanity key
doppler-keygen grind

# Find multiple vanity keys
doppler-keygen grind 10
```

### Convert keypair to assembly format

```bash
# Convert a keypair file to assembly constants
doppler-keygen address keypair.json
```

This outputs the public key in little-endian assembly constant format:
```asm
.equ EXPECTED_ADMIN_KEY_0, 0xef390f15
.equ EXPECTED_ADMIN_KEY_1, 0xabba7e83075e8fbb
.equ EXPECTED_ADMIN_KEY_2, 0x88fc9fa7225f4ff6
.equ EXPECTED_ADMIN_KEY_3, 0xd9900e10585ab4e2
```

## Building from Source

```bash
git clone https://github.com/blueshift-gg/doppler-keygen
cd doppler-keygen
cargo build --release
```

## Output

When a matching keypair is found:
- Displays the public key in both hex and base58 format
- Shows the i32 value and its sign-extended i64 representation
- Saves the keypair to a JSON file named after the public key
- Shows generation statistics including attempts and keys/sec

Example output:
```
✅ FOUND MATCHING KEYPAIR #1/1
Thread: 3
Public Key: 1234567800000000abcdef...
Public Key (base58): 2xV4K9...
First 8 bytes (hex): 12 34 56 78 | 00 00 00 00
  i32 value: 2018915346 (0x78563412)
  i64 value: 2018915346 (0x0000000078563412)

Keypair saved to: 2xV4K9....json
```

## Why This Pattern?

This specific pattern is useful for:
- Embedding 32-bit immediate values directly in public keys
- VM or assembly contexts where sign extension matters
- Ensuring compatibility between 32-bit and 64-bit representations
- Optimized on-chain operations that can leverage these patterns