//! Rust port of the legacy `main.c`.

use sonar_modern::{
    Contact, g_last_max_confidence, MAX_CONTACTS, MAX_SIGNALS, analyze, generate_data,
};

fn contact_type_to_string(contact_type: i32) -> &'static str {
    match contact_type {
        0 => "Unknown",
        1 => "Rock/Structure",
        2 => "Vessel",
        3 => "Marine Life",
        _ => "Invalid",
    }
}

fn main() {
    let mut data = [0.0_f32; MAX_SIGNALS];
    let mut bearing_angles = [0.0_f32; MAX_SIGNALS];
    let mut contacts = [Contact::default(); MAX_CONTACTS];
    let num_data_points = 360; // One per degree

    // Generate synthetic sonar data.
    generate_data(&mut data[..num_data_points], &mut bearing_angles[..num_data_points], 5,);
    // Analyze the sonar data to detect contacts.
    let num_contacts = analyze(&data[..num_data_points], &bearing_angles[..num_data_points], &mut contacts,);

    // Print results
    println!("Detected {num_contacts} contacts:");
    println!("--------------------------------------------");
    println!("  Distance (m) | Bearing | Size (m) | Type");
    println!("--------------------------------------------");

    for (i, c) in contacts.iter().take(num_contacts).enumerate() {
        println!(
            "{:3}: {:7.1} m | {:7.1}° | {:7.1} m | {} ({:.0}%)",
            i + 1,
            c.distance,
            c.bearing,
            c.size,
            contact_type_to_string(c.contact_type),
            c.confidence * 100.0
        );
    }

    // Safety: g_last_max_confidence is a mutable static variable, but we only read it here after the analysis is complete.
    let max_confidence = unsafe { g_last_max_confidence };
    println!("\nHighest confidence detection: {:.0}%", max_confidence * 100.0);
}
