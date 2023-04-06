// SPDX-License-Identifier: AGPL-3.0-or-later

#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value, clippy::no_effect_underscore_binding)]

mod trivial;

use askama::Template;
use reqwest::{Client, StatusCode, Url};
use rocket::http::{ContentType, Header, Status};
use rocket::response::{self, Debug, Responder};
use rocket::{get, routes, Request, Response, State};
use serde::Deserialize;
use std::io::Cursor;
use std::str::FromStr;

#[rocket::launch]
fn rocket() -> _ {
    let client = Client::builder()
        .user_agent(format!(
            "emojos.in/{} (+https://github.com/iliana/emojos.in)",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .unwrap();

    rocket::build().manage(client).mount(
        "/",
        routes![
            instance,
            trivial::code,
            trivial::copy_js,
            trivial::css,
            trivial::favicon_ico,
            trivial::index,
            trivial::instance_form,
            trivial::robots_txt,
        ],
    )
}

#[derive(Debug)]
struct Html<T: Template>(T);

impl<T: Template> Responder<'_, 'static> for Html<T> {
    fn respond_to(self, _request: &Request<'_>) -> response::Result<'static> {
        let data = self.0.render().map_err(|_| Status::InternalServerError)?;
        Response::build()
            .header(Header::new("content-type", T::MIME_TYPE))
            .sized_body(data.len(), Cursor::new(data))
            .ok()
    }
}

#[get("/<instance>?<show_all>&<show_animated>")]
async fn instance(
    client: &State<Client>,
    instance: &str,
    show_all: Option<bool>,
    show_animated: Option<bool>,
) -> Result<(ContentType, String), Debug<anyhow::Error>> {
    #[derive(Deserialize)]
    struct Emojo {
        shortcode: String,
        url: String,
        static_url: String,
        visible_in_picker: Option<bool>,
    }

    #[derive(Template)]
    #[template(path = "emojo.html")]
    struct Output<'a> {
        instance: &'a str,
        show_animated: bool,
        emojo: Vec<Emojo>,
    }

    #[derive(Template)]
    #[template(path = "oh_no.html")]
    struct OhNo<'a> {
        instance: &'a str,
        status: StatusCode,
        why: &'a str,
    }

    let show_all = show_all.unwrap_or_default();
    let show_animated = show_animated.unwrap_or_default();

    let output = async {
        let mut url = Url::from_str("https://host.invalid/api/v1/custom_emojis").unwrap();
        url.set_host(Some(instance))?;

        let response = client.get(url).send().await?;
        if response.status().is_client_error() || response.status().is_server_error() {
            return Ok(OhNo {
                instance,
                status: response.status(),
                why: match response.status() {
                    StatusCode::FORBIDDEN => "This instance's emoji list is private.",
                    StatusCode::NOT_FOUND => {
                        "This instance doesn't support the Mastodon custom emoji API."
                    }
                    _ => "That's all we know.",
                },
            }
            .render()?);
        }

        let mut emojo: Vec<Emojo> = response.json().await?;
        if !show_all {
            emojo.retain(|x| x.visible_in_picker.unwrap_or(true));
        }

        anyhow::Ok(
            Output {
                instance,
                show_animated,
                emojo,
            }
            .render()?,
        )
    }
    .await?;
    Ok((ContentType::HTML, output))
}
