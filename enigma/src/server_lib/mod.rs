//! Main library entry point for enigmaservice implementation.

mod server;

mod errors {
    error_chain! {}
}

use std::clone::Clone;
use std::io;
use std::marker::PhantomData;
use std::sync::Arc;

use crypt_config::config::SyncedConfig;
use enigmaservice;
use hyper;
use swagger::{Has, XSpanIdString};

pub use self::errors::*;

pub struct NewService<C> {
    marker: PhantomData<C>,
    config: Arc<SyncedConfig>,
}

impl<C> NewService<C> {
    pub fn new(config: Arc<SyncedConfig>) -> Self {
        NewService {
            marker: PhantomData,
            config: config,
        }
    }
}

impl<C> hyper::server::NewService for NewService<C>
where
    C: Has<XSpanIdString> + Clone + 'static,
{
    type Request = (hyper::Request, C);
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Instance = enigmaservice::server::Service<server::Server<C>, C>;

    /// Instantiate a new server.
    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(enigmaservice::server::Service::new(server::Server::new(
            self.config.clone(),
        )))
    }
}
