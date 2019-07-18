#ifndef __indy__resolver__context__included__
#define __indy__resolver__context__included__

/**
*   There is no existing method to update or delete a context_json once written
*   to the ledger.
*
*
*   FUTURE: When available, methods will be added to implement these features.
*           Methods will be called `indy_res_context_update` and indy_res_context_deactivate`
*/

/**
*   Given a `context_json` structure, will create the specified context on the ledger and
*   returns the its identifier as a did in `context_did`
*
* Examples for `context_json`:
*   {
*      "@context": [
*          "https://...",
*          "did:sov:...",
*          { "name": "http://...",
*            "address": "https://...",
*            "mail": "did:sov:..." }
*       ]
*   }
*
*  Returns 1 on success, 0 otherwise. If 0 is returned for detailed failure information
*  the caller should check `err`
*/
extern int32_t indy_res_context_create(int32_t pool_handle,
                                       const char* const submitter_did,
                                       const char* const submitter_did_private_key,
                                       int32_t submitter_did_private_key_length,
                                       const char* const context_json,
                                       const char* context_did,
                                       const struct ExternError* err);

/**
*   Given a `context_did` string, returns the `context_json`.
*
*   Returns 1 on success, 0 otherwise
*/
extern int32_t indy_res_context_read(const char* const context_did,
                                     const char* context_json,
                                     const struct ExternError* err);



extern int32_t indy_res_build_context_create_transaction();

#endif
