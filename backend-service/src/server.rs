use std::convert::Infallible;
use std::sync::Arc;

use chrono::{NaiveDateTime, TimeZone, Utc};
use contact_tracing::DailyTracingKey;
use hyper::{service::make_service_fn, Server};
use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::store::DailyTracingKeyStore;
use crate::utils::{api_reply, response_format};

#[derive(Debug, Clone)]
pub struct BackendState {
    store: Arc<DailyTracingKeyStore>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DailyTracingKeyStoreRequest {
    keys: Vec<(u32, DailyTracingKey)>,
}

pub async fn serve() {
    let backend_state = Arc::new(BackendState {
        store: Arc::new(DailyTracingKeyStore::open("db").unwrap()),
    });

    macro_rules! pass_state {
        () => {
            warp::any().map({
                let backend_state = backend_state.clone();
                move || backend_state.clone()
            })
        };
    }

    let make_svc = make_service_fn(move |_| {
        let fetch = warp::path("fetch")
            .and(warp::path::param())
            .and(pass_state!())
            .map(|ts: u64, state: Arc<BackendState>| {
                let ts = Utc.from_utc_datetime(&NaiveDateTime::from_timestamp(ts as i64, 0));
                state.store.fetch_buckets(ts).unwrap()
            })
            .map(api_reply);

        let submit = warp::path("submit")
            .and(warp::body::json())
            .and(pass_state!())
            .map(
                |data: DailyTracingKeyStoreRequest, state: Arc<BackendState>| {
                    for (day_num, key) in data.keys {
                        state.store.add_daily_tracing_key(day_num, key).unwrap();
                    }
                },
            )
            .map(api_reply);

        let routes = response_format().and(fetch.or(submit));
        let svc = warp::service(routes);
        async move { Ok::<_, Infallible>(svc) }
    });

    let mut listenfd = listenfd::ListenFd::from_env();
    let server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        Server::from_tcp(l).unwrap()
    } else {
        Server::bind(&([127, 0, 0, 1], 5000).into())
    };
    server.serve(make_svc).await.unwrap();
}
