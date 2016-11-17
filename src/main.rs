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

extern crate evdev;
extern crate midir;
extern crate ioctl;
extern crate rustc_serialize;

use std::error::Error;
use std::io::{stdin};
use std::path::Path;

mod core;

fn main() {
    let mut args = std::env::args_os();

    match args.len() {
        3 => {
            match run(&args.nth(1).unwrap(), &args.nth(0).unwrap()) {
                Ok(_) => (),
                Err(err) => println!("Error: {}", err.description())
            }
        },
        _ => {
            println!("Wrong number of arguments provided. Example usage: evdev-midi /dev/input/event1 mapping.json");
            return;
        }
    }
}

fn run(device_path: &AsRef<Path>, mapping_path: &AsRef<Path>) -> Result<(), Box<Error>> {
    let mut input       = String::new();
    let mut converter   = core::Converter::new();
    let mapping         = core::mapping::load(mapping_path).unwrap();

    println!("{:?}", device_path.as_ref());

    if let Ok(device) = evdev::Device::open(&device_path) {
        converter.start(device, mapping);
        loop {
            input.clear();
            try!(stdin().read_line(&mut input));
            if input.trim() == "q" {
                println!("{:?}", "wtf");
                converter.stop();
                break;
            }
        }
    } else {
        println!("Could not open device!");
    }

    Ok(())
}
