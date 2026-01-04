use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};
use uuid::Uuid;

#[derive(
    Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct UlidId(pub ulid::Ulid);

impl Default for UlidId {
    fn default() -> Self {
        Self::nil()
    }
}

impl UlidId {
    pub fn nil() -> Self {
        UlidId(ulid::Ulid::nil())
    }

    pub fn new() -> Self {
        UlidId(ulid::Ulid::new())
    }
}

impl From<uuid::Uuid> for UlidId {
    fn from(value: Uuid) -> Self {
        UlidId(ulid::Ulid::from(value))
    }
}

impl From<UlidId> for uuid::Uuid {
    fn from(value: UlidId) -> Self {
        value.0.into()
    }
}

// sqlx postgres mapping -->
impl Type<Postgres> for UlidId {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("uuid")
    }
}

impl PgHasArrayType for UlidId {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::array_of("uuid")
    }
}

impl Encode<'_, Postgres> for UlidId {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let uuid: Uuid = self.0.into();
        buf.extend_from_slice(uuid.as_bytes());

        Ok(IsNull::No)
    }
}

impl Decode<'_, Postgres> for UlidId {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        let uuid = match value.format() {
            PgValueFormat::Binary => Uuid::from_slice(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse(),
        }
        .map_err(Into::<BoxDynError>::into)?;
        Ok(UlidId::from(uuid))
    }
}
