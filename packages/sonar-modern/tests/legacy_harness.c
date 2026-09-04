/*
 * Test harness around the legacy analyze() for differential testing.
 *
 * stdin:  noise_threshold min_contact_size distance_factor max_contacts n
 *         then n lines of "<ping bits> <bearing bits>" (float bit patterns as
 *         unsigned decimal), so values round-trip exactly.
 * stdout: "<count> <max_confidence bits>" then one line per contact:
 *         "<distance bits> <size bits> <bearing bits> <confidence bits> <type>"
 *
 * Bit patterns are used instead of %f so the comparison is exact.
 */
#include <stdio.h>
#include <string.h>
#include "../../sonar-legacy/library.h"

static float from_bits(unsigned int u) { float f; memcpy(&f, &u, sizeof f); return f; }
static unsigned int to_bits(float f) { unsigned int u; memcpy(&u, &f, sizeof u); return u; }

int main(void)
{
    static float ping[MAX_SIGNALS], angles[MAX_SIGNALS];
    static Contact contacts[MAX_CONTACTS];
    int n, max_contacts, i, count;

    if (scanf("%d %d %f %d %d", &g_noise_threshold, &g_min_contact_size,
              &g_distance_factor, &max_contacts, &n) != 5)
        return 2;
    if (n < 1 || n > MAX_SIGNALS || max_contacts > MAX_CONTACTS)
        return 3;

    for (i = 0; i < n; i++) {
        unsigned int p, a;
        if (scanf("%u %u", &p, &a) != 2)
            return 4;
        ping[i] = from_bits(p);
        angles[i] = from_bits(a);
    }

    count = analyze(ping, n, angles, contacts, max_contacts);

    printf("%d %u\n", count, to_bits(g_last_max_confidence));
    for (i = 0; i < count; i++) {
        printf("%u %u %u %u %d\n",
               to_bits(contacts[i].distance), to_bits(contacts[i].size),
               to_bits(contacts[i].bearing), to_bits(contacts[i].confidence),
               contacts[i].type);
    }
    return 0;
}
