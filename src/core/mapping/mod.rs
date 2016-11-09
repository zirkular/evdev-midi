/**
 * This file is part of evdev-midi.
 *
 * Copyright (C) 2016 by Erik Kundt <bitshift@posteo.org>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.

 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.

 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

extern crate rustc_serialize;

use std::collections::HashMap;
use std::io::prelude::*;
use std::io::Error;
use std::path::Path;
use std::fs::File;

use rustc_serialize::json;


#[derive(RustcDecodable, RustcEncodable)]
pub struct Mapping {
    pub device_name: String,
    pub map: HashMap<u16, u8>,
}

pub fn load(path: &AsRef<Path>) -> Result<Mapping, Error> {
    let mut file = try!(File::open(path));
    let mut content = String::new();
    try!(file.read_to_string(&mut content));

    let mapping: Mapping = json::decode(&content).unwrap();
    Ok(mapping)
}

#[test]
fn encode_test() {
    let mut mapping = Mapping {
        device_name: String::from("Test"),
        map: HashMap::new(),
    };
    mapping.map.insert(1, 1);

    assert_eq!(json::encode(&mapping).unwrap(), "{\"device_name\":\"Test\",\"map\":{\"1\":1}}");
}

#[test]
fn load_test() {
    let mapping = load(&Path::new("resources/mappings/test.json")).unwrap();

    assert_eq!(mapping.device_name, "Test");
}
