// SPDX-License-Identifier: AGPL-3.0-or-later

#![warn(clippy::pedantic)]
#![allow(clippy::no_effect_underscore_binding)]

mod app;

#[rocket::launch]
fn rocket() -> _ {
    app::rocket()
}
