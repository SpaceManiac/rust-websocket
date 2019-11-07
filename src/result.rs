//! The result type used within Rust-WebSocket

use crate::server::upgrade::HyperIntoWsError;
pub use hyper::status::StatusCode;
use hyper::Error as HttpError;
use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io;

use url::ParseError;

#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
use native_tls::Error as TlsError;
#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
use native_tls::HandshakeError as TlsHandshakeError;

/// The type used for WebSocket results
pub type WebSocketResult<T> = Result<T, WebSocketError>;

/// This module contains convenience types to make working with Futures and
/// websocket results easier.
#[cfg(feature = "async")]
pub mod r#async {
	use super::WebSocketError;
	use futures::Future;

	/// The most common Future in this library, it is simply some result `I` or
	/// a `WebSocketError`. This is analogous to the `WebSocketResult` type.
	pub type WebSocketFuture<I> = Box<dyn Future<Item = I, Error = WebSocketError> + Send>;
}

pub use websocket_base::result::WebSocketError;

/// Represents a WebSocket error while connecting
#[derive(Debug)]
pub enum WebSocketOtherError {
	/// A WebSocket protocol error
	ProtocolError(&'static str),
	/// Invalid WebSocket request error
	RequestError(&'static str),
	/// Invalid WebSocket response error
	ResponseError(&'static str),
	/// Received unexpected status code
	StatusCodeError(StatusCode),
	/// An HTTP parsing error
	HttpError(HttpError),
	/// A URL parsing error
	UrlError(ParseError),
	/// An input/output error
	IoError(io::Error),
	/// A WebSocket URL error
	WebSocketUrlError(WSUrlErrorKind),
	/// An SSL error
	#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
	TlsError(TlsError),
	/// an ssl handshake failure
	#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
	TlsHandshakeFailure,
	/// an ssl handshake interruption
	#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
	TlsHandshakeInterruption,
}

impl fmt::Display for WebSocketOtherError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.write_str("WebSocketError: ")?;
		fmt.write_str(self.description())?;
		Ok(())
	}
}

impl Error for WebSocketOtherError {
	fn description(&self) -> &str {
		match *self {
			WebSocketOtherError::RequestError(_) => "WebSocket request error",
			WebSocketOtherError::ResponseError(_) => "WebSocket response error",
			WebSocketOtherError::HttpError(_) => "HTTP failure",
			WebSocketOtherError::UrlError(_) => "URL failure",
			#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
			WebSocketOtherError::TlsError(_) => "TLS failure",
			#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
			WebSocketOtherError::TlsHandshakeFailure => "TLS Handshake failure",
			#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
			WebSocketOtherError::TlsHandshakeInterruption => "TLS Handshake interrupted",
			WebSocketOtherError::WebSocketUrlError(_) => "WebSocket URL failure",
			WebSocketOtherError::IoError(ref e) => e.description(),
			WebSocketOtherError::ProtocolError(e) => e,
			WebSocketOtherError::StatusCodeError(_) => "Received unexpected status code",
		}
	}

	fn cause(&self) -> Option<&dyn Error> {
		match *self {
			WebSocketOtherError::HttpError(ref error) => Some(error),
			WebSocketOtherError::UrlError(ref error) => Some(error),
			#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
			WebSocketOtherError::TlsError(ref error) => Some(error),
			WebSocketOtherError::WebSocketUrlError(ref error) => Some(error),
			WebSocketOtherError::IoError(ref e) => Some(e),
			_ => None,
		}
	}
}

impl From<HttpError> for WebSocketOtherError {
	fn from(err: HttpError) -> WebSocketOtherError {
		WebSocketOtherError::HttpError(err)
	}
}

impl From<ParseError> for WebSocketOtherError {
	fn from(err: ParseError) -> WebSocketOtherError {
		WebSocketOtherError::UrlError(err)
	}
}

#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
impl From<TlsError> for WebSocketOtherError {
	fn from(err: TlsError) -> WebSocketOtherError {
		WebSocketOtherError::TlsError(err)
	}
}

#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
impl<T> From<TlsHandshakeError<T>> for WebSocketOtherError {
	fn from(err: TlsHandshakeError<T>) -> WebSocketOtherError {
		match err {
			TlsHandshakeError::Failure(_) => WebSocketOtherError::TlsHandshakeFailure,
			TlsHandshakeError::WouldBlock(_) => WebSocketOtherError::TlsHandshakeInterruption,
		}
	}
}

#[cfg(feature = "async")]
impl From<crate::codec::http::HttpCodecError> for WebSocketOtherError {
	fn from(src: crate::codec::http::HttpCodecError) -> Self {
		match src {
			crate::codec::http::HttpCodecError::Io(e) => WebSocketOtherError::IoError(e),
			crate::codec::http::HttpCodecError::Http(e) => WebSocketOtherError::HttpError(e),
		}
	}
}

impl From<WSUrlErrorKind> for WebSocketOtherError {
	fn from(err: WSUrlErrorKind) -> WebSocketOtherError {
		WebSocketOtherError::WebSocketUrlError(err)
	}
}

impl From<HyperIntoWsError> for WebSocketOtherError {
	fn from(err: HyperIntoWsError) -> WebSocketOtherError {
		use self::HyperIntoWsError::*;
		use self::WebSocketOtherError::*;
		match err {
			Io(io) => IoError(io),
			Parsing(err) => HttpError(err),
			MethodNotGet => ProtocolError("Request method must be GET"),
			UnsupportedHttpVersion => ProtocolError("Unsupported request HTTP version"),
			UnsupportedWebsocketVersion => ProtocolError("Unsupported WebSocket version"),
			NoSecWsKeyHeader => ProtocolError("Missing Sec-WebSocket-Key header"),
			NoWsUpgradeHeader => ProtocolError("Invalid Upgrade WebSocket header"),
			NoUpgradeHeader => ProtocolError("Missing Upgrade WebSocket header"),
			NoWsConnectionHeader => ProtocolError("Invalid Connection WebSocket header"),
			NoConnectionHeader => ProtocolError("Missing Connection WebSocket header"),
		}
	}
}

/// Represents a WebSocket URL error
#[derive(Debug)]
pub enum WSUrlErrorKind {
	/// Fragments are not valid in a WebSocket URL
	CannotSetFragment,
	/// The scheme provided is invalid for a WebSocket
	InvalidScheme,
	/// There is no hostname or IP address to connect to
	NoHostName,
}

impl fmt::Display for WSUrlErrorKind {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.write_str("WebSocket Url Error: ")?;
		fmt.write_str(self.description())?;
		Ok(())
	}
}

impl Error for WSUrlErrorKind {
	fn description(&self) -> &str {
		match *self {
			WSUrlErrorKind::CannotSetFragment => "WebSocket URL cannot set fragment",
			WSUrlErrorKind::InvalidScheme => "WebSocket URL invalid scheme",
			WSUrlErrorKind::NoHostName => "WebSocket URL no host name provided",
		}
	}
}

impl From<WebSocketOtherError> for WebSocketError {
	fn from(e: WebSocketOtherError) -> WebSocketError {
		WebSocketError::Other(Box::new(e))
	}
}

pub(crate) fn towse<E>(e: E) -> WebSocketError
where
	E: Into<WebSocketOtherError>,
{
	let e: WebSocketOtherError = e.into();
	e.into()
}
