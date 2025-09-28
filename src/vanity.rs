use core::sync::atomic::{AtomicU64, Ordering};
use solana_keypair::Keypair;
use solana_signer::Signer as _;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum VanityPattern {
    /// Match pattern at the beginning of the key
    Prefix(String),
    /// Match pattern at the end of the key
    Suffix(String),
    /// Match pattern anywhere in the key
    Contains(String),
    /// Match pattern at a specific byte position
    AtPosition(String, usize),
}

impl VanityPattern {
    pub fn matches(&self, key_bytes: &[u8]) -> bool {
        match self {
            VanityPattern::Prefix(pattern) => {
                let pattern_bytes = pattern.as_bytes();
                key_bytes.len() >= pattern_bytes.len() &&
                key_bytes[..pattern_bytes.len()] == *pattern_bytes
            }
            VanityPattern::Suffix(pattern) => {
                let pattern_bytes = pattern.as_bytes();
                key_bytes.len() >= pattern_bytes.len() &&
                key_bytes[key_bytes.len()-pattern_bytes.len()..] == *pattern_bytes
            }
            VanityPattern::Contains(pattern) => {
                let pattern_bytes = pattern.as_bytes();
                key_bytes.windows(pattern_bytes.len()).any(|window| window == pattern_bytes)
            }
            VanityPattern::AtPosition(pattern, position) => {
                let pattern_bytes = pattern.as_bytes();
                key_bytes.len() >= position + pattern_bytes.len() &&
                key_bytes[*position..*position + pattern_bytes.len()] == *pattern_bytes
            }
        }
    }
}

pub fn parse_vanity_pattern(pattern_str: &str, position: Option<usize>) -> Result<VanityPattern, String> {
    if let Some(pos) = position {
        Ok(VanityPattern::AtPosition(pattern_str.to_string(), pos))
    } else if pattern_str.starts_with("prefix:") {
        Ok(VanityPattern::Prefix(pattern_str[7..].to_string()))
    } else if pattern_str.starts_with("suffix:") {
        Ok(VanityPattern::Suffix(pattern_str[7..].to_string()))
    } else if pattern_str.starts_with("contains:") {
        Ok(VanityPattern::Contains(pattern_str[9..].to_string()))
    } else if pattern_str.starts_with("at:") {
        let parts: Vec<&str> = pattern_str[3..].split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid at: format. Use at:position:pattern".to_string());
        }
        let pos: usize = parts[0].parse().map_err(|_| "Invalid position number".to_string())?;
        Ok(VanityPattern::AtPosition(parts[1].to_string(), pos))
    } else {
        // Default to prefix matching
        Ok(VanityPattern::Prefix(pattern_str.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct BatchPattern {
    pub pattern: VanityPattern,
    pub pattern_str: String,
    pub target_count: usize,
    pub current_count: usize,
}

impl BatchPattern {
    pub fn new(pattern_str: &str, position: Option<usize>, target_count: usize) -> Result<Self, String> {
        let pattern = parse_vanity_pattern(pattern_str, position)?;
        Ok(BatchPattern {
            pattern,
            pattern_str: pattern_str.to_string(),
            target_count,
            current_count: 0,
        })
    }

    pub fn is_complete(&self) -> bool {
        self.current_count >= self.target_count
    }

    pub fn increment(&mut self) {
        self.current_count += 1;
    }
}

pub fn vanity_keys(pattern_str: &str, count: usize, position: Option<usize>) {
    let batch_pattern = BatchPattern::new(pattern_str, position, count).unwrap();
    let mut patterns = vec![batch_pattern];
    vanity_keys_batch(&mut patterns);
}

pub fn vanity_keys_batch(batch_patterns: &mut [BatchPattern]) {
    let patterns = batch_patterns.to_vec();
    let pattern_count = patterns.len();
    if pattern_count == 0 { return; }

    let total_targets: usize = patterns.iter().map(|p| p.target_count).sum();
    let total_found: usize = patterns.iter().map(|p| p.current_count).sum();

    println!("Batch processing {} patterns ({}/{} keys found)", pattern_count, total_found, total_targets);
    for pattern in patterns.iter() {
        println!("  {}: {}/{}", pattern.pattern_str, pattern.current_count, pattern.target_count);
    }

    let num_threads = thread::available_parallelism().unwrap().get();
    let keys_found = Arc::new(AtomicUsize::new(0));
    let attempts = Arc::new(AtomicU64::new(0));
    let start = Instant::now();
    let patterns_arc = Arc::new(std::sync::Mutex::new(patterns));

    // Progress thread
    let attempts_clone = Arc::clone(&attempts);
    let keys_found_clone = Arc::clone(&keys_found);
    let patterns_clone = Arc::clone(&patterns_arc);
    thread::spawn(move || {
        let mut last_attempts = 0u64;
        loop {
            thread::sleep(Duration::from_secs(3));
            let current_keys = keys_found_clone.load(Ordering::Relaxed);
            let current_attempts = attempts_clone.load(Ordering::Relaxed);
            let rate = (current_attempts - last_attempts) as f64 / 3.0;

            println!("Progress: {}/{} keys | {:.0} keys/sec", current_keys, total_targets, rate);
            last_attempts = current_attempts;

            if patterns_clone.lock().unwrap().iter().all(|p| p.is_complete()) { break; }
        }
    });

    // Worker threads
    let handles: Vec<_> = (0..num_threads).map(|_| {
        let keys_found = Arc::clone(&keys_found);
        let attempts = Arc::clone(&attempts);
        let patterns = Arc::clone(&patterns_arc);
        thread::spawn(move || {
            let mut local_attempts = 0u64;
            loop {
                if patterns.lock().unwrap().iter().all(|p| p.is_complete()) { break; }

                let keypair = Keypair::new();
                let pubkey_bytes = keypair.pubkey().to_bytes();

                let mut found_match = false;
                {
                    let mut patterns = patterns.lock().unwrap();
                    for pattern in patterns.iter_mut() {
                        if !pattern.is_complete() && pattern.pattern.matches(&pubkey_bytes) {
                            pattern.increment();
                            found_match = true;
                            println!("Found key for '{}': {}", pattern.pattern_str, keypair.pubkey());
                            break;
                        }
                    }
                }

                if found_match { keys_found.fetch_add(1, Ordering::Relaxed); }
                local_attempts += 1;
                if local_attempts % 5_000 == 0 { attempts.fetch_add(5_000, Ordering::Relaxed); }
            }
            attempts.fetch_add(local_attempts % 5_000, Ordering::Relaxed);
        })
    }).collect();

    for handle in handles { handle.join().unwrap(); }

    let elapsed = start.elapsed();
    let total_attempts = attempts.load(Ordering::Relaxed);
    let final_keys = keys_found.load(Ordering::Relaxed);

    println!("Completed: {}/{} keys in {:.1}s ({:.0} keys/sec)",
        final_keys, total_targets, elapsed.as_secs_f64(), total_attempts as f64 / elapsed.as_secs_f64());

    // Update original array
    let final_patterns = patterns_arc.lock().unwrap();
    for (i, pattern) in final_patterns.iter().enumerate() {
        if i < batch_patterns.len() { batch_patterns[i] = pattern.clone(); }
    }
}
