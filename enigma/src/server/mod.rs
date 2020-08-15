//! Main library entry point for enigmaservice implementation.

#![allow(unused_imports)]

use async_trait::async_trait;
use futures::{future, Stream, StreamExt, TryFutureExt, TryStreamExt};
use hyper::server::conn::Http;
use hyper::service::Service;
use log::info;
use openssl::ssl::SslAcceptorBuilder;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use swagger::{Has, XSpanIdString};
use swagger::auth::MakeAllowAllAuthenticator;
use swagger::EmptyContext;
use tokio::net::TcpListener;

use crypt_config::config::SyncedConfig;
use crypt_config::convert::{decrypt_document, encrypt_document, modify_find_query};

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use enigmaservice::models;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
/// Builds an SSL implementation for Simple HTTPS from some hard-coded file names
pub async fn create(addr: &str, config: Arc<SyncedConfig>, https: bool) {
    let addr = addr.parse().expect("Failed to parse bind address");

    let server = Server::new(config);

    let service = MakeService::new(server);

    let service = MakeAllowAllAuthenticator::new(service, "cosmo");

    let mut service =
        enigmaservice::server::context::MakeAddContext::<_, EmptyContext>::new(
            service
        );

    if https {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
        {
            unimplemented!("SSL is not implemented for the examples on MacOS, Windows or iOS");
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
        {
            let mut ssl = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls()).expect("Failed to create SSL Acceptor");

            // Server authentication
            ssl.set_private_key_file("server-key.pem", SslFiletype::PEM).expect("Failed to set private key");
            ssl.set_certificate_chain_file("server-chain.pem").expect("Failed to set cerificate chain");
            ssl.check_private_key().expect("Failed to check private key");

            let tls_acceptor = Arc::new(ssl.build());
            let mut tcp_listener = TcpListener::bind(&addr).await.unwrap();
            let mut incoming = tcp_listener.incoming();

            while let (Some(tcp), rest) = incoming.into_future().await {
                if let Ok(tcp) = tcp {
                    let addr = tcp.peer_addr().expect("Unable to get remote address");
                    let service = service.call(addr);
                    let tls_acceptor = Arc::clone(&tls_acceptor);

                    tokio::spawn(async move {
                        let tls = tokio_openssl::accept(&*tls_acceptor, tcp).await.map_err(|_| ())?;

                        let service = service.await.map_err(|_| ())?;

                        Http::new().serve_connection(tls, service).await.map_err(|_| ())
                    });
                }

                incoming = rest;
            }
        }
    } else {
        // Using HTTP
        hyper::server::Server::bind(&addr).serve(service).await.unwrap()
    }
}

#[derive(Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
    config: Arc<SyncedConfig>,
}

impl<C> Server<C> {
    pub fn new(config: Arc<SyncedConfig>) -> Self {
        Server{marker: PhantomData, config: config}
    }
}


use enigmaservice::{
    Api,
    DecryptResponse,
    EncryptResponse,
    QueryResponse,
};
use enigmaservice::server::MakeService;
use std::error::Error;
use swagger::ApiError;

#[async_trait]
impl<C> Api<C> for Server<C> where C: Has<XSpanIdString> + Send + Sync
{
    /// Decrypt all fields from given JSON document which are specified in config file to be decrypted
    async fn decrypt(
        &self,
        body: String,
        context: &C) -> Result<DecryptResponse, ApiError>
    {
        let context = context.clone();
        info!("decrypt(\"{}\") - X-Span-ID: {:?}", body, context.get().0.clone());
        let config = self.config.get_config();
        match decrypt_document(&config, &body) {
          Ok(resp) => Ok(DecryptResponse::DecryptionFinishedSuccessfully(resp)),
          Err(err) => Ok(DecryptResponse::OneOrMoreFieldsAreIncorrect(models::ErrorResponse {
            code:    Some(403),
            message: Some(String::from(err.to_string()))
          }))
        }
    }

    /// Encrypt/hash all fields from given JSON document which are specified in config file to be encrypted/hashed
    async fn encrypt(
        &self,
        body: String,
        context: &C) -> Result<EncryptResponse, ApiError>
    {
        let context = context.clone();
        info!("encrypt(\"{}\") - X-Span-ID: {:?}", body, context.get().0.clone());
        let config = self.config.get_config();
        match encrypt_document(&config, &body) {
          Ok(resp) => Ok(EncryptResponse::EncryptionFinishedSuccessfully(resp)),
          Err(err) => Ok(EncryptResponse::InternalServerError(models::ErrorResponse {
            code:    Some(503),
            message: Some(String::from(err.to_string()))
          }))
        }
    }

    /// If the mongoDB query contains encrypted fields then those elements would be repalced in a way that query would be cappable to fetch specific elements
    async fn query(
        &self,
        body: String,
        context: &C) -> Result<QueryResponse, ApiError>
    {
        let context = context.clone();
        info!("query(\"{}\") - X-Span-ID: {:?}", body, context.get().0.clone());
        let config = self.config.get_config();
        match modify_find_query(&config, &body) {
          Ok(resp) => Ok(QueryResponse::DecryptionFinishedSuccessfully(resp)),
          Err(err) => Ok(QueryResponse::OneOrMoreFieldsAreIncorrect(models::ErrorResponse {
            code:    Some(403),
            message: Some(String::from(err.to_string()))
          }))
        }
    }
}
