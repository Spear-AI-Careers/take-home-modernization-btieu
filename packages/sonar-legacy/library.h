// Maximum number of signals and contacts
#define MAX_SIGNALS 1000
#define MAX_CONTACTS 20

// Contact type definitions
#define CONTACT_TYPE_UNKNOWN 0
#define CONTACT_TYPE_ROCK 1
#define CONTACT_TYPE_VESSEL 2
#define CONTACT_TYPE_MARINE_LIFE 3

// Structure to represent a sonar contact
typedef struct
{
  float distance;   // meters
  float size;       // estimated size in meters
  float bearing;    // degrees (0-359)
  float confidence; // 0.0 to 1.0
  int type;         // 0=unknown, 1=rock, 2=vessel, 3=marine life
} Contact;

// Global configuration variables - declared as extern so they can be
// accessed and modified by code using this header
extern int g_noise_threshold;
extern int g_min_contact_size;
extern float g_distance_factor;
extern float g_last_max_confidence;

int analyze(float *ping_data, int data_points, float *bearing_angles,
            Contact *contacts, int max_contacts);

int generate_data(float *output_data, float *output_angles,
                  int data_points, int num_contacts);
