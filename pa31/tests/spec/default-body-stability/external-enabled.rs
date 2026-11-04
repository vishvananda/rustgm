#![feature(default_api)]
extern crate default_body_provider;use default_body_provider::Service;struct Client;impl Service for Client{}
pub fn call()->u8{Client.value()}
