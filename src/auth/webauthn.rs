use nom::{bytes::complete::take, error::ParseError, number::complete};
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct AttestationObject<'a> {
    #[serde(rename = "authData")]
    auth_data: &'a [u8],
}

pub fn get_aaguid(raw_attestation_object: &[u8]) -> nom::IResult<&[u8], Uuid> {
    let parsed: AttestationObject =
        serde_cbor_2::from_slice(raw_attestation_object).map_err(|e| {
            tracing::error!("failed to parse attestation object: {e}");
            nom::Err::Failure(nom::error::Error::from_error_kind(
                raw_attestation_object,
                nom::error::ErrorKind::Fail,
            ))
        })?;

    let (i, _) = take(32usize)(parsed.auth_data)?;
    let (i, flags) = complete::u8(i)?;
    let (i, _) = complete::be_u32(i)?;

    let has_attestation_cred_data = (flags & 0b0100_0000) != 0;
    if !has_attestation_cred_data {
        return Err(nom::Err::Failure(nom::error::Error::from_error_kind(
            i,
            nom::error::ErrorKind::Verify,
        )));
    }

    let (i, aaguid) = take(16usize)(i)?;
    let uuid = Uuid::from_slice(aaguid).map_err(|e| {
        tracing::error!("failed to parse aaguid: {e}");
        nom::Err::Failure(nom::error::Error::from_error_kind(
            i,
            nom::error::ErrorKind::Verify,
        ))
    })?;

    Ok((i, uuid))
}
