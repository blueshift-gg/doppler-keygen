# Doppler Keygen

A multi-threaded Solana keypair generator that searches for public keys that match the required doppler-asm byte pattern.

## Features

- Generates Solana keypairs where bytes 4-7 of the public key are all zeros
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
- Saves the keypair to a JSON file named after the public key
- Shows generation statistics including attempts and keys/sec
- Supports batch generation with progress tracking for multiple keys