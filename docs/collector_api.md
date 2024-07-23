# Collector (Write-Enabled) API for the database
## Layout
All API Endpoints follow this schematic:
`/api/v{version}/collector/{collector id}/{end point}` 
Where the version of the api is used to discern wether old or newer versions of the protocol are used if any additions are made in the future.
- `version`: version of the api is used to discern wether old or newer versions of the protocol are used if any additions are made in the future.
- `collector id`: human-readable, unique id of the collector
- `end point`: the remainder of the url, as specified below

The collector MUST be authenticated to the database (but not necessarily vice versa), since malicious updates can corrupt the internal state of the database. However, the API is rate-limited to prevent abuse.


## Protocols
### Content Update Protocol
#### Protocol Overview
The CUP-Protocol is used to update the database with newly collected information, whatever they may be.
The Collector can only update, not delete or modify any existing entries directly, this task is performed by the database if necessary based on data provided by the collector(s). All Messages sent from the collector to the database MUST contain: A numeric UUID of the message, a timestamp, a human-readable ID for the collector. This makes logging and error recovery easier.  

There are, naively, two purposes to the CUP:
1. creation of a new "Gesetzesvorhaben", including all documents and informations associated with it
2. update of a "Gesetzesvorhaben" with new data - be it documents or any other entity or relation
Both stages rely on the "Gesetzesvorhaben" as the central reference point. Without it, they are useless.

**However, there is another consideration to be taken into account: That of shared data points.**  
All things that reference or are referenced by only one "Gesetzesvorhaben" are easily placed, but things like "Schlagworte" may be shared by many
"Gesetzesvorhaben" and may be created or removed in the future, making the database system unable to pre-fill all such data.
This necessitates some mechanism on how to create or update such shared data. These requirements are to be derived from this prompt:
- there must be a way to add another point of data (e.g some Ausschuss was mentioned for the first time, a Schlagwort was used for the first time)
  - Any Shared data point SHOULD NOT be added twice
  - Any New Data point MUST be rejected when it is confused with an existing one
  - There MUST be a way to merge two data points within the database
- there must be a way to _reliably_ reference an already created point of data (e.g. how do I know some Ausschuss was mentioned a second time?)


Of course there are some entities, which are to be considered under "special protection", e.g. "Abstimmungstyp" or "Parlamente" which should not be changed without human review. If there is no match to be found, some robust way has to be found to notify the developers of that circumstance. In this case these steps should be taken:
1. the collectors sends something to the database which it cannot fit into the schema
2. The Database...
   1. sends the error back to the collector, where it can be examined in full, including all raw material
   2. as well as logs a brief version of it, containing the message id & collector id
3. The Database notifies the developers over some channel (email/slack messages/signal messages/github issues/...)
4. These things must be changed manually from the outside. A way MUST be specified to gracefully handle this case.

#### Protocol Proposal
The CUP should be sending as little data as possible. For that purpose, apart from the ID for information interchange I propose this data format, which is wholly representative of the underlying data structures:

```Rust
struct CUPDoc{
  docid: String, // off_id
  datum: Datetime,
  url: Url,
  collector_url: Url,
  filename: String, 
  hash: String,
  typ: String,
}
struct CUPAusschuss{
  name: String,
  parlament: String,
}
struct CUPAusschussberatung{
  datum: Datetime,
  ausschuss: CUPAusschuss,
  dokument: CUPDoc,
  tops: Vec<String>,
}
struct CUPStatus{
  statusname: String,
  parlament: String,
}
struct CUPTransfer{
  msg_id: Uuid,
  timestamp: Datetime,
  gesvh_id : Uuid,
  titel: String,
  off_titel: String,
  verf_aenderung: bool,
  url_gesblatt: Url,
  id_gesblatt: String,
  trojaner: bool,
  federfuehrung: String,
  initiator: String,
  documents: Vec<CUPDoc>,
  ausschussberatungen: Vec<Ausschussberatung>,
  sonstige_ids: Vec<String>,
  status: Vec<CUPStatus>,
  eigenschaften: Vec<String>,
  schlagworte: Vec<String>,
}
```

While `CUPTransfer` is the main structure being transferred.
All data which is not different to a known state, apart from the `gesvh_id` MUST be empty. All data apart from it is presumed different.

```Rust
// 
struct CUPDBResponse{
responding_to: Uuid, // message id this is a response to
errors: CUPTransfer, // mirrored struct with all data which was rejected
}
```

#### Authentication
The following security properties are relevant:
  - Authentication
  - Replay protection

The following general properties are relevant:
  - Speed
  - An open secure session should be usable for multiple writes, minimizing overhead due to session initialization
##### The protocol (Secure Write)
 | Step | Collector                                                                                            | Database                                                                                                                                         | Method |
 | ---- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ------ |
 | 1    | Request to open Session, InitNonce                                                                   | -                                                                                                                                                | GET    |
 | 2    | -                                                                                                    | 401 Unauthorized: Random Session ID (Challenge), InitNonce, MessageNumber (0), Message Signature (Enc_KeyPrivServer (Hash (Session Identifier))) | ?      |
 | 3    | Write Data, MessageNumber (1), Session Identifier, Signature (Enc_KeyPrivCollector (Hash (Message))) | -                                                                                                                                                | PUT?   |
 | (4)  | (Collector sends more write updates, signed with its private key, message number and Session ID)     | -                                                                                                                                                | (PUT?) |
 | 5    | Collector closes session                                                                             | -                                                                                                                                                | PUT?   |

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
