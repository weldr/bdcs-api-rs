/// TOML FromData and Responder for use with Rocket

// Copyright (C) 2016-2017 Red Hat, Inc.
//
// This file is part of bdcs-api-server.
//
// bdcs-api-server is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// bdcs-api-server is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with bdcs-api-server.  If not, see <http://www.gnu.org/licenses/>.

use std::ops::{Deref, DerefMut};
use std::fs::File;
use std::io::Read;

use rocket::data::{self, Data, FromData};
use rocket::http::{Status, ContentType};
use rocket::outcome::Outcome;
use rocket::request::Request;
use rocket::response::{self, Responder};
use rocket::response::content::Content;
use serde::{Serialize, Deserialize};
use toml;
use toml::de::Error as SerdeError;


/// TOML FromData implementation allowing TOML data to be used directly in a POST
/// with Rocket.
///
/// ```rust,ignore
/// #[post("/document/", data="<doc>")]
/// fn new_document(doc: TOML<Document>) {
///     ...
/// }
/// ```
#[derive(Debug)]
pub struct TOML<T>(pub T);

impl<T> TOML<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Deserialize> FromData for TOML<T> {
    type Error = SerdeError;

    fn from_data(request: &Request, data: Data) -> data::Outcome<Self, SerdeError> {
        let x_toml = ContentType::new("text", "x-toml");
        if !request.content_type().map_or(false, |ct| ct == x_toml) {
            error!("Content-Type is not TOML");
            return Outcome::Forward(data);
        }

        let mut input = String::new();
        let _ = data.open().read_to_string(&mut input);
        match toml::from_str(&input).map(|val| TOML(val)) {
            Ok(value) => Outcome::Success(value),
            Err(e) => {
                error!("Couldn't parse TOML body: {:?}", e);
                Outcome::Failure((Status::BadRequest, e))
            }
        }
    }
}

// Serializes the wrapped value into TOML. Returns a response with Content-Type
// text/x-toml and a fixed-size body with the serialization. If serialization fails, an
// `Err` of `Status::InternalServerError` is returned.
impl<T: Serialize> Responder<'static> for TOML<T> {
    fn respond(self) -> response::Result<'static> {
        let x_toml = ContentType::new("text", "x-toml");

        toml::Value::try_from(&self.0).map(|value| {
            Content(x_toml, value.to_string()).respond().unwrap()
        }).map_err(|e| {
            error!("TOML failed to serialize: {:?}", e);
            Status::InternalServerError
        })
    }
}

impl<T> Deref for TOML<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        &self.0
    }
}

impl<T> DerefMut for TOML<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.0
    }
}
