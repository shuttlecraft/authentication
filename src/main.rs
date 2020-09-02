// Copyright (c) 2020 Aravinth T M <realaravinth@batsense.net>.
// See the COPYRIGHT file at the top-level directory of this
// distribution

//This program is free software; you can redistribute it and/or
//modify it under the terms of the GNU General Public License
//as published by the Free Software Foundation; either version 2
//of the License, or (at your option) any later version.

//This program is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU General Public License for more details.

//You should have received a copy of the GNU General Public License
//along with this program; if not, write to the Free Software
//Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301, USA.

extern crate argon2;
extern crate config;
extern crate regex;
extern crate unicode_normalization;
#[macro_use]
extern crate diesel;
extern crate env_logger;
extern crate num_cpus;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

use actix_http::cookie::SameSite;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_session::CookieSession;
use actix_web::{
    middleware::{Compress, Logger},
    App, HttpServer,
};

use regex::Regex;
use std::env;

mod database;
mod errors;
mod pow;
mod schema;
mod settings;
mod users;

use crate::users::filters::blacklist::tables::BLACKLIST;
use crate::users::filters::profainity::tables::PROFAINITY;
use crate::users::filters::user_case_mapped::tables::USERNAME_CASE_MAPPED;

use database::pool::get_connection_pool;
use settings::Settings;
use users::server;

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new().expect("couldn't load settings");
    pub static ref RE_BLACKLIST: Regex =
        Regex::new(BLACKLIST).expect("couldn't setup blacklist list filter");
    pub static ref RE_PROFAINITY: Regex =
        Regex::new(PROFAINITY).expect("coudln't setup profainity filter");
    pub static ref RE_USERNAME_CASE_MAPPED: Regex =
        Regex::new(USERNAME_CASE_MAPPED).expect("coudln't setup username case mapped filter");
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let cookie_secret = &SETTINGS.server.cookie_secret;

    let database_connection_pool = get_connection_pool(&SETTINGS.database.url);

    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .wrap(
                CookieSession::signed(&cookie_secret.as_bytes())
                    .domain(&SETTINGS.server.domain)
                    .name("shuttlecraft-session")
                    .path("/")
                    .secure(false),
            )
            .wrap(
                CookieSession::signed(&cookie_secret.as_bytes())
                    .domain(&SETTINGS.server.domain)
                    .name("on")
                    .path("/")
                    .secure(false),
            )
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(cookie_secret.as_bytes())
                    .name("Authorization")
                    .max_age(20)
                    .domain(&SETTINGS.server.domain)
                    .same_site(SameSite::Lax)
                    .secure(true),
            ))
            .configure(server::config)
            .wrap(Logger::default())
            .data(database_connection_pool.clone())
    })
    .bind(format!(
        "{}:{}",
        &SETTINGS.server.host, &SETTINGS.server.port
    ))
    .expect(&format!(
        "Couldn't bind to IP address: {} and port: {}, are they avaiable?",
        &SETTINGS.server.host, &SETTINGS.server.port
    ))
    .run()
    .await
}
