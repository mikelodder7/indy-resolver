#ifndef __indy__resolver__did__included__
#define __indy__resolver__did__included__

#include <stdint.h>


/*
 * Creates a nym on the indy ledger
 *
 * #param `pool_handle` - a previously opened pool handle to create for the DID
 * #param `nym` - the fully qualified did string. Examples are did:sov:12345tgfvgrfdfrtg
 * #param `document` - .
 * #return `txn_rsp` - a pointer that will be filled with the txn response.
 * #return `err` - The result of the call. Will contain an error messages that occur
 * 
 */
extern int32_t indy_res_nym_create(int32_t pool_handle,
                                        const char* const nym,
                                        const char* const document,
                                        const char* txn_rsp,
                                        const struct ExternError* err);

/*
 * Resolves a nym on the indy ledger
 *
 * #param `pool_handle` - a previously opened pool handle to look for the DID
 * #param `nym` - the fully qualified did string. Examples are did:sov:12345tgfvgrfdfrtg
 * #param `document` - a pointer that will be filled with the resolved document.
 * #return `err` - The result of the call. Will contain an error messages that occur
 */
extern int32_t indy_res_nym_read(int32_t pool_handle,
                                 const char* const nym,
                                 const char* document,
                                 const struct ExternError* err);


extern int32_t indy_res_nym_update(int32_t pool_handle,
                                   const char* const did,
                                   const char* const document,
                                   const struct ExternError* err);

extern int32_t indy_res_nym_delete(int32_t pool_handle,
                                   const char* const did,
                                   const struct ExternError* err);
#endif
