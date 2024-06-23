/// ```rust
/// use bili_wbi_sign_rs::filename_in_url;
/// assert_eq!(filename_in_url("https://www.google.com/index.html"), Some("index"));
/// ```
pub fn filename_in_url(url: &str) -> Option<&str> {
    Some(url.rsplit_once('/')?.1.split_once('.')?.0)
}