#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <math.h>

// Global configuration variables
int g_noise_threshold = 15;
int g_min_contact_size = 3;
float g_distance_factor = 10.0;
float g_last_max_confidence = 0.0;

#define MAX_SIGNALS 1000
#define MAX_CONTACTS 20

typedef struct
{
    float distance;
    float size;
    float bearing;
    float confidence;
    int type;
} Contact;

// Temporary buffer for filtered data - global to avoid stack overflow
float filtered_data[MAX_SIGNALS];

int analyze(float *ping_data, int data_points, float *bearing_angles,
            Contact *contacts, int max_contacts)
{
    int i, j, contact_count = 0;

    // Simple noise filtering with 3-point moving average
    for (i = 0; i < data_points; i++)
    {
        if (i > 0 && i < data_points - 1)
        {
            filtered_data[i] = (ping_data[i - 1] + ping_data[i] + ping_data[i + 1]) / 3.0;
        }
        else
        {
            filtered_data[i] = ping_data[i];
        }
    }

    // Reset confidence tracker
    g_last_max_confidence = 0.0;

    for (i = 0; i < data_points; i++)
    {
        if (filtered_data[i] > g_noise_threshold)
        {
            int start_idx = i;

            // Find the end of this contact signature
            while (i < data_points && filtered_data[i] > g_noise_threshold)
            {
                i++;
            }

            int end_idx = i - 1;
            int signature_width = end_idx - start_idx + 1;

            // Only process if contact signature is wide enough and we have space
            if (signature_width >= g_min_contact_size && contact_count < max_contacts)
            {
                float peak_strength = 0;
                int peak_idx = 0;
                float total_strength = 0;

                // Find the peak signal and total strength
                for (j = start_idx; j <= end_idx; j++)
                {
                    total_strength += filtered_data[j];

                    if (filtered_data[j] > peak_strength)
                    {
                        peak_strength = filtered_data[j];
                        peak_idx = j;
                    }
                }

                // Calculate estimated distance based on signal strength
                float distance = 100.0 - peak_strength;  // Stronger signals are closer
                distance = distance * g_distance_factor; // Scale to reasonable range in meters

                // Populate the contact data
                contacts[contact_count].distance = distance;
                contacts[contact_count].size = signature_width * 2.0; // Crude size estimation
                contacts[contact_count].bearing = bearing_angles[peak_idx];
                contacts[contact_count].confidence = peak_strength / 100.0;

                // Track highest confidence
                if (contacts[contact_count].confidence > g_last_max_confidence)
                {
                    g_last_max_confidence = contacts[contact_count].confidence;
                }

                if (peak_strength > 80.0)
                {
                    contacts[contact_count].type = 1; // Likely a rock or solid structure
                }
                else if (signature_width > 10)
                {
                    contacts[contact_count].type = 2; // Likely a vessel
                }
                else if (peak_strength < 40.0 && signature_width < 5)
                {
                    contacts[contact_count].type = 3; // Possibly marine life
                }
                else
                {
                    contacts[contact_count].type = 0; // Unknown
                }

                contact_count++;
            }
        }
    }

    return contact_count;
}

int generate_data(float *output_data, float *output_angles,
                  int data_points, int num_contacts)
{
    int i, j;

    // Initialize with background noise
    for (i = 0; i < data_points; i++)
    {
        // Random noise between 5-10
        output_data[i] = 5.0 + (rand() % 6);

        // Evenly distributed bearing angles
        output_angles[i] = (360.0 * i) / data_points;
    }

    if (num_contacts > 10)
    {
        num_contacts = 10; // Limit to reasonable number
    }

    for (i = 0; i < num_contacts; i++)
    {
        int contact_center = rand() % data_points;
        int contact_width = 3 + (rand() % 8);          // Width between 3-10
        float contact_strength = 40.0 + (rand() % 60); // Strength between 40-99

        // Create a bell curve of signal strength
        for (j = -contact_width / 2; j <= contact_width / 2; j++)
        {
            int idx = (contact_center + j) % data_points;
            if (idx < 0)
                idx += data_points;

            float falloff = 1.0 - (j * j) / (float)(contact_width * contact_width);
            output_data[idx] = contact_strength * falloff;
        }
    }

    return 0;
}
