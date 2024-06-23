use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use itertools::Itertools;
use md5::compute;
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

    #[cfg(feature = "log")]
    log::debug!("signed params: {params_sorted:?}");
    params_sorted
}
