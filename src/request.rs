use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

use ahash::{HashMap, HashMapExt};
use anyhow::{anyhow, Context, Result};
use derive_more::derive::{Display, FromStr};
use itertools::Itertools;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, FromStr)]
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
            target.to_string()
        } else {
            return Err(anyhow!("Target value must start with '/'"));
        };
        let version = if version.starts_with("HTTP/") {
            version.to_string()
        } else {
            return Err(anyhow!("Invalid HTTP version"));
        };
        line.clear();

        let mut in_headers = true;
        let mut body: Option<String> = None;
        let mut headers = HashMap::new();
        while reader.read_line(&mut line)? > 0 {
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
            line.clear();
        }

        Ok(Self {
            method,
            target,
            version,
            headers,
            body,
        })
    }
}
