use askama::Template;
use reqwest::{Client, StatusCode, Url};
use rocket::form::{Form, FromForm};
use rocket::http::{ContentType, Header};
use rocket::response::status::NoContent;
use rocket::response::{Debug, Redirect, Responder};
use rocket::{get, post, routes, uri, Build, Rocket, State};
use serde::Deserialize;
use std::str::FromStr;

pub fn rocket() -> Rocket<Build> {
    rocket::build()
        .manage(
            Client::builder()
                .user_agent(format!(
                    "emojos.in/{} (+https://github.com/iliana/emojos.in)",
                    env!("CARGO_PKG_VERSION")
                ))
                .build()
                .unwrap(),
        )
        .mount(
            "/",
            routes![
                code,
                copy_js,
                css,
                favicon_ico,
                index,
                instance,
                instance_form,
                robots_txt
            ],
        )
}

#[get("/")]
fn index() -> Result<(ContentType, String), Debug<askama::Error>> {
    #[derive(Template)]
    #[template(path = "index.html")]
    struct Index;

    Ok((ContentType::HTML, Index.render()?))
}

#[derive(FromForm)]
struct InstanceForm<'a> {
    instance: &'a str,
    show_all: bool,
    show_animated: bool,
}

#[post("/", data = "<form>")]
fn instance_form(form: Form<InstanceForm<'_>>) -> Redirect {
    Redirect::to(uri!(instance(
        form.instance,
        form.show_all.then_some(true),
        form.show_animated.then_some(true),
    )))
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
        visible_in_picker: bool,
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
            emojo.retain(|x| x.visible_in_picker);
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

#[derive(Responder)]
struct Code {
    zip: &'static [u8],
    content_type: ContentType,
    disposition: Header<'static>,
}

#[get("/code")]
fn code() -> Option<Code> {
    let zip = include!(concat!(env!("OUT_DIR"), "/zip.rs"))?;
    Some(Code {
        zip,
        content_type: ContentType::ZIP,
        disposition: Header::new(
            "content-disposition",
            r#"attachment; filename="emojos.in.zip""#,
        ),
    })
}

#[get("/static/site.css")]
fn css() -> (ContentType, &'static str) {
    (
        ContentType::CSS,
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/site.css")),
    )
}

#[get("/static/copy.js")]
fn copy_js() -> (ContentType, &'static str) {
    (
        ContentType::JavaScript,
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/copy.js")),
    )
}

#[get("/favicon.ico")]
fn favicon_ico() -> NoContent {
    NoContent
}

#[get("/robots.txt")]
fn robots_txt() -> NoContent {
    NoContent
}
