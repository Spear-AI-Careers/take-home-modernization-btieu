//! Differential test: run the *original C* `analyze()` and the Rust port on
//! identical inputs and require bit-identical output.
//!
//! Needs a C compiler (`cc`) and the legacy sources in `../sonar-legacy`, so
//! it is `#[ignore]`d by default. Run it with:
//!
//! ```sh
//! cargo test --test legacy_differential -- --ignored
//! ```

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use sonar_modern::{Contact, G_LAST_MAX_CONFIDENCE, MAX_CONTACTS, MAX_SIGNALS, analyze};

/// Compile the C harness exactly once per test process.
fn build_harness() -> &'static Path {
    static HARNESS: OnceLock<PathBuf> = OnceLock::new();
    HARNESS.get_or_init(|| {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("legacy_harness");
        let status = Command::new("cc")
            .args(["-O2", "-o"])
            .arg(&out)
            .arg(manifest.join("tests/legacy_harness.c"))
            .arg(manifest.join("../sonar-legacy/library.c"))
            .arg("-lm")
            .status()
            .expect("failed to run `cc`; a C compiler is required for this test");
        assert!(status.success(), "legacy harness failed to compile");
        out
    })
}

/// Run the legacy C analyzer with default globals; returns (contacts, max confidence).
fn run_legacy(harness: &Path, ping: &[f32], angles: &[f32]) -> (Vec<Contact>, f32) {
    let mut input = format!("15 3 10.0 {} {}\n", MAX_CONTACTS, ping.len());
    for (p, a) in ping.iter().zip(angles) {
        input.push_str(&format!("{} {}\n", p.to_bits(), a.to_bits()));
    }

    let mut child = Command::new(harness)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "harness exited with {}",
        output.status
    );

    let text = String::from_utf8(output.stdout).unwrap();
    let mut lines = text.lines();
    let mut header = lines.next().unwrap().split_whitespace();
    let count: usize = header.next().unwrap().parse().unwrap();
    let max_conf = f32::from_bits(header.next().unwrap().parse().unwrap());

    let contacts = lines
        .take(count)
        .map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let bits = |s: &str| f32::from_bits(s.parse::<u32>().unwrap());
            Contact {
                distance: bits(f[0]),
                size: bits(f[1]),
                bearing: bits(f[2]),
                confidence: bits(f[3]),
                contact_type: f[4].parse().unwrap(),
            }
        })
        .collect();

    (contacts, max_conf)
}

fn assert_same(harness: &Path, ping: &[f32], angles: &[f32]) {
    let (c_contacts, c_max) = run_legacy(harness, ping, angles);

    let mut contacts = [Contact::default(); MAX_CONTACTS];
    let n = analyze(ping, angles, &mut contacts);
    // SAFETY: the test reads the global right after the call that set it.
    let rust_max = unsafe { G_LAST_MAX_CONFIDENCE };

    assert_eq!(
        &contacts[..n],
        &c_contacts[..],
        "contacts differ from legacy C"
    );
    assert_eq!(
        rust_max.to_bits(),
        c_max.to_bits(),
        "max confidence differs from legacy C"
    );
}

fn bearings(n: usize) -> Vec<f32> {
    (0..n).map(|i| (360.0 * i as f32) / n as f32).collect()
}

#[test]
#[ignore = "requires a C compiler and ../sonar-legacy; run with --ignored"]
fn matches_legacy_on_random_pings() {
    let harness = build_harness();
    let mut rng = StdRng::seed_from_u64(2024);
    for case in 0..500 {
        let n = rng.gen_range(1..=MAX_SIGNALS);
        // Mix of pure-random and "noise floor with plateaus" pings.
        let ping: Vec<f32> = if case % 2 == 0 {
            (0..n).map(|_| rng.gen_range(0.0..=100.0)).collect()
        } else {
            let mut p: Vec<f32> = (0..n).map(|_| 5.0 + rng.gen_range(0..6) as f32).collect();
            for _ in 0..rng.gen_range(0..12) {
                let start = rng.gen_range(0..n);
                let width = rng.gen_range(1..=15);
                let strength = rng.gen_range(20.0..=100.0);
                for s in p.iter_mut().skip(start).take(width) {
                    *s = strength;
                }
            }
            p
        };
        assert_same(harness, &ping, &bearings(n));
    }
}

#[test]
#[ignore = "requires a C compiler and ../sonar-legacy; run with --ignored"]
fn matches_legacy_on_edge_cases() {
    let harness = build_harness();
    let cases: Vec<Vec<f32>> = vec![
        vec![90.0],                                        // single sample
        vec![90.0, 90.0],                                  // two samples, both edges
        vec![15.0; 50],                                    // exactly on threshold
        vec![100.0; MAX_SIGNALS],                          // saturated, max length
        vec![0.0; MAX_SIGNALS],                            // silent, max length
        (0..360).map(|i| (i % 7) as f32 * 20.0).collect(), // many narrow runs
    ];
    for ping in cases {
        assert_same(harness, &ping, &bearings(ping.len()));
    }
}
