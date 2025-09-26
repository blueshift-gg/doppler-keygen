# Doppler Keygen

A multi-threaded Solana keypair generator that searches for public keys where any 8-byte segment forms a valid 32-bit immediate value with proper sign extension.

## The Pattern

This tool mines for Solana public keys where any of the four 8-byte segments can be used as a 32-bit immediate value in assembly or VM contexts. The pattern ensures compatibility with sign extension from i32 to i64.

### How It Works

The 32-byte public key is divided into 4 segments:
- **Segment 0**: bytes 0-7
- **Segment 1**: bytes 8-15
- **Segment 2**: bytes 16-23
- **Segment 3**: bytes 24-31

For each segment, the first 4 bytes are interpreted as a little-endian i32 value, and bytes 4-7 must match the sign extension pattern:

- **Positive values** (bit 31 clear, byte 3 of segment < 0x80):
  - Bytes 4-7 of the segment must all be `0x00`
  - Example: `12 34 56 78 | 00 00 00 00` → i32 = 0x78563412 → i64 = 0x0000000078563412

- **Negative values** (bit 31 set, byte 3 of segment ≥ 0x80):
  - Bytes 4-7 of the segment must all be `0xFF`
  - Example: `12 34 56 80 | FF FF FF FF` → i32 = 0x80563412 → i64 = 0xFFFFFFFF80563412

The miner will accept a key if **any** of the 4 segments matches this pattern, making valid keys 4x more likely to be found.

## Features

- Checks all 4 segments of each 32-byte public key
- Automatically detects and mines both positive and negative patterns
- 4x higher probability of finding valid keys compared to checking only one segment
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
Public Key: abcdef1234567890...1234567800000000...
Public Key (base58): 2xV4K9...
Matched Segment: 2 (bytes 16-23)
Segment 2 bytes (hex): 12 34 56 78 | 00 00 00 00
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