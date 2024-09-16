// qcp::protocol

pub mod control;
pub mod session;

use tokio_util::compat::{
    Compat as tokCompat, TokioAsyncReadCompatExt as _, TokioAsyncWriteCompatExt as _,
};

/// Syntactic sugar type (though I expect some might call it salt)
pub struct StreamPair {
    pub send: tokCompat<quinn::SendStream>,
    pub recv: tokCompat<quinn::RecvStream>,
    // The underlying Send/Recv stream objects have Drop handlers which do the Right Thing.
}

impl From<(quinn::SendStream, quinn::RecvStream)> for StreamPair {
    fn from(value: (quinn::SendStream, quinn::RecvStream)) -> Self {
        Self {
            send: value.0.compat_write(),
            recv: value.1.compat(),
        }
    }
}
