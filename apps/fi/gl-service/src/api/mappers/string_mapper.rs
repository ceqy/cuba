//! 字符串转换工具
//! 统一处理空字符串 → Option 的转换

/// 空字符串转 Option trait
pub trait EmptyToOption {
    fn empty_to_option(self) -> Option<Self>
    where
        Self: Sized;
}

impl EmptyToOption for String {
    fn empty_to_option(self) -> Option<Self> {
        if self.is_empty() {
            None
        } else {
            Some(self)
        }
    }
}

impl<'a> EmptyToOption for &'a str {
    fn empty_to_option(self) -> Option<Self> {
        if self.is_empty() {
            None
        } else {
            Some(self)
        }
    }
}

/// 辅助函数：将字符串引用转换为 Option<String>
pub fn str_to_option(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/// 辅助函数：将 Option<String> 转换为字符串（空字符串作为默认值）
pub fn option_to_str(opt: Option<String>) -> String {
    opt.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_to_option_string() {
        assert_eq!("".to_string().empty_to_option(), None);
        assert_eq!("test".to_string().empty_to_option(), Some("test".to_string()));
    }

    #[test]
    fn test_empty_to_option_str() {
        assert_eq!("".empty_to_option(), None);
        assert_eq!("test".empty_to_option(), Some("test"));
    }

    #[test]
    fn test_str_to_option() {
        assert_eq!(str_to_option(""), None);
        assert_eq!(str_to_option("test"), Some("test".to_string()));
    }
}
