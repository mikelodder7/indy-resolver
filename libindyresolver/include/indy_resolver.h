#ifndef __indy__resolver__included__
#define __indy__resolver__included__

#include <stdint.h>

typedef enum {
    SUCCESS = 0,
    CONTEXT_CREATE_FAIL = 1,
    INVALID_DID = 2
} indy_res_status_t;

struct ByteBuffer {
    int64_t len;
    uint8_t *data;
};

struct ExternError {
    indy_res_status_t code;
    char* message;
};

#ifdef __cplusplus
extern "C" {
#endif

extern int32_t indy_res_txn_sign(const char* const in_txn,
                                 const void* const signer_key,
                                 const char* out_txn,
                                 const struct ExternError* err);

extern int32_t indy_res_txn_send(int32_t pool_handle,
                                         const char* const txn,
                                         const struct ExternError* err);

extern void indy_res_bytebuffer_free(struct ByteBuffer buffer);
extern void indy_res_free_string(char* input);

//Put other header file includes here
#include "indy_resolver_context.h"

#ifdef __cplusplus
}
#endif

#endif
