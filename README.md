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

- **32-bit Immediate Pattern Mining**: Checks all 4 segments of each 32-byte public key for valid 32-bit immediate values
- **Vanity Key Generation**: Generate keys with custom patterns (prefix, suffix, contains, position-based)
- **Batch Processing**: Process multiple patterns simultaneously for maximum efficiency
- Automatically detects and mines both positive and negative patterns
- 4x higher probability of finding valid keys compared to checking only one segment
- Multi-threaded for maximum performance (uses all available CPU cores)
- Real-time progress reporting with generation rates and per-pattern tracking
- Batch generation support for multiple keys and patterns
- Converts keypairs to assembly constants format
- Automatically saves matching keypairs to JSON files
- Multiple pattern types: prefix, suffix, contains, and position-based matching

## Installation

Install directly from GitHub using cargo:

```bash
cargo install --git https://github.com/blueshift-gg/doppler-keygen
```

## Usage

### Generate 32-bit Immediate Compatible Keys

```bash
# Find a single key with 32-bit immediate pattern
doppler-keygen grind

# Find multiple keys with 32-bit immediate pattern
doppler-keygen grind 10
```

### Generate Vanity Keys with Custom Patterns

```bash
# Find a key starting with "abc"
doppler-keygen vanity abc

# Find a key starting with "deadbeef"
doppler-keygen vanity deadbeef

# Find 5 keys starting with "mybrand"
doppler-keygen vanity mybrand 5

# Find a key ending with "cafe"
doppler-keygen vanity suffix:cafe

# Find a key containing "babe" anywhere
doppler-keygen vanity contains:babe

# Find a key with "pattern" starting at byte position 8
doppler-keygen vanity at:8:pattern
```

### Batch Process Multiple Patterns

```bash
# Find one key each for 'ab' and 'te' patterns
doppler-keygen batch ab:1 te:1

# Find 2 keys for 'sol' and 1 key for 'web'
doppler-keygen batch sol:2 web:1

# Mix different pattern types in one batch
doppler-keygen batch abc:1 suffix:xyz:2 contains:test:1
```

### Convert Keypair to Assembly Format

```bash
# Convert a keypair file to assembly constants
doppler-keygen address keypair.json
```

This outputs the public key in little-endian assembly constant format:
```asm
.equ EXPECTED_ADMIN_KEY_0, 0x683ad07a38261fb5
.equ EXPECTED_ADMIN_KEY_1, 0xb8cd5a8f402a8499
.equ EXPECTED_ADMIN_KEY_2, 0x819d9dac
.equ EXPECTED_ADMIN_KEY_3, 0x8b15d101a873733c
```

It also outputs the correct assembly instructions to handle optimal key comparisons:

```asm
ldxdw r2, [r1+ADMIN_KEY_0]
lddw r3, EXPECTED_ADMIN_KEY_0
jne r2, r3, abort
ldxdw r2, [r1+ADMIN_KEY_1]
lddw r3, EXPECTED_ADMIN_KEY_1
jne r2, r3, abort
ldxdw r2, [r1+ADMIN_KEY_2]
jne r2, EXPECTED_ADMIN_KEY_2, abort
ldxdw r2, [r1+ADMIN_KEY_3]
lddw r3, EXPECTED_ADMIN_KEY_3
jne r2, r3, abort
```

## Batch Processing

The `batch` command allows you to process multiple vanity patterns simultaneously, making it much more efficient than running individual vanity searches.

### Batch Syntax

```bash
doppler-keygen batch <pattern1:count1> [pattern2:count2] ...
```

### Batch Examples

```bash
# Find one key each for 'ab' and 'te' patterns
doppler-keygen batch ab:1 te:1

# Find 2 keys for 'sol' and 1 key for 'web'
doppler-keygen batch sol:2 web:1

# Mix different pattern types
doppler-keygen batch abc:1 suffix:xyz:2 contains:test:1
```

### Batch Output

Batch processing provides:
- **Real-time progress** for each pattern individually
- **Per-pattern status** (✅ completed, ⏳ in progress)
- **Combined statistics** across all patterns
- **Efficient resource usage** - all patterns processed simultaneously

Example batch output:
```
Batch processing 3 patterns (0/5 keys found)
  ab: 0/1
  te: 0/2
  xy: 0/2

Found key for 'ab': 7Z9k2qTpgbF1HKJwa1hi2eGqnePMssWMgYjrJdgqjJhT
...

Completed: 5/5 keys in 0.5s (450000 keys/sec)
```

## Vanity Pattern Guide

### Pattern Types

| Type | Syntax | Description | Example |
|------|--------|-------------|---------|
| **Prefix** | `abc123` or `prefix:abc123` | Match pattern at the start of the key | `abc123...` |
| **Suffix** | `suffix:cafe` | Match pattern at the end of the key | `...cafe` |
| **Contains** | `contains:babe` | Match pattern anywhere in the key | `...babe...` |
| **Position** | `at:8:pattern` | Match pattern starting at byte position | `........pattern...` |

### Pattern Probability

The probability of finding a vanity key depends on the pattern length:

- **2 characters**: ~1 in 4,096 attempts
- **3 characters**: ~1 in 16,777,216 attempts
- **4 characters**: ~1 in 4.29 billion attempts
- **5 characters**: ~1 in 1.1 trillion attempts

**Tips for faster generation:**
- Use shorter patterns (2-3 characters) for quick results
- Use common patterns for demonstration
- Longer patterns may take hours/days to find

### Vanity Key Output

When a vanity key is found, you'll see:

```
✅ FOUND VANITY KEYPAIR #1/1
Thread: 6
Public Key: 616263159a6c9de83d7c23b17e5e19f64771f04b7ab22cc41b95b2c3be613d7f
Public Key (base58): 7Z9ZX9oRtDiqdAoJmFuGGpwN9oRBzqvqFLtPZGsaR7NN
Pattern: Prefix("abc")
Matched bytes: [97, 98, 99, 21, 154, 108, 157, 232, 61, 124, 35, 177, 126, 94, 25, 246, 71, 113, 240, 75, 122, 178, 44, 196, 27, 149, 178, 195, 190, 97, 61, 127]

Keypair saved to: vanity_abc_7Z9ZX9oRtDiqdAoJmFuGGpwN9oRBzqvqFLtPZGsaR7NN.json
```

## Building from Source

```bash
git clone https://github.com/blueshift-gg/doppler-keygen
cd doppler-keygen
cargo build --release
```

## Output

### 32-bit Immediate Key Output

When a 32-bit immediate compatible key is found:
- Displays the public key in both hex and base58 format
- Shows which segment matched and the i32/i64 values
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

### Vanity Key Output

When a vanity key is found:
- Displays the public key in both hex and base58 format
- Shows the matched pattern type and the actual matched bytes
- Saves the keypair to a JSON file with vanity pattern in filename
- Shows generation statistics including attempts and keys/sec

Example output:
```
✅ FOUND VANITY KEYPAIR #1/1
Thread: 6
Public Key: 616263159a6c9de83d7c23b17e5e19f64771f04b7ab22cc41b95b2c3be613d7f
Public Key (base58): 7Z9ZX9oRtDiqdAoJmFuGGpwN9oRBzqvqFLtPZGsaR7NN
Pattern: Prefix("abc")
Matched bytes: [97, 98, 99, 21, 154, 108, 157, 232, 61, 124, 35, 177, 126, 94, 25, 246, 71, 113, 240, 75, 122, 178, 44, 196, 27, 149, 178, 195, 190, 97, 61, 127]

Keypair saved to: vanity_abc_7Z9ZX9oRtDiqdAoJmFuGGpwN9oRBzqvqFLtPZGsaR7NN.json
```

## Why These Patterns?

### 32-bit Immediate Pattern
This specific pattern is useful for:
- Embedding 32-bit immediate values directly in public keys
- VM or assembly contexts where sign extension matters
- Ensuring compatibility between 32-bit and 64-bit representations
- Optimized on-chain operations that can leverage these patterns
- Smart contracts that need efficient constant comparisons

### Vanity Keys
Vanity keys are useful for:
- **Brand recognition**: Create memorable keys starting with your brand name
- **Personalization**: Keys that reflect your identity or project
- **Marketing**: Eye-catching key patterns for social media and presentations
- **Organization**: Group related keys by common prefixes
- **Debugging**: Easily identifiable keys in logs and transactions
- **Aesthetics**: Keys that look good in base58 format