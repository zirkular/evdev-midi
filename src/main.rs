/**
 * This file is part of honeycomb.
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

use std::error::Error;
use std::io::{stdin};
use std::path::Path;

mod mapping;

fn main() {

    let mut args = std::env::args_os();

    // Check command line arguments. The first argument is the binary itself, so the 1st arguemnt
    // is the path we're lokking for: E.g. /dev/input/event*
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

    let mut input   = String::new();
    let mut converter   = mapping::Converter::new();

    // Start the conversion thread which reads input events and send MIDI messages
    converter.start(path);

    // Main loop: only accept 'q' as user input to terminate correctly
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
