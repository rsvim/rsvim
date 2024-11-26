//! The "file-encoding" option for Vim buffer.

#[derive(Debug, Copy, Clone)]
pub enum FileEncoding {
  Utf8,
  Utf16,
  Utf32,
}

impl FileEncoding {
  pub fn to_string(&self) -> String {
    match self {
      FileEncoding::Utf8 => "utf-8".to_string(),
      FileEncoding::Utf16 => "utf-16".to_string(),
      FileEncoding::Utf32 => "utf-32".to_string(),
    }
  }
}

impl TryFrom<&str> for FileEncoding {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let lower_value = value.to_lowercase();
    match lower_value.as_str() {
      "utf-8" | "utf8" => Ok(FileEncoding::Utf8),
      "utf-16" | "utf16" => Ok(FileEncoding::Utf16),
      "utf-32" | "utf32" => Ok(FileEncoding::Utf32),
      _ => Err("Unknown FileEncoding value".to_string()),
    }
  }
}
