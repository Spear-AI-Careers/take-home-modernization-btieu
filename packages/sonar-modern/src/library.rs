//! Rust port of the legacy SONAR `library.c` / `library.h`.
//!
//! In this file, you will find the Rust translation/port of the library.c and library.h files.
//! I tried my best to keep the same structure, logic, and even variable/constant names as the original
//! C code. If I deviate from the original C code, I will explain why in the comments.
//!
//! The C code has its configu in mutable globals, but in Rust we use static mut
//! so every access is wrapped in 'unsafe'.

use rand::Rng;

/// Maximum number of signals in a ping. Fixed size
pub const MAX_SIGNALS: usize = 1000;
/// Maximum number of contacts. Fixed size: 20
pub const MAX_CONTACTS: usize = 20;

// Contact type definitions, each type is an integer to match the C code.
pub const CONTACT_TYPE_UNKNOWN: i32 = 0;
pub const CONTACT_TYPE_ROCK: i32 = 1;
pub const CONTACT_TYPE_VESSEL: i32 = 2;
pub const CONTACT_TYPE_MARINE_LIFE: i32 = 3;

/// A detected sonar contact.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Contact {
    /// Distance of object in meters.
    pub distance: f32,
    /// Estimated size of object in meters
    pub size: f32,
    /// Degres from 0 to <360, where 0 is straight ahead and 90 is to the right.
    pub bearing: f32,
    /// Decimal confidence in the detection, 0.0 to 1.0
    pub confidence: f32,
    /// 0=unknown, 1=rock, 2=vessel, 3=marine life (`type` is a Rust keyword)
    pub contact_type: i32,
}

// Global configuration variables
pub static mut g_noise_threshold: f32 = 15.0;
pub static mut g_min_contact_size: i32 = 3;
pub static mut g_distance_factor: f32 = 10.0;
pub static mut g_last_max_confidence: f32 = 0.0;

// Temporary buffer for filtered data - global to avoid stack overflow
static mut FILTERED_DATA: [f32; MAX_SIGNALS] = [0.0; MAX_SIGNALS];

/// Detect and classify contacts in `ping_data`. Detected contacts are written into 
/// `contacts` and the number written is returned. 
/// Arguments are ping_data, bearing_angles, contacts. The C code also has 
/// data_points and max_contacts, but those are replaced by the slice lengths in Rust.
#[allow(clippy::needless_range_loop)]
pub fn analyze(ping_data: &[f32], bearing_angles: &[f32], contacts: &mut [Contact]) -> usize {
    let data_points = ping_data.len().min(MAX_SIGNALS);
    let max_contacts = contacts.len();
    let mut contact_count = 0;
    // As g_min_contact_size is i32 and min_contact_size is used as usize, we need to cast it here.
    let min_contact_size = unsafe { g_min_contact_size } as usize;
    // So we don't us mutable static references, we can use a local variable here.
    let distance_factor = unsafe { g_distance_factor };
    let noise_threshold = unsafe { g_noise_threshold };

    // Simple noise filtering with 3-point moving average
    for i in 0..data_points {
        let value = if i > 0 && i < data_points - 1 {
            (ping_data[i - 1] + ping_data[i] + ping_data[i + 1]) / 3.0
        } else {
            ping_data[i]
        };
        unsafe { FILTERED_DATA[i] = value };
    }
    // safety: FILTERED_DATA is a mutable static variable, but we only read it here after the filtering is complete.
    let filtered_data = unsafe { &*std::ptr::addr_of!(FILTERED_DATA) };

    // Reset confidence tracker
    unsafe { g_last_max_confidence = 0.0 };

    let mut i = 0;
    while i < data_points {
        if filtered_data[i] > noise_threshold {
            let start_idx = i;

            // Find the end of this contact signature
            while i < data_points && filtered_data[i] > noise_threshold {
                i += 1;
            }

            let end_idx = i - 1;
            let signature_width = end_idx - start_idx + 1;

            // Only process if contact signature is wide enough and we have space
            if signature_width >= min_contact_size && contact_count < max_contacts {
                let mut peak_strength: f32 = 0.0;
                let mut peak_idx = 0;

                // Find the peak signal
                for j in start_idx..=end_idx {
                    if filtered_data[j] > peak_strength {
                        peak_strength = filtered_data[j];
                        peak_idx = j;
                    }
                }

                // Calculate estimated distance based on signal strength
                let mut distance = 100.0 - peak_strength; // Stronger signals are closer
                distance *= distance_factor; // Scale to reasonable range in meters
                
                // Populate the contact data.
                let contact = &mut contacts[contact_count];
                contact.distance = distance;
                contact.size = signature_width as f32 * 2.0; // Crude size estimation
                contact.bearing = bearing_angles[peak_idx];
                contact.confidence = peak_strength / 100.0;

                // Track highest confidence
                unsafe {
                    if contact.confidence > g_last_max_confidence {
                        g_last_max_confidence = contact.confidence;
                    }
                }

                contact.contact_type = if peak_strength > 80.0 {
                    CONTACT_TYPE_ROCK // Likely a rock or solid structure
                } else if signature_width > 10 {
                    CONTACT_TYPE_VESSEL // Likely a vessel
                } else if peak_strength < 40.0 && signature_width < 5 {
                    CONTACT_TYPE_MARINE_LIFE // Possibly marine life
                } else {
                    CONTACT_TYPE_UNKNOWN
                };

                contact_count += 1;
            }
        }
        i += 1;
    }

    contact_count
}

/// Fill `output_data` and `output_angles` with a synthetic ping containing
/// background noise and up to `num_contacts` contacts.
/// Arguments are output_data, output_angles, num_contacts. The C code also has 
/// data_points, but that is replaced by the slice length in Rust.
pub fn generate_data(output_data: &mut [f32], output_angles: &mut [f32], num_contacts: usize) {
    let data_points = output_data.len();
    // thread-local RNG 
    let mut rng = rand::thread_rng();

    // Initialize with background noise
    for i in 0..data_points {
        // Random noise between 5-10
        output_data[i] = 5.0 + rng.gen_range(0..6) as f32;

        // Evenly distributed bearing angles
        output_angles[i] = (360.0 * i as f32) / data_points as f32;
    }

    let num_contacts = num_contacts.min(10); // Limit to reasonable number

    for _ in 0..num_contacts {
        let contact_center = rng.gen_range(0..data_points) as i64;
        let contact_width: i64 = 3 + rng.gen_range(0..8); // Width between 3-10
        let contact_strength = 40.0 + rng.gen_range(0..60) as f32; // Strength between 40-99

        // Create a bell curve of signal strength
        for j in -(contact_width / 2)..=(contact_width / 2) {
            // Wrap around the ends of the array (C: `%` then `+= data_points` if negative)
            let idx = (contact_center + j).rem_euclid(data_points as i64) as usize;

            let falloff = 1.0 - (j * j) as f32 / (contact_width * contact_width) as f32;
            output_data[idx] = contact_strength * falloff;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bearings(n: usize) -> Vec<f32> {
        (0..n).map(|i| (360.0 * i as f32) / n as f32).collect()
    }

    #[test]
    fn pure_noise_yields_no_contacts() {
        let ping = [7.0_f32; 360];
        let mut contacts = [Contact::default(); MAX_CONTACTS];
        let n = analyze(&ping, &bearings(360), &mut contacts);
        assert_eq!(n, 0);
        assert_eq!(unsafe { G_LAST_MAX_CONFIDENCE }, 0.0);
    }

    #[test]
    fn detects_plateau_and_fills_fields() {
        let mut ping = [5.0_f32; 360];
        for s in &mut ping[100..110] {
            *s = 90.0;
        }
        let mut contacts = [Contact::default(); MAX_CONTACTS];
        let n = analyze(&ping, &bearings(360), &mut contacts);
        assert_eq!(n, 1);
        let c = contacts[0];
        assert_eq!(c.confidence, 0.9);
        assert_eq!(c.distance, (100.0 - 90.0) * 10.0);
        assert_eq!(c.size, 12.0 * 2.0); // smoothed run is samples 99..=110
        assert_eq!(c.bearing, 101.0); // first sample that averages to exactly 90
        assert_eq!(c.contact_type, CONTACT_TYPE_ROCK);
    }

    #[test]
    fn classification_branches() {
        // (strength, width) -> type, same order as the C if/else chain
        let cases = [
            (90.0, 4, CONTACT_TYPE_ROCK),
            (60.0, 12, CONTACT_TYPE_VESSEL),
            (30.0, 3, CONTACT_TYPE_MARINE_LIFE),
            (60.0, 6, CONTACT_TYPE_UNKNOWN),
        ];
        for (strength, width, expected) in cases {
            let mut ping = [0.0_f32; 100];
            for s in &mut ping[40..40 + width] {
                *s = strength;
            }
            let mut contacts = [Contact::default(); MAX_CONTACTS];
            let n = analyze(&ping, &bearings(100), &mut contacts);
            assert_eq!(n, 1, "strength {strength} width {width}");
            assert_eq!(
                contacts[0].contact_type, expected,
                "strength {strength} width {width}"
            );
        }
    }

    #[test]
    fn narrow_signature_is_ignored() {
        let mut ping = [0.0_f32; 100];
        ping[50] = 40.0; // smooths to 13.3 at 49..=51, all below threshold
        let mut contacts = [Contact::default(); MAX_CONTACTS];
        assert_eq!(analyze(&ping, &bearings(100), &mut contacts), 0);
    }

    #[test]
    fn never_writes_past_contacts_slice() {
        let mut ping = [0.0_f32; 100];
        for start in (0..100).step_by(10) {
            for s in &mut ping[start..start + 5] {
                *s = 50.0;
            }
        }
        let mut contacts = [Contact::default(); 3];
        assert_eq!(analyze(&ping, &bearings(100), &mut contacts), 3);
    }

    #[test]
    fn single_sample_does_not_panic() {
        let mut contacts = [Contact::default(); MAX_CONTACTS];
        assert_eq!(analyze(&[90.0], &[0.0], &mut contacts), 0);
    }

    #[test]
    fn generated_data_is_in_range() {
        let mut data = [0.0_f32; 360];
        let mut angles = [0.0_f32; 360];
        generate_data(&mut data, &mut angles, 5);
        assert!(data.iter().all(|&v| (5.0..=100.0).contains(&v)));
        assert!(angles.iter().all(|&a| (0.0..360.0).contains(&a)));
        assert_eq!(angles[90], 90.0);
        assert!(angles.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn generated_noise_only_is_integer_5_to_10() {
        let mut data = [0.0_f32; 500];
        let mut angles = [0.0_f32; 500];
        generate_data(&mut data, &mut angles, 0);
        assert!(
            data.iter()
                .all(|&v| (5.0..=10.0).contains(&v) && v.fract() == 0.0)
        );
    }

    #[test]
    fn generated_contacts_are_detectable() {
        let mut data = [0.0_f32; 360];
        let mut angles = [0.0_f32; 360];
        generate_data(&mut data, &mut angles, 5);
        let mut contacts = [Contact::default(); MAX_CONTACTS];
        let n = analyze(&data, &angles, &mut contacts);
        // A contact straddling sample 0 wraps around and is counted twice
        // (legacy behaviour), hence <= 6.
        assert!((1..=6).contains(&n), "detected {n}");
    }
}