use std::fmt::Display;

use crate::{codes::ResponseCode, SUPPORTED_HTTP_VERSION};

pub struct StatusLine {
    version: &'static str,
    code: ResponseCode,
}

impl StatusLine {
    pub fn new(code: ResponseCode) -> Self {
        Self {
            version: SUPPORTED_HTTP_VERSION,
            code,
        }
    }
}

impl Display for StatusLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.version, self.code as i32, self.code)
    }
}
