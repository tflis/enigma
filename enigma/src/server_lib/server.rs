use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;

use crypt_config::config::SyncedConfig;
use crypt_config::convert::{decrypt_document, encrypt_document, modify_find_query};
use futures::{self, Future};
use swagger;
use swagger::{Has, XSpanIdString};

use enigmaservice::models;
use enigmaservice::{Api, ApiError, DecryptResponse, EncryptResponse, QueryResponse};

#[derive(Clone)]
pub struct Server<C> {
  marker: PhantomData<C>,
  config: Arc<SyncedConfig>
}

impl<C> Server<C> {
  pub fn new(config: Arc<SyncedConfig>) -> Self {
    Server { marker: PhantomData, config: config }
  }
}

impl<C> Api<C> for Server<C>
where C: Has<XSpanIdString>
{
  /// Decrypt all fields from given JSON document which are specified in config
  /// file to be decrypted
  fn decrypt(&self, body: String, context: &C) -> Box<dyn Future<Item = DecryptResponse, Error = ApiError>> {
    let context = context.clone();
    println!("[INFO][decrypt] X-Span-ID: {:?}", context.get().0.clone());
    let config = self.config.get_config();
    match decrypt_document(&config, &body) {
      Ok(resp) => Box::new(futures::finished(DecryptResponse::DecryptionFinishedSuccessfully(resp))),
      Err(err) => Box::new(futures::finished(DecryptResponse::OneOrMoreFieldsAreIncorrect(models::ErrorResponse {
        code:    Some(403),
        message: Some(String::from(err.description()))
      })))
    }
  }

  /// Encrypt/hash all fields from given JSON document which are specified in
  /// config file to be encrypted/hashed
  fn encrypt(&self, body: String, context: &C) -> Box<dyn Future<Item = EncryptResponse, Error = ApiError>> {
    let context = context.clone();
    println!("[INFO][encrypt] X-Span-ID: {:?}", context.get().0.clone());
    let config = self.config.get_config();
    match encrypt_document(&config, &body) {
      Ok(resp) => Box::new(futures::finished(EncryptResponse::EncryptionFinishedSuccessfully(resp))),
      Err(err) => Box::new(futures::finished(EncryptResponse::InternalServerError(models::ErrorResponse {
        code:    Some(503),
        message: Some(String::from(err.description()))
      })))
    }
  }

  /// If the mongoDB query contains encrypted fields then those elements would
  /// be repalced in a way that query would be cappable to fetch specific
  /// elements
  fn query(&self, body: String, context: &C) -> Box<dyn Future<Item = QueryResponse, Error = ApiError>> {
    let context = context.clone();
    println!("[INFO][query] X-Span-ID: {:?}", context.get().0.clone());
    let config = self.config.get_config();
    match modify_find_query(&config, &body) {
      Ok(resp) => Box::new(futures::finished(QueryResponse::DecryptionFinishedSuccessfully(resp))),
      Err(err) => Box::new(futures::finished(QueryResponse::OneOrMoreFieldsAreIncorrect(models::ErrorResponse {
        code:    Some(403),
        message: Some(String::from(err.description()))
      })))
    }
  }
}
