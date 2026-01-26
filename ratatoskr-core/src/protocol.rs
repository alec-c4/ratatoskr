use async_trait::async_trait;
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::request_response::Codec;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Clone)]
pub struct MailboxProtocol();

impl AsRef<str> for MailboxProtocol {
    fn as_ref(&self) -> &str {
        "/ratatoskr/mailbox/1.0.0"
    }
}

#[derive(Clone, Default)]
pub struct MailboxCodec();

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MailboxRequest {
    StoreMessage {
        recipient_did: String,
        message: Vec<u8>, // Encrypted blob (EncryptedMessage serialized)
    },
    FetchMessages {
        owner_did: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MailboxResponse {
    Stored,
    Messages(Vec<Vec<u8>>), // List of encrypted blobs
    NotFound,
    Error(String),
}

#[async_trait]
impl Codec for MailboxCodec {
    type Protocol = MailboxProtocol;
    type Request = MailboxRequest;
    type Response = MailboxResponse;

    async fn read_request<T>(
        &mut self,
        _: &MailboxProtocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut vec = Vec::new();
        io.read_to_end(&mut vec).await?;
        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        serde_json::from_slice(&vec).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _: &MailboxProtocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut vec = Vec::new();
        io.read_to_end(&mut vec).await?;
        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        serde_json::from_slice(&vec).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _: &MailboxProtocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data =
            serde_json::to_vec(&req).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&data).await
    }

    async fn write_response<T>(
        &mut self,
        _: &MailboxProtocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data =
            serde_json::to_vec(&res).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&data).await
    }
}
