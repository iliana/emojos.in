// Copyright (c) 2022 iliana etaoin
// SPDX-License-Identifier: AGPL-3.0-or-later

#![warn(clippy::pedantic)]
#![allow(
    clippy::map_unwrap_or,
    clippy::needless_pass_by_value,
    clippy::no_effect_underscore_binding
)]

mod app;

use anyhow::{anyhow, Context, Result};
use lambda_http::{http, service_fn, Body};
use rocket::http::Header;
use rocket::local::asynchronous::Client;

fn main() {
    // Rocket needs its own async runtime, but that runtime doesn't support the !Send lambda_http
    // future.
    if std::env::var_os("AWS_LAMBDA_RUNTIME_API").is_some() {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let client = Client::untracked(app::rocket()).await.unwrap();
                lambda_http::run(service_fn(|request| async {
                    Ok(handler(request, &client).await?)
                }))
                .await
                .unwrap();
            });
    } else {
        rocket::execute(async {
            app::rocket().launch().await.ok();
        });
    }
}

async fn handler(request: http::Request<Body>, client: &Client) -> Result<http::Response<Body>> {
    let mut local_request = client.req(
        request
            .method()
            .to_string()
            .parse()
            .map_err(|()| anyhow!("invalid method"))?,
        request
            .uri()
            .path_and_query()
            .context("request has no path")?
            .as_str(),
    );
    for (name, value) in request.headers() {
        local_request.add_header(Header::new(
            name.as_str().to_owned(),
            value.to_str()?.to_owned(),
        ));
    }
    let local_response = local_request.body(request.body()).dispatch().await;

    let mut response = http::Response::builder().status(local_response.status().code);
    for header in local_response.headers().iter() {
        let name: &str = header.name.as_ref();
        let value: &str = header.value.as_ref();
        response = response.header(name, value);
    }
    Ok(response.body(
        local_response
            .into_bytes()
            .await
            .map(Body::Binary)
            .unwrap_or(Body::Empty),
    )?)
}
