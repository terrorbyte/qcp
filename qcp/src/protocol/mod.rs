// qcp::protocol

pub mod control;
pub mod session;

use tokio_util::compat::{
    Compat as tokCompat, TokioAsyncReadCompatExt as _, TokioAsyncWriteCompatExt as _,
};

pub type RawStreamPair = (quinn::SendStream, quinn::RecvStream);

/// Syntactic sugar type (though I expect some might call it salt)
pub struct StreamPair {
    pub send: tokCompat<quinn::SendStream>,
    pub recv: tokCompat<quinn::RecvStream>,
    // The underlying Send/Recv stream objects have Drop handlers which do the Right Thing.
}

impl From<RawStreamPair> for StreamPair {
    fn from(value: RawStreamPair) -> Self {
        Self {
            send: value.0.compat_write(),
            recv: value.1.compat(),
        }
    }
}
