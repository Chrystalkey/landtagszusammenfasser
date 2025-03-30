# Authentication
## Basic Idea

Authentication works using API Keys. The Keys have to be supplied via the `X-API-Key` HTTP header on each request that requires authentication.
Thus it relies on a secure channel and does no encryption on its own.

Requests that require authentication are requests that do any kind of writing operation on the full dataset: any `POST`, `PUT` and `DELETE`.

Keys have a total length of 64 characters and start with `ltzf_`, followed by alphanumeric characters where capitalizaton matters.

## Scopes

API Keys are split into scopes that authorize the key to be used for different operations. The three scopes are:

1. Keyadder
2. Admin
3. Collector

Scopes with a lower number include the permissions of the ones with higher numbers.

The _Keyadder_ scope is the most powerful one as it allows interaction with the `/.../auth` endpoint using `POST` and `DELETE`, requesting new valid api keys in doing so.

The _Admin_ scope enables explicit writes and deletes of the main dataset by name. Keys with _Admin_ scope are allowed to use `PUT`, and `DELETE` on the `/.../vorgang/{vg_id}` and `/.../sitzung[{sid}]` endpoints. They are also allowed to use `PUT` on `/.../kalender/{parlament}/{datum} without date limitations.

The _Collector_ scope is allows the smallest set of operations. The only endpoint requiring authentication included in this scope is `PUT` on `/.../vorgang/`, which triggers the merging process in which data is deduplicated and integrated into the dataset. This scope also enables the use of `PUT` on the `/.../kalender/{parlament}/{datum}` up to one full day after the current timestamp.

All read-only endpoints do not require authentication and thus also no x-api-header key.

## Key Metadata

Keys are hashed using sha256 and stored in the database in that way. Associated with the key hash is
- its scope
- and expiration timestamp (default 1 year)
- a creation timestamp
- a timestamp of last usage (this means any endpoint call requiring authentication, regardless of the success of the associated operation)
- a flag for deletion
- a "created by" field, which references the api key this one was authorized by


## Authentication Workflow

The root of trust is an api key that is provided via an environment variable on startup of the backend service. 
This is automatically added to the database referencing itself in the "created by" field and has an expiry timer of 1 year.
Any number of distinct root keys with self-reference can be added, be aware.

Any subsequent keys are created using either the root key, or a key created by the root key with _Keyadder_ scope.
_Note that keys are not automatically invalidated after their creator has expired. This means: It is possible for the root key to have expired and all other keys to still be valid._

### Requesting a key
To request a key, run http `POST` on the `/api/v1/auth` endpoint, supplying the scope as described in the openapi spec. 
The Key can optionally be set to expire at a specified timestamp. It is not possible to create keys with no expiration date.

The key is returned as plain text in the response body. This is the only time the key can be extracted from the server, because only the hash is saved to the database.