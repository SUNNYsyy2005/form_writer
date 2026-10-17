use std::fmt;
use std::error::Error;

// 定义错误类型 - 展示自定义错误类型和错误处理
#[derive(Debug)]
pub enum FormError {
    InvalidInput(String),
    ParseError(String),
    RangeError(String),
    FormatError(String),
    IoError(std::io::Error),
}

impl fmt::Display for FormError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FormError::InvalidInput(msg) => write!(f, "无效输入: {}", msg),
            FormError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            FormError::RangeError(msg) => write!(f, "范围错误: {}", msg),
            FormError::FormatError(msg) => write!(f, "格式错误: {}", msg),
            FormError::IoError(err) => write!(f, "IO错误: {}", err),
        }
    }
}

impl Error for FormError {}

impl From<std::io::Error> for FormError {
    fn from(err: std::io::Error) -> Self {
        FormError::IoError(err)
    }
}
