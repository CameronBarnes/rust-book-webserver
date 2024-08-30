use std::{ffi::OsString, fs, path::PathBuf, sync::LazyLock};

use ahash::HashMap;
use anyhow::{anyhow, Result};

use crate::{codes::ResponseCode, request::Request};

#[allow(clippy::module_name_repetitions)]
pub type FnRoute = fn(&Request) -> Result<(String, ResponseCode)>;

#[derive(Debug, Clone)]
pub enum Route {
    Static(String),
    Plain(String),
    Dynamic(FnRoute),
}

impl Route {
    fn apply(&self, request: &Request) -> Result<(String, ResponseCode)> {
        match self {
            Self::Static(path) => Ok((fs::read_to_string(path)?, ResponseCode::Ok)),
            Self::Plain(content) => Ok((content.clone(), ResponseCode::Ok)),
            Self::Dynamic(f) => f(request),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Routes {
    map: HashMap<String, Route>,
    four_oh_four: Option<Route>,
    static_dir: Option<PathBuf>,
}

impl Routes {
    pub fn add_static<A: Into<String>, B: Into<String>>(
        &mut self,
        target: A,
        path: B,
    ) -> Result<()> {
        let target: String = target.into();
        if let std::collections::hash_map::Entry::Vacant(e) = self.map.entry(target) {
            // TODO: Verify that the provided target is valid
            // TODO: Verify that the provided path is valid
            e.insert(Route::Static(path.into()));
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
    ) -> Result<()> {
        let target: String = target.into();
        if let std::collections::hash_map::Entry::Vacant(e) = self.map.entry(target) {
            // TODO: Verify that the provided target is valid
            e.insert(Route::Plain(content.into()));
            Ok(())
        } else {
            // TODO: Implement custom error type to handle this
            Err(anyhow!("Target already exists"))
        }
    }

    pub fn add_dynamic<A: Into<String>>(
        &mut self,
        target: A,
        f: FnRoute,
    ) -> Result<()> {
        let target: String = target.into();
        if let std::collections::hash_map::Entry::Vacant(e) = self.map.entry(target) {
            // TODO: Verify that the provided target is valid
            e.insert(Route::Dynamic(f));
            Ok(())
        } else {
            // TODO: Implement custom error type to handle this
            Err(anyhow!("Target already exists"))
        }
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
        if let Some(route) = self.map.get(request.target()) {
            route.apply(request)
        } else if let Some(dir) = self.static_dir.as_ref() {
            let target = PathBuf::from(request.target()).canonicalize()?;
            let mut path = dir.clone();
            path.push(target);
            if !path.starts_with(&*PATH_BOUNDS) {
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
        self.four_oh_four
            .as_ref()
            .map_or(Ok(("404 Not Found".to_string(), ResponseCode::Not_Found)), |route| {
                route.apply(request)
            })
    }
}
