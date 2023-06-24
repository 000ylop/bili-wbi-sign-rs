#![feature(string_remove_matches)]

mod nav_resp;

use crate::nav_resp::{Nav, WbiImg};
use log::debug;

const MIXIN_KEY_REORDER_MAP: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

/// SAFETY: key should be valid ASCII string
pub unsafe fn mixin_key(key: &[u8]) -> String {
    String::from_utf8_unchecked(
        MIXIN_KEY_REORDER_MAP
            .iter()
            .map(|index| key[*index])
            .take(32)
            .collect(),
    )
}

/// ```rust
/// use bili_wbi_sign_rs::filename_in_url;
/// assert_eq!(filename_in_url("https://www.google.com/index.html"), Some("index"));
/// ```
pub fn filename_in_url(url: &str) -> Option<&str> {
    Some(url.rsplit_once('/')?.1.split_once('.')?.0)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid wbi key uri")]
    ParseError,
    #[error("serde error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub const WBI_URI: &str = "https://api.bilibili.com/x/web-interface/nav";

pub fn parse_wbi_keys(resp: &[u8]) -> Result<String, Error> {
    let formed_result: Nav = serde_json::from_slice(&resp)?;
    let WbiImg { img_url, sub_url } = formed_result.data.wbi_img;
    let wbi_key = filename_in_url(&img_url).ok_or(Error::ParseError)?.to_owned()
        + filename_in_url(&sub_url).ok_or(Error::ParseError)?;
    Ok(wbi_key)
}

#[cfg(feature = "expires_time")]
/// after such duration, mixin_key will expire.
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

// ---- ChatGPT 3.5 ----
use itertools::Itertools;
use md5::compute;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use url::form_urlencoded;

pub fn wbi_sign_encode(
    mut params: HashMap<String, String>,
    mixin_key: &str,
) -> Vec<(String, String)> {
    let curr_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    params.insert("wts".into(), curr_time.to_string()); // 添加 wts 字段
    let mut params_sorted = params
        .into_iter()
        .sorted_by_key(|(k, _)| k.to_owned()) // 按照 key 重排参数
        .map(|(k, mut v)| {
            v.remove_matches(|ch| "!*()'".contains(ch));
            (k, v)
        })
        .collect::<Vec<(String, String)>>(); // 过滤 value 中的 "!'()*" 字符
    let query = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(params_sorted.iter())
        .finish(); // 序列化参数
    let wbi_sign = format!("{:x}", compute(&(query + &mixin_key).as_bytes())); // 计算 w_rid

    params_sorted.push(("w_rid".into(), wbi_sign));

    debug!("signed params: {params_sorted:?}");
    params_sorted
}
// ---- ChatGPT 3.5 ----
