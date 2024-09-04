use std::{ffi::OsString, fs, path::PathBuf, sync::LazyLock};

use ahash::HashMap;
use anyhow::{anyhow, Result};
use tracing::error;

use crate::{
    codes::ResponseCode,
    request::{Method, Request},
};

#[allow(clippy::module_name_repetitions)]
pub type FnRoute = fn(&Request) -> Result<(String, ResponseCode)>;

#[derive(Debug, Clone)]
pub enum Route {
    Static(String, Option<ResponseCode>),
    Plain(String, Option<ResponseCode>),
    Dynamic(FnRoute),
}

impl Route {
    fn apply(&self, request: &Request) -> Result<(String, ResponseCode)> {
        match self {
            Self::Static(path, code) => {
                Ok((fs::read_to_string(path)?, code.unwrap_or(ResponseCode::Ok)))
            }
            Self::Plain(content, code) => Ok((content.clone(), code.unwrap_or(ResponseCode::Ok))),
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

    pub fn apply(&self, request: &Request) -> Result<(String, ResponseCode)> {
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
                Err(anyhow!("Invalid path traversal")) // TODO: Create custom error
            } else if path.exists() {
                Ok((fs::read_to_string(path)?, ResponseCode::Ok))
            } else {
                self.four_oh_four(request)
            }
        } else {
            self.four_oh_four(request)
        }
    }

    pub fn four_oh_four(&self, request: &Request) -> Result<(String, ResponseCode)> {
        self.four_oh_four.as_ref().map_or(
            Ok(("404 Not Found".to_string(), ResponseCode::Not_Found)),
            |route| route.apply(request),
        )
    }

    pub fn four_oh_five(
        &self,
        request: &Request,
        expecting: Method,
    ) -> Result<(String, ResponseCode)> {
        self.four_oh_five.as_ref().map_or_else(
            || {
                Ok((
                    format!(
                        "Method: {}, not allowed. Expecting: {expecting}, instead.",
                        request.method()
                    ),
                    ResponseCode::Method_Not_Allowed,
                ))
            },
            |route| route.apply(request),
        )
    }
}
