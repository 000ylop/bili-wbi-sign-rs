#![feature(string_remove_matches)]

mod types;
use crate::types::{Nav, WbiImg};
mod utils;
pub use crate::utils::filename_in_url;
mod sign;
pub use sign::wbi_sign_encode;

const MIXIN_KEY_REORDER_MAP: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

/// Once you got raw WBI key, you could call this function,
///
/// to get `mixin_key`.
///
/// SAFETY: key should be valid ASCII string
pub unsafe fn mixin_key(raw_key: impl AsRef<[u8]>) -> String {
    let key = raw_key.as_ref();
    String::from_utf8_unchecked(
        MIXIN_KEY_REORDER_MAP
            .iter()
            .map(|index| key[*index])
            .take(32)
            .collect(),
    )
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid wbi key uri")]
    ParseError,
    #[error("serde error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// WBI params url
pub const WBI_URI: &str = "https://api.bilibili.com/x/web-interface/nav";

/// You should GET `WBI_URI`, and pass the response to this.
///
/// So you get the raw WBI key.
pub fn parse_wbi_keys(resp: impl AsRef<[u8]>) -> Result<String, Error> {
    let resp = resp.as_ref();
    let formed_result: Nav = serde_json::from_slice(&resp)?;
    let WbiImg { img_url, sub_url } = formed_result.data.wbi_img;
    let wbi_key = filename_in_url(&img_url)
        .ok_or(Error::ParseError)?
        .to_owned()
        + filename_in_url(&sub_url).ok_or(Error::ParseError)?;
    Ok(wbi_key)
}

#[cfg(feature = "expires_time")]
/// After such duration, mixin_key will expire.
///
/// [`moka`]: https://docs.rs/moka/latest/moka/
/// [usage]: https://github.com/mokurin000/yuanshen-workarounds-bot/blob/0551b3a5978ef0d9dd82fadb689304a5eb17b18a/src/expiry.rs#L5-L14
/// See [usage] with [`moka`].
pub fn expires_after() -> Option<chrono::Duration> {
    use chrono::prelude::*;
    let utc = Utc::now().naive_utc();
    let tz = FixedOffset::east_opt(8 * 3600)?;
    let utc8_now = tz.from_utc_datetime(&utc);
    let utc8_nextday = utc8_now
        .date_naive()
        .succ_opt()?
        .and_hms_opt(0, 0, 0)?
        .and_local_timezone(tz)
        .single()?;
    Some(utc8_nextday.signed_duration_since(utc8_now))
}
