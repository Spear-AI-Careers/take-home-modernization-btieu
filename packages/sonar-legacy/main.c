#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include "library.h"

const char *contact_type_to_string(int type)
{
    switch (type)
    {
    case 0:
        return "Unknown";
    case 1:
        return "Rock/Structure";
    case 2:
        return "Vessel";
    case 3:
        return "Marine Life";
    default:
        return "Invalid";
    }
}

int main(int argc, char **argv)
{
    float data[MAX_SIGNALS];
    float bearing_angles[MAX_SIGNALS];
    Contact contacts[MAX_CONTACTS];
    int num_data_points = 360; // One per degree
    int i, num_contacts;

    srand(time(NULL));

    generate_data(data, bearing_angles, num_data_points, 5);

    num_contacts = analyze(data, num_data_points, bearing_angles,
                           contacts, MAX_CONTACTS);

    // Print results
    printf("Detected %d contacts:\n", num_contacts);
    printf("--------------------------------------------\n");
    printf("  Distance (m) | Bearing | Size (m) | Type\n");
    printf("--------------------------------------------\n");

    for (i = 0; i < num_contacts; i++)
    {
        printf("%3d: %7.1f m | %7.1f° | %7.1f m | %s (%.0f%%)\n",
               i + 1,
               contacts[i].distance,
               contacts[i].bearing,
               contacts[i].size,
               contact_type_to_string(contacts[i].type),
               contacts[i].confidence * 100.0);
    }

    printf("\nHighest confidence detection: %.0f%%\n", g_last_max_confidence * 100.0);

    return 0;
}
