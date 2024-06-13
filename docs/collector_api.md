# Collector (Write-Enabled) API for the database
## Layout
All API Endpoints follow this schematic:
`/api/v{version}/collector/{end point}` 
Where the version of the api is used to discern wether old or newer versions of the protocol are used if any additions are made in the future.

The collector MUST be authenticated to the database (but not necessarily vice versa), since malicious updates can corrupt the internal state of the database. However, the API is rate-limited to prevent abuse.


## Protocols
### Content Update Protocol
#### The protocol
The CUP-Protocol is used to update the database with newly collected information, whatever they may be.
The Collector can only update, not delete or modify any existing entries directly, this task is performed by the database if necessary.
The CUP follows this schematic:
 | Step | Collector                                  | Database                                       | Method | Endpoint |
 | ---- | ------------------------------------------ | ---------------------------------------------- | ------ | -------- |
 | 1    | Sends Hashes of new Entries                | -                                              | PUT    | /hashes  |
 | 2    | -                                          | Compares hashes internally with stored entries | -      | -        |
 | 3    | -                                          | Sends unknown hashes back                      |
 | 4    | Sends Data associated with returned hashes after processing | -                                              | PUT    | /data    |

The Data for the hashes is shown in the table below and exchanged (there and back) in JSON format :

| name   | type           | description                                                                |
| ------ | -------------- | -------------------------------------------------------------------------- |
| hashes | list of hashes | contains a list of hashes as strings in hex encoding (base64? DISCUSSSION) |

The data is then being uploaded like this:

| name | type   | description                                                                         |
| ---- | ------ | ----------------------------------------------------------------------------------- |
| hash | string | contains the hash of the current data point, equivalent to the hash sent beforehand |
/*TODO: Structure that can contain any updated parameters. maybe just a Key/value store */

#### Authentication
The following security properties are relevant:
  - Authentication
  - Replay protection

The following general properties are relevant:
  - Speed
  - An open secure session should be usable for multiple writes, minimizing overhead due to session initialization
##### The protocol (Secure Write)
 | Step | Collector                                  | Database                                                           | Method |
 | ---- | ------------------------------------------ | ------------------------------------------------------------------ | ------ |
 | 1    | Request to open Session, InitNonce         | -                                                                  | GET    |
 | 2    | -                                          | 401 Unauthorized: Random Session ID (Challenge), InitNonce, MessageNumber (0), Message Signature (Enc_KeyPrivServer (Hash (Session Identifier)))                                                                           | ?      |
 | 3    | Write Data, MessageNumber (1), Session Identifier, Signature (Enc_KeyPrivCollector (Hash (Message))) |-  | PUT?
 | (4)  | (Collector sends more write updates, signed with its private key, message number and Session ID)  | -                                                                  | (PUT?)   |
 | 5    | Collector closes session                   | -                                                                   | PUT?            

 ### Source Locator Update

If a source URL or any kind of crawling/collecting activity locator changes in the database, the database can notify the collector of that change.
Since it is the single source of truth, the collector cannot by itself decide on an update or change.

The process of notifying collectors of a changed resource is done via the collectors requesting the database to send them updated or changed entries through the SLU-Protocol everytime they are attemping a collecting operation to ensure all URLs are correct.

DISCUSSION: Is it useful to add a list of currently actively pursued cases to the collector response so the database does not have to push out everything?

The SLU follows this schematic:
 | Step | Collector                          | Database                                  | Method | Endpoint    |
 | ---- | ---------------------------------- | ----------------------------------------- | ------ | ----------- |
 | 1    | Sends a request to receive updates | -                                         | GET    | /slu-update |
 | 2    | -                                  | sends updated ressources and informations | -      | -           |

The data returned from the database looks like this:
| name       | type                 | description                                                                    |
| ---------- | -------------------- | ------------------------------------------------------------------------------ |
| vorgang_id | string/int           | the unique identifier for the law process under scrutiny                       |
| updates    | list of "update-kvs" | list of updated locators with their respective values as "locator":"new value" |
