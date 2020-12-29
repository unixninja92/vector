use crate::{
    event::{Event, LookupBuf},
    internal_events::{SocketEventReceived, SocketMode},
    log_event,
    config::log_schema,
    shutdown::ShutdownSignal,
    sources::{
        util::{build_unix_datagram_source, build_unix_stream_source},
        Source,
    },
    Pipeline,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio_util::codec::LinesCodec;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct UnixConfig {
    pub path: PathBuf,
    #[serde(default = "default_max_length")]
    pub max_length: usize,
    pub host_key: Option<LookupBuf>,
}

fn default_max_length() -> usize {
    bytesize::kib(100u64) as usize
}

impl UnixConfig {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            max_length: default_max_length(),
            host_key: None,
        }
    }
}

/**
* Function to pass to build_unix_*_source, specific to the basic unix source.
* Takes a single line of a received message and builds an Event object.
**/
fn build_event(host_key: LookupBuf, received_from: Option<Bytes>, line: &str) -> Option<Event> {
    let byte_size = line.len();
    let mut event = log_event! {
        log_schema().message_key().clone() => line,
        log_schema().timestamp_key().clone() => chrono::Utc::now(),
    };
    event.as_mut_log().insert(
        log_schema().source_type_key().clone(),
        Bytes::from("socket"),
    );
    if let Some(host) = received_from {
        event.as_mut_log().insert(host_key, host);
    }
    emit!(SocketEventReceived {
        byte_size,
        mode: SocketMode::Unix
    });
    Some(event)
}

pub(super) fn unix_datagram(
    path: PathBuf,
    max_length: usize,
    host_key: LookupBuf,
    shutdown: ShutdownSignal,
    out: Pipeline,
) -> Source {
    build_unix_datagram_source(
        path,
        max_length,
        host_key,
        LinesCodec::new_with_max_length(max_length),
        shutdown,
        out,
        build_event,
    )
}

pub(super) fn unix_stream(
    path: PathBuf,
    max_length: usize,
    host_key: LookupBuf,
    shutdown: ShutdownSignal,
    out: Pipeline,
) -> Source {
    build_unix_stream_source(
        path,
        LinesCodec::new_with_max_length(max_length),
        host_key,
        shutdown,
        out,
        build_event,
    )
}
