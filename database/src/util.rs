use ulid::Ulid;
use uuid::Uuid;

pub fn from_ulid(ulid: Ulid) -> Uuid {
    return Uuid::from_u128(ulid.0);
}
