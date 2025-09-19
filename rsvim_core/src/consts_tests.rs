use super::constant::*;

#[test]
fn mutex_timeout1() {
  assert!(*MUTEX_TIMEOUT_SECS > 0);
}

#[test]
fn window_drive_begin_regex_test1() {
  assert!(WINDOWS_DRIVE_BEGIN_REGEX.is_match("C:\\\\"));
  assert!(WINDOWS_DRIVE_BEGIN_REGEX.is_match("d:\\"));
  assert!(
    !WINDOWS_DRIVE_BEGIN_REGEX.is_match("//localhost:22/home/users/C:\\/bin")
  );
}

#[test]
fn http_url_begin_regex_test1() {
  assert!(HTTP_URL_BEGIN_REGEX.is_match("http://github.com"));
  assert!(HTTP_URL_BEGIN_REGEX.is_match("https://google.com"));
  assert!(
    !HTTP_URL_BEGIN_REGEX.is_match("ftp://localhost?=https://github.com")
  );
}
