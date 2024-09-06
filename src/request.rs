use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

use ahash::{HashMap, HashMapExt};
use anyhow::{anyhow, Context, Result};
use derive_more::derive::{Display, FromStr, IsVariant};
use itertools::Itertools;
use tracing::debug;
use urlencoding::decode;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, FromStr, Hash, IsVariant)]
pub enum Method {
    GET,
    HEAD,
    PUT,
    POST,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl From<Method> for Vec<Method> {
    fn from(val: Method) -> Self {
        vec![val]
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    method: Method,
    target: String,
    version: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl Request {
    pub fn parse(mut reader: BufReader<&mut TcpStream>) -> Result<Self> {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let (method, target, version) = line
            .split_whitespace()
            .take(3)
            .collect_tuple()
            .context("Failed to parse start-line")?; // Error for the collect_touple
        let method: Method = method.parse().context("Failed to parse HTTP Method")?;
        let target = if target.is_empty() {
            String::from("/")
        } else if target.starts_with('/') {
            // NOTE: Looks like the decode function here resolves path travesal at this point by
            // resolving the path now
            // I'll leave the code for preventing path traversal in place in the routes apply
            // function just in case. I'd rather not rely on this being here
            decode(target)?.into_owned()
        } else {
            return Err(anyhow!("Target value must start with '/'"));
        };
        debug!("Target: {target}");
        let version = if version.starts_with("HTTP/") {
            version.to_string()
        } else {
            return Err(anyhow!("Invalid HTTP version"));
        };

        let mut in_headers = true;
        let mut body: Option<String> = None;
        let mut headers = HashMap::new();
        for line in reader
            .lines() // FIXME: Because of the code bellow the body is never captured
            .take_while(|line| line.as_ref().is_ok_and(|line| !line.is_empty()))
        {
            let line = line?;
            dbg!(&line);
            if line.trim().is_empty() {
                in_headers = false;
                continue;
            }

            if in_headers {
                let (name, content) = line
                    .trim()
                    .split_once(':')
                    .context("Failed to parse header")?;
                let name = name.to_lowercase();
                let content = content.trim().to_string();
                headers.insert(name, content);
            } else if let Some(body) = &mut body {
                body.push_str(&line);
            } else if !line.trim().is_empty() {
                body = Some(line.to_string());
            }
        }
        //debug!("Finished reading request body");

        Ok(Self {
            method,
            target,
            version,
            headers,
            body,
        })
    }

    pub const fn method(&self) -> Method {
        self.method
    }

    pub const fn target(&self) -> &String {
        &self.target
    }

    pub fn target_as_path(&self) -> &str {
        self.target.trim_start_matches('/')
    }

    pub const fn version(&self) -> &String {
        &self.version
    }

    pub const fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub const fn body(&self) -> Option<&String> {
        self.body.as_ref()
    }

    pub fn as_string(&self) -> String {
        let mut out = format!("{} {} {}\n", self.method(), self.target(), self.version());
        for (key, val) in self.headers() {
            out.push_str(&format!("{key}: {val}\n"));
        }
        if let Some(body) = self.body() {
            out.push_str(body);
        }

        out
    }
}
