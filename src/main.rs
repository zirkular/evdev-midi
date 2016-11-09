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
        1 => {
            println!("No argument provided. Need at least path to device. (/dev/input/event*)");
            return;
        },
        2 => {
            match run(&args.nth(1).unwrap()) {
                Ok(_) => (),
                Err(err) => println!("Error: {}", err.description())
            }
        },
        _ => {
            println!("Too many arguments provided.");
            return;
        }
    }
}

fn run(path: &AsRef<Path>) -> Result<(), Box<Error>> {

    let mut input = String::new();
    let mut converter = core::Converter::new();

    converter.start(path);

    loop {
        input.clear();
        try!(stdin().read_line(&mut input));
        if input.trim() == "q" {
            println!("{:?}", "wtf");

            converter.stop();
            break;
        }
    }
    Ok(())
}
