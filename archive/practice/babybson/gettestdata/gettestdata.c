#include <stdio.h>
#include <stdint.h>
#include <bson/bson.h>
static void hexdump (const uint8_t *data, size_t len) {
    for (size_t i = 0; i < len; i++) {
        printf ("0x%02x, ", data[i]);
    }
    printf ("\n");
}

int main () {
    bson_t *b = BCON_NEW ("x", "{", "y", BCON_INT32(1), "}");    
    char *str = bson_as_canonical_extended_json (b, NULL);
    printf ("JSON: %s\n", str);
    printf ("Hex : ");
    hexdump (bson_get_data(b), b->len);
    bson_free (str);
    bson_destroy (b);
}