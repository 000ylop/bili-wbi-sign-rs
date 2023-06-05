#![feature(string_remove_matches)]

mod nav_resp;

use crate::nav_resp::{Nav, WbiImg};

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

fn parse_wbi_url(url: &str) -> Option<&str> {
    Some(url.rsplit_once('/')?.1.split_once('.')?.0)
}

pub async fn get_wbi_keys(client: &reqwest::Client) -> Result<String, Box<dyn std::error::Error>> {
    let resp = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .send()
        .await?;
    let result = resp.text().await?;
    let formed_result: Nav = serde_json::from_str(&result)?;
    let WbiImg { img_url, sub_url } = formed_result.data.wbi_img;
    let wbi_key = parse_wbi_url(&img_url)
        .ok_or("invalid wbi key url")?
        .to_owned()
        + parse_wbi_url(&sub_url).ok_or("invalid wbi key url")?;
    Ok(wbi_key)
}

#[cfg(blocking_req)]
pub fn get_wbi_keys_blocking(
    client: &reqwest::blocking::Client,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let resp = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .send()?;
    let result = resp.text()?;
    let formed_result: Nav = serde_json::from_str(&result)?;
    let WbiImg { img_url, sub_url } = formed_result.data.wbi_img;
    let wbi_key = parse_wbi_url(&img_url)
        .ok_or("invalid wbi key url")?
        .to_owned()
        + parse_wbi_url(&sub_url).ok_or("invalid wbi key url")?;
    Ok(wbi_key)
}

// ---- ChatGPT 3.5 ----
use itertools::Itertools;
use md5::compute;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use url::form_urlencoded;

pub fn wbi_sign_encode(mut params: HashMap<String, String>, mixin_key: &str) -> String {
    let curr_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    params.insert("wts".into(), curr_time.to_string()); // 添加 wts 字段
    let params_sorted = params
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

    form_urlencoded::Serializer::new(String::new())
        .extend_pairs(params_sorted.iter())
        .extend_pairs(std::iter::once(("w_rid".to_string(), wbi_sign)))
        .finish()
}
// ---- ChatGPT 3.5 ----
