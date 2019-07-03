#ifndef __indy__resolver__did__included__
#define __indy__resolver__did__included__

/*
 * Resolves a DID on the indy ledger
 *
 * #param `pool_handle` - a previously opened pool handle to look for the DID
 * #param `did` - the fully qualified did string. Examples are did:sov:12345tgfvgrfdfrtg
 * #return `document` - a pointer that will be filled with the resolved document.
 * #return `err` - The result of the call. Will contain an error messages that occur
 */
extern int32_t indy_resolver_did_read(int32_t pool_handle,
                                      const char* const did,
                                      const char* document,
                                      const struct ExternError* err);

/*
 * Creates a DID on the indy ledger
 *
 * #param `pool_handle` - a previously opened pool handle to create for the DID
 * #param `did` - the fully qualified did string. Examples are did:sov:12345tgfvgrfdfrtg
 * #return `document` - a pointer that will be filled with the resolved document.
 * #return `err` - The result of the call. Will contain an error messages that occur
 */
extern int32_t indy_resolver_did_create(int32_t pool_handle,
                                        const char* const did,
                                        const char* const document,
                                        const struct ExternError* err);

extern int32_t indy_resolver_did_update(int32_t pool_handle,
                                        const char* const did,
                                        const char* const document,
                                        const struct ExternError* err);

extern int32_t indy_resolver_did_delete(int32_t pool_handle,
                                        const char* const did,
                                        const struct ExternError* err);

extern int32_t indy_resolver_did_new(const char* did,
                                     const struct ByteBuffer* did_verification_key,
                                     const struct ByteBuffer* did_secret_key,
                                     const struct ExternError* err);

extern int32_t indy_resolver_did_key_new(const struct ByteBuffer* did_new_public_key,
                                         const struct ByteBuffer* did_new_secret_key,
                                         const struct ExternError* err);

#endif
