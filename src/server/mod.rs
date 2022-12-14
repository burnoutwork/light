use std::fmt::{Debug, Formatter};
use tokio::io;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

const VERSION: u8 = 0x05;

pub struct ProxyListener {
    tcp_listener: TcpListener
}

impl ProxyListener {
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        TcpListener::bind(addr).await.map(|tcp_listener| Self { tcp_listener })
    }

    pub async fn run(&mut self) -> io::Result<()> {
        loop {
            let (mut socket, _) = self.tcp_listener.accept().await?;
            let proxy_stream = ProxyStream::init(&mut socket).await;
        }
    }
}

struct ProxyStream {}

impl ProxyStream {
    async fn init(socket: &mut TcpStream) -> io::Result<Self> {
        let hello = packages::Hello::from_io(socket).await?;
        println!("Auth: {:?}", hello.auth_methods());

        Ok(Self {})
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AuthMethods {
    NoAuth,
    GSSAPI,
    UsernamePass,
    IANA,
    Private,
}

impl From<u8> for AuthMethods {
    fn from(method: u8) -> Self {
        if method >= 0x03 && method <= 0x7F {
            return Self::IANA
        }

        if method >= 0x80 && method <= 0xFE {
            return Self::Private
        }

        match method {
            0x00 => Self::NoAuth,
            0x01 => Self::GSSAPI,
            0x02 => Self::UsernamePass,
            _ => unreachable!()
        }
    }
}

mod packages {
    use tokio::io;
    use tokio::io::{AsyncRead, AsyncReadExt};
    use crate::server::AuthMethods;

    pub(super) struct Hello {
        version: u8,
        auth_methods: Vec<AuthMethods>
    }

    impl Hello {
        pub(super) fn new(version: u8, auth_methods: Vec<AuthMethods>) -> Self {
            Self { version, auth_methods }
        }

        pub(super) fn version(&self) -> u8 {
            self.version
        }


        pub(super) fn auth_methods(&self) -> Vec<AuthMethods> {
            self.auth_methods.clone()
        }

        pub(super) async fn from_io<R: AsyncRead + std::marker::Unpin>(reader: &mut R) -> io::Result<Self> {
            let version = reader.read_u8().await?;
            let len_methods = reader.read_u8().await?;

            if len_methods == 0 {
                return Err(io::Error::new(io::ErrorKind::Other, "Length methods equal zero"))
            }

            let mut auth_methods = Vec::new();

            for _ in 0..len_methods {
                auth_methods.push(AuthMethods::from(reader.read_u8().await?))
            }

            Ok(Self::new(version, auth_methods))
        }
    }
}