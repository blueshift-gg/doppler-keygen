use core::sync::atomic::{AtomicU64, Ordering};
use solana_keypair::Keypair;
use solana_signer::Signer as _;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn address_from_keypair<P: AsRef<Path>>(filepath: P) -> Result<(), Box<dyn core::error::Error>> {
    // Read the keypair file
    let json_content = fs::read_to_string(filepath)?;
    let bytes: Vec<u8> = serde_json::from_str(&json_content)?;

    // Create keypair from bytes
    let keypair: Keypair = bytes.as_slice().try_into()?;
    let pubkey_bytes = keypair.pubkey().to_bytes();

    println!("Public Key: {}", keypair.pubkey());
    println!("\nPublic Key (hex): {}", hex::encode(pubkey_bytes));
    println!("\nAssembly constants (little endian):");

    // Split into 4 sections for assembly constants
    // First section: bytes 0-3 (32-bit)
    let section0 = u32::from_le_bytes([
        pubkey_bytes[0],
        pubkey_bytes[1],
        pubkey_bytes[2],
        pubkey_bytes[3],
    ]);

    // Second section: bytes 8-16 (64-bit)
    let section1 = u64::from_le_bytes([
        pubkey_bytes[8],
        pubkey_bytes[9],
        pubkey_bytes[10],
        pubkey_bytes[11],
        pubkey_bytes[12],
        pubkey_bytes[13],
        pubkey_bytes[14],
        pubkey_bytes[15],
    ]);

    // Third section: bytes 16-24 (64-bit)
    let section2 = u64::from_le_bytes([
        pubkey_bytes[16],
        pubkey_bytes[17],
        pubkey_bytes[18],
        pubkey_bytes[19],
        pubkey_bytes[20],
        pubkey_bytes[21],
        pubkey_bytes[22],
        pubkey_bytes[23],
    ]);

    // Fourth section: bytes 24-32 (64-bit)
    let section3 = u64::from_le_bytes([
        pubkey_bytes[24],
        pubkey_bytes[25],
        pubkey_bytes[26],
        pubkey_bytes[27],
        pubkey_bytes[28],
        pubkey_bytes[29],
        pubkey_bytes[30],
        pubkey_bytes[31],
    ]);

    println!(".equ EXPECTED_ADMIN_KEY_0, 0x{section0:08x}");
    println!(".equ EXPECTED_ADMIN_KEY_1, 0x{section1:016x}");
    println!(".equ EXPECTED_ADMIN_KEY_2, 0x{section2:016x}");
    println!(".equ EXPECTED_ADMIN_KEY_3, 0x{section3:016x}");
    Ok(())
}

fn grind_keys(count: usize) {
    println!("Doppler Keygen - Grinding for keys where first 8 bytes end in 4 zero bytes...");
    println!("Pattern: ???[<0x80]00000000 (byte 3 < 0x80, bytes 4-7 must be zero)");
    println!("Target: {count} key(s)\n");

    let num_threads = thread::available_parallelism()
        .expect("Failed to get available parallelism")
        .get();
    println!("Using {num_threads} threads");

    let keys_found = Arc::new(AtomicUsize::new(0));
    let attempts = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    // Start progress reporting thread
    let attempts_clone = Arc::clone(&attempts);
    let keys_found_clone = Arc::clone(&keys_found);
    thread::spawn(move || {
        let mut last_attempts = 0u64;
        let mut last_time = Instant::now();

        loop {
            thread::sleep(Duration::from_secs(5));
            let current_keys = keys_found_clone.load(Ordering::Relaxed);
            if current_keys >= count {
                break;
            }

            let current_attempts = attempts_clone.load(Ordering::Relaxed);
            let current_time = Instant::now();
            let elapsed = current_time.duration_since(last_time).as_secs_f64();
            let rate = ((current_attempts - last_attempts) as f64) / elapsed;

            println!(
                "Progress: {current_attempts} attempts | {rate:.0} keys/sec | Found: {current_keys}/{count}"
            );

            last_attempts = current_attempts;
            last_time = current_time;
        }
    });

    // 7ab7b9534d75fadb0e360ec50e33dbd01c5a4a28df115be12d65f0e5044d165eac2a276200000000e2db41114b745d8c7e6970a5f8e2f5d7b234af9bd14299e0

    // Start worker threads
    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let keys_found = Arc::clone(&keys_found);
            let attempts = Arc::clone(&attempts);

            thread::spawn(move || {
                let mut local_attempts = 0u64;

                loop {
                    // Check if we've found enough keys
                    if keys_found.load(Ordering::Relaxed) >= count {
                        break;
                    }

                    let keypair = Keypair::new();
                    let pubkey_bytes = keypair.pubkey().to_bytes();

                    // Check if byte 3 < 0x80 and bytes 4-7 are all zero
                    if pubkey_bytes[3] < 0x80
                        && pubkey_bytes[4] == 0
                        && pubkey_bytes[5] == 0
                        && pubkey_bytes[6] == 0
                        && pubkey_bytes[7] == 0
                    {
                        // Found a match!
                        let key_number = keys_found.fetch_add(1, Ordering::Relaxed) + 1;

                        // Check again if we haven't exceeded count
                        if key_number > count {
                            break;
                        }

                        println!("\n✅ FOUND MATCHING KEYPAIR #{key_number}/{count}");
                        println!("Thread: {thread_id}");
                        println!("Public Key: {}", hex::encode(keypair.pubkey().to_bytes()));
                        println!("Public Key (base58): {}", keypair.pubkey());

                        // Display the first 8 bytes in hex with byte 3 highlighted
                        print!("First 8 bytes (hex): ");
                        for (i, &byte) in pubkey_bytes.iter().take(8).enumerate() {
                            if i == 3 {
                                print!("[{byte:02x}]");
                            } else {
                                print!("{byte:02x}");
                            }
                            if i == 3 {
                                print!(" | ");
                            } else if i < 7 {
                                print!(" ");
                            }
                        }
                        println!(" (byte 3 = 0x{:02x} < 0x80 ✓)\n", pubkey_bytes[3]);

                        // Save keypair to file
                        let keypair_json = format!(
                            "[{}]",
                            keypair
                                .to_bytes()
                                .iter()
                                .map(std::string::ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(",")
                        );

                        let filename = format!("{}.json", keypair.pubkey());
                        fs::write(&filename, keypair_json).expect("Failed to write keypair file");
                        println!("Keypair saved to: {filename}");

                        // Continue looking for more keys if needed
                        if key_number >= count {
                            break;
                        }
                    }

                    local_attempts += 1;

                    // Update global counter periodically
                    if local_attempts % 10_000 == 0 {
                        attempts.fetch_add(10_000, Ordering::Relaxed);
                    }
                }

                // Add any remaining attempts
                attempts.fetch_add(local_attempts % 10_000, Ordering::Relaxed);
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let elapsed = start.elapsed();
    let total_attempts = attempts.load(Ordering::Relaxed);
    let final_keys = keys_found.load(Ordering::Relaxed);

    println!("\n------- Summary -------");
    println!("Keys found: {final_keys}/{count}");
    println!("Total attempts: {total_attempts}");
    println!("Time elapsed: {:.2} seconds", elapsed.as_secs_f64());
    println!(
        "Average rate: {:.0} keys/sec",
        total_attempts as f64 / elapsed.as_secs_f64()
    );
}

fn print_usage() {
    println!("Doppler Keygen - Solana vanity key generator\n");
    println!("Usage:");
    println!("  doppler-keygen grind [count]    - Grind for vanity keys (default: 1 key)");
    println!("  doppler-keygen address <file>   - Convert keypair to assembly constants");
    println!("\nExamples:");
    println!("  doppler-keygen grind            - Find 1 vanity key");
    println!("  doppler-keygen grind 5          - Find 5 vanity keys");
    println!("  doppler-keygen address key.json - Convert key.json to assembly format");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "grind" => {
            let count = if args.len() > 2 {
                args[2].parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("Error: Invalid count number");
                    process::exit(1);
                })
            } else {
                1
            };

            if count == 0 {
                eprintln!("Error: Count must be at least 1");
                process::exit(1);
            }

            grind_keys(count);
        }
        "address" => {
            if args.len() != 3 {
                eprintln!("Error: address command requires a keypair file");
                eprintln!("Usage: {} address <keypair.json>", args[0]);
                process::exit(1);
            }

            if let Err(e) = address_from_keypair(&args[2]) {
                eprintln!("Error converting keypair: {e}");
                process::exit(1);
            };
        }
        _ => {
            eprintln!("Error: Unknown command '{}'\n", args[1]);
            print_usage();
            process::exit(1);
        }
    }
}
