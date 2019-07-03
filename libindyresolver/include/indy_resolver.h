#ifndef __indy__resolver__included__
#define __indy__resolver__included__

#include <stdint.h>

typedef enum {
    SUCCESS = 0
} indy_resolver_status_t;

struct ByteBuffer {
    int64_t len;
    uint8_t *data;
};

struct ExternError {
    indy_resolver_status_t code;
    char* message;
};

#ifdef __cplusplus
extern "C" {
#endif
}
extern void indy_resolver_bytebuffer_free(struct ByteBuffer buffer);

//Put other header file includes here

#ifdef __cplusplus
}
#endif

#endif
