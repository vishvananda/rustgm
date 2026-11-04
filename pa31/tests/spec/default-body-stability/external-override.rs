extern crate default_body_provider;use default_body_provider::Service;struct Client;impl Service for Client{fn value(&self)->u8{9}}
pub fn call()->u8{Client.value()}
