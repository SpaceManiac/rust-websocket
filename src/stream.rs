//! Provides the default stream type for WebSocket connections.
extern crate net2;

use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use self::net2::TcpStreamExt;
use openssl::ssl::SslStream;

pub use std::net::{SocketAddr, Shutdown, TcpStream};

/// A useful stream type for carrying WebSocket connections.
pub enum WebSocketStream {
	/// A TCP stream.
	Tcp(TcpStream),
	/// An SSL-backed TCP Stream
	Ssl(Arc<Mutex<SslStream<TcpStream>>>)
}

impl Read for WebSocketStream {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.read(buf),
			WebSocketStream::Ssl(ref mut inner) => inner.lock().expect("lock").read(buf),
		}
	}
}

impl Write for WebSocketStream {
	fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.write(msg),
			WebSocketStream::Ssl(ref mut inner) => inner.lock().expect("lock").write(msg),
		}
	}

	fn flush(&mut self) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.flush(),
			WebSocketStream::Ssl(ref mut inner) => inner.lock().expect("lock").flush(),
		}
	}
}

impl WebSocketStream {
	/// See `TcpStream.peer_addr()`.
	pub fn peer_addr(&self) -> io::Result<SocketAddr> {
		match *self {
			WebSocketStream::Tcp(ref inner) => inner.peer_addr(),
			WebSocketStream::Ssl(ref inner) => inner.lock().expect("lock").get_ref().peer_addr(),
		}
	}
	/// See `TcpStream.local_addr()`.
	pub fn local_addr(&self) -> io::Result<SocketAddr> {
		match *self {
			WebSocketStream::Tcp(ref inner) => inner.local_addr(),
			WebSocketStream::Ssl(ref inner) => inner.lock().expect("lock").get_ref().local_addr(),
		}
	}
	/// See `TcpStream.set_nodelay()`.
	pub fn set_nodelay(&mut self, nodelay: bool) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => TcpStreamExt::set_nodelay(inner, nodelay),
			WebSocketStream::Ssl(ref mut inner) => {
				let mut inner = inner.lock().expect("lock");
				TcpStreamExt::set_nodelay(inner.get_mut(), nodelay)
			},
		}
	}
	/// See `TcpStream.set_keepalive()`.
	pub fn set_keepalive(&mut self, delay_in_ms: Option<u32>) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => TcpStreamExt::set_keepalive_ms(inner, delay_in_ms),
			WebSocketStream::Ssl(ref mut inner) => {
				let mut inner = inner.lock().expect("lock");
				TcpStreamExt::set_keepalive_ms(inner.get_mut(), delay_in_ms)
			},
		}
	}
	/// See `TcpStream.shutdown()`.
	pub fn shutdown(&mut self, shutdown: Shutdown) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.shutdown(shutdown),
			WebSocketStream::Ssl(ref mut inner) => {
				let mut inner = inner.lock().expect("lock");
				inner.get_mut().shutdown(shutdown)
			},
		}
	}
	/// See `TcpStream.try_clone()`.
	pub fn try_clone(&self) -> io::Result<WebSocketStream> {
		Ok(match *self {
			WebSocketStream::Tcp(ref inner) => WebSocketStream::Tcp(try!(inner.try_clone())),
			WebSocketStream::Ssl(ref arc) => WebSocketStream::Ssl(arc.clone()),
		})
	}

    /// Changes whether the stream is in nonblocking mode.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        match *self {
			WebSocketStream::Tcp(ref inner) => inner.set_nonblocking(nonblocking),
			WebSocketStream::Ssl(ref inner) => inner.lock().expect("lock").get_ref().set_nonblocking(nonblocking),
        }
    }
}
