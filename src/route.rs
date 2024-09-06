use std::{ffi::OsString, fs, path::PathBuf, sync::LazyLock};

use ahash::HashMap;
use anyhow::{anyhow, Result};
use tracing::error;

use crate::{
    codes::ResponseCode,
    request::{Method, Request},
};

#[allow(clippy::module_name_repetitions)]
pub struct RouteResponse {
    content: String,
    response_code: ResponseCode,
    require_logging: bool,
    logging_context: Option<String>,
}

impl RouteResponse {
    pub const fn new_ok(content: String, response_code: ResponseCode) -> Self {
        Self {
            content,
            response_code,
            require_logging: false,
            logging_context: None,
        }
    }

    pub const fn new_logging(
        content: String,
        response_code: ResponseCode,
        logging_context: Option<String>,
    ) -> Self {
        Self {
            content,
            response_code,
            require_logging: true,
            logging_context,
        }
    }

    pub const fn code(&self) -> ResponseCode {
        self.response_code
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    // TODO: Fill out the rest of the functions for logging
}

impl From<(String, ResponseCode)> for RouteResponse {
    fn from(val: (String, ResponseCode)) -> Self {
        Self::new_ok(val.0, val.1)
    }
}

impl From<(String, ResponseCode, bool)> for RouteResponse {
    fn from(val: (String, ResponseCode, bool)) -> Self {
        if val.2 {
            Self::new_logging(val.0, val.1, None)
        } else {
            (val.0, val.1).into()
        }
    }
}

impl From<(String, ResponseCode, Option<String>)> for RouteResponse {
    fn from(val: (String, ResponseCode, Option<String>)) -> Self {
        Self::new_logging(val.0, val.1, val.2)
    }
}

impl From<(String, ResponseCode, String)> for RouteResponse {
    fn from(val: (String, ResponseCode, String)) -> Self {
        (val.0, val.1, Some(val.2)).into()
    }
}

impl From<(String, ResponseCode, &str)> for RouteResponse {
    fn from(val: (String, ResponseCode, &str)) -> Self {
        (val.0, val.1, Some(val.2.into())).into()
    }
}

#[allow(clippy::module_name_repetitions)]
pub type FnRoute = fn(&Request) -> Result<RouteResponse>;

#[derive(Debug, Clone)]
pub enum Route {
    Static(String, Option<ResponseCode>),
    Plain(String, Option<ResponseCode>),
    Dynamic(FnRoute),
}

impl Route {
    fn apply(&self, request: &Request) -> Result<RouteResponse> {
        match self {
            Self::Static(path, code) => {
                Ok((fs::read_to_string(path)?, code.unwrap_or(ResponseCode::Ok)).into())
            }
            Self::Plain(content, code) => {
                Ok((content.clone(), code.unwrap_or(ResponseCode::Ok)).into())
            }
            Self::Dynamic(f) => f(request),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Routes {
    map: HashMap<(Method, String), Route>,
    four_oh_four: Option<Route>,
    four_oh_five: Option<Route>,
    static_dir: Option<PathBuf>,
}

impl Routes {
    pub fn add_static<A: Into<String>, B: Into<String>>(
        &mut self,
        target: A,
        path: B,
        code: Option<ResponseCode>,
    ) -> Result<()> {
        let target: String = target.into();
        if let std::collections::hash_map::Entry::Vacant(e) = self.map.entry((Method::GET, target))
        {
            // TODO: Verify that the provided target is valid
            // TODO: Verify that the provided path is valid
            e.insert(Route::Static(path.into(), code));
            Ok(())
        } else {
            // TODO: Implement custom error type to handle this
            Err(anyhow!("Target already exists"))
        }
    }

    pub fn add_plain<A: Into<String>, B: Into<String>>(
        &mut self,
        target: A,
        content: B,
        code: Option<ResponseCode>,
    ) -> Result<()> {
        let target: String = target.into();
        if let std::collections::hash_map::Entry::Vacant(e) = self.map.entry((Method::GET, target))
        {
            // TODO: Verify that the provided target is valid
            e.insert(Route::Plain(content.into(), code));
            Ok(())
        } else {
            // TODO: Implement custom error type to handle this
            Err(anyhow!("Target already exists"))
        }
    }

    pub fn add_dynamic<A: Into<String>, M: Into<Vec<Method>>>(
        &mut self,
        target: A,
        method: M,
        f: FnRoute,
    ) -> Result<()> {
        let target: String = target.into();
        // TODO: Verify that the provided target is valid

        for method in method.into() {
            if let std::collections::hash_map::Entry::Vacant(e) =
                self.map.entry((method, target.clone()))
            {
                e.insert(Route::Dynamic(f));
            } else {
                // TODO: Implement custom error type to handle this
                return Err(anyhow!("Target already exists"));
            }
        }
        Ok(())
    }

    pub fn set_404(&mut self, route: Route) {
        self.four_oh_four = Some(route);
    }

    pub fn set_static_dir<A: Into<PathBuf>>(&mut self, path: A) {
        self.static_dir = Some(path.into());
    }

    pub fn apply(&self, request: &Request) -> Result<RouteResponse> {
        // TODO: Handle wildcard targets
        static PATH_BOUNDS: LazyLock<OsString> =
            LazyLock::new(|| PathBuf::from("./").canonicalize().unwrap().into_os_string());
        // TODO: This clone is not ideal
        if let Some(route) = self.map.get(&(request.method(), request.target().clone())) {
            route.apply(request)
        } else if let Some(dir) = self.static_dir.as_ref() {
            // We only accept GET requests to this route, so we'll return a 405 otherwise
            if !request.method().is_get() {
                return self.four_oh_five(request, Method::GET);
            }
            // Build a new path with the presumably relative path provided by the user and the
            // PATH_BOUNDS of this application. // TODO: Consider if we want to allow this outside
            // TODO: of the static_dir, probably not
            let mut path = dir.clone();
            path.push(request.target_as_path());
            let Ok(path) = path.canonicalize() else {
                // If we fail to canonicalize the path it's either not valid for this server to
                // return, not sure if this will ever actually happen, or we dont have it
                // We'll return a 404 either way
                if path.exists() {
                    error!("Failed to canonicalize path: {:#?}", path.into_os_string());
                }
                return self.four_oh_four(request);
            };
            if !path.starts_with(&*PATH_BOUNDS) {
                //TODO: Return a 401 Unauthorized here. We still want to make sure this gets logged
                //for security purposes, likely more effectively than we've done here.
                Ok(("Unauthorized".into(), ResponseCode::Unauthorized, "Invalid path traversal").into())
            } else if path.exists() {
                Ok((fs::read_to_string(path)?, ResponseCode::Ok).into())
            } else {
                self.four_oh_four(request)
            }
        } else {
            self.four_oh_four(request)
        }
    }

    pub fn four_oh_four(&self, request: &Request) -> Result<RouteResponse> {
        self.four_oh_four.as_ref().map_or(
            Ok(("404 Not Found".to_string(), ResponseCode::Not_Found).into()),
            |route| route.apply(request),
        )
    }

    pub fn four_oh_five(&self, request: &Request, expecting: Method) -> Result<RouteResponse> {
        self.four_oh_five.as_ref().map_or_else(
            || {
                let error = format!(
                    "Method: {}, not allowed. Expecting: {expecting}, instead.",
                    request.method()
                );
                Ok((
                    error.clone(),
                    ResponseCode::Method_Not_Allowed,
                    error, // We want to provide context here that requires this be logged
                )
                    .into())
            },
            |route| route.apply(request),
        )
    }
}
