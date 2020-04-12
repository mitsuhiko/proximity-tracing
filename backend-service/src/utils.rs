use std::cell::Cell;

use http::header::{HeaderValue, CONTENT_TYPE};
use http::{Response, StatusCode};
use hyper::Body;
use serde::Serialize;
use warp::{Filter, Reply};

thread_local! {
    static RESPONSE_FORMAT: Cell<ResponseFormat> = Cell::new(ResponseFormat::Json);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResponseFormat {
    Json,
    Cbor,
}

pub fn response_format() -> impl Filter<Extract = (), Error = warp::Rejection> + Copy {
    warp::header("accept")
        .map(|value: String| {
            RESPONSE_FORMAT.with(|format| {
                format.set(if value.eq_ignore_ascii_case("application/x-cbor") {
                    ResponseFormat::Cbor
                } else {
                    ResponseFormat::Json
                });
            });
        })
        .untuple_one()
}

pub fn api_reply<T>(val: T) -> ApiReply
where
    T: Serialize,
{
    ApiReply {
        inner: match RESPONSE_FORMAT.with(|x| x.get()) {
            ResponseFormat::Json => ::serde_json::to_vec(&val).map_err(|err| {
                log::error!("Invalid json serialization: {:?}", err);
                ();
            }),
            ResponseFormat::Cbor => ::serde_cbor::to_vec(&val).map_err(|err| {
                log::error!("Invalid cbor serialization: {:?}", err);
                ();
            }),
        },
    }
}

/// An API response.
pub struct ApiReply {
    inner: Result<Vec<u8>, ()>,
}

impl Reply for ApiReply {
    #[inline]
    fn into_response(self) -> Response<Body> {
        match self.inner {
            Ok(body) => {
                let mut res = Response::new(body.into());
                res.headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                res
            }
            Err(()) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
