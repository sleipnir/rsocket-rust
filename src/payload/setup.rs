extern crate bytes;

use crate::frame::Setup;
use crate::mime::MIME_BINARY;
use bytes::Bytes;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SetupPayload {
  m: Option<Bytes>,
  d: Option<Bytes>,
  keepalive: (Duration, Duration),
  mime_m: Option<String>,
  mime_d: Option<String>,
}

#[derive(Clone)]
pub struct SetupPayloadBuilder {
  inner: SetupPayload,
}

impl SetupPayload {
  pub fn builder() -> SetupPayloadBuilder {
    SetupPayloadBuilder::new()
  }
}

impl SetupPayloadBuilder {
  fn new() -> SetupPayloadBuilder {
    SetupPayloadBuilder {
      inner: SetupPayload {
        m: None,
        d: None,
        keepalive: (Duration::from_secs(20), Duration::from_secs(90)),
        mime_m: Some(String::from(MIME_BINARY)),
        mime_d: Some(String::from(MIME_BINARY)),
      },
    }
  }

  pub fn set_metadata(mut self, metadata: Bytes) -> Self{
    self.inner.m = Some(metadata);
    self
  }

  pub fn set_metadata_utf8(mut self, metadata: &str) -> Self{
    self.set_metadata(Bytes::from(metadata))
  }

  pub fn set_data(mut self, data: Bytes) -> Self {
    self.inner.d = Some(data);
    self
  }

  pub fn set_data_utf8(mut self, data: &str) -> Self {
    self.set_data(Bytes::from(data))
  }

  pub fn set_keepalive(
    mut self,
    tick_period: Duration,
    ack_timeout: Duration,
    missed_acks: u64,
  ) -> Self {
    let lifetime_mills = (ack_timeout.as_millis() as u64) * missed_acks;
    self.inner.keepalive = (tick_period, Duration::from_millis(lifetime_mills));
    self
  }

  pub fn set_data_mime_type(mut self, mime: &str) -> Self {
    self.inner.mime_d = Some(String::from(mime));
    self
  }
  pub fn set_metadata_mime_type(mut self, mime: &str) -> Self {
    self.inner.mime_m = Some(String::from(mime));
    self
  }

  pub fn build(self) -> SetupPayload {
    self.inner.clone()
  }
}

impl SetupPayload {
  pub fn metadata(&self) -> Option<Bytes> {
    self.m.clone()
  }

  pub fn data(&self) -> Option<Bytes> {
    self.d.clone()
  }

  pub fn keepalive_interval(&self) -> Duration {
    self.keepalive.0.clone()
  }

  pub fn keepalive_lifetime(&self) -> Duration {
    self.keepalive.1.clone()
  }

  pub fn metadata_mime_type(&self) -> Option<String> {
    self.mime_m.clone()
  }

  pub fn data_mime_type(&self) -> Option<String> {
    self.mime_d.clone()
  }
}

impl From<&Setup> for SetupPayload {
  fn from(input: &Setup) -> SetupPayload {
    let mut bu = SetupPayload::builder();
    if let Some(b) = input.get_data() {
      bu = bu.set_data(b);
    }
    if let Some(b) = input.get_metadata() {
      bu = bu.set_metadata(b);
    }
    // TODO: fill other properties.
    bu = bu.set_data_mime_type(input.get_mime_data());
    bu = bu.set_metadata_mime_type(input.get_mime_metadata());
    // bu.set_data_mime_type(String::input.get_mime_data());
    let mut pa = bu.build();
    pa.keepalive = (input.get_keepalive(),input.get_lifetime());
    pa
  }
}
