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

use std::io::{stdin};
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::path::Path;

use midir::MidiOutput;

mod input;
mod mapping;

// ------------------------------------------------------------------------------------------------

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

// ------------------------------------------------------------------------------------------------

fn run(path: &AsRef<Path>) -> Result<(), Box<Error>> {

    let spinlock    = Arc::new(AtomicBool::new(true));
    let mut input   = String::new();

    let mut device = evdev::Device::open(&path).unwrap();

    // Worker thread: poll input events and send corresponding MIDI message
    let spinlock_clone = spinlock.clone();
    let child = thread::spawn(move || {

        let mapping = mapping::create();
        let mut midi_msg: [u8; 3] = [0, 0, 0];

        let midi_out = match MidiOutput::new("honeycomb") {
            Ok(midi_out) => midi_out,
            Err(err) => {
                println!("{:?}", err);
                return;
            },
        };

        let mut conn_out = match midi_out.connect(0, "TRAKTOR Kontrol X1") {
            Ok(conn_out) => conn_out,
            Err(err) => {
                println!("{:?}", err);
                return;
            },
        };

        println!("Polling new events. Press q to quit!");

        loop {
            if spinlock_clone.load(Ordering::SeqCst) {

                // Get all new input events and filter out the ones with type 0.
                // The filter can be seen as a workaround because the root cause of these 'false'
                // events has to be found in the evdev.
                for ev in device.events_no_sync().unwrap().filter(|ev| ev._type != 0) {

                    // Convert input event to MIDI byte array and try to send it.
                    // Rust feature: Be explicit about mutability :)
                    match mapping::event_to_midi(&mapping, &ev, &mut midi_msg) {
                        Ok(()) => {
                            match conn_out.send(&midi_msg) {
                                Ok(()) => {},
                                Err(err) => {
                                    println!("Could not send MIDI message. {:?}", err);
                                    break;
                                }
                            }
                        },
                        Err(err) => {
                            println!("Could not convert event. {:?}! Event: {:?}", err, ev);
                            break;
                        },
                    };

                    // println!("Converted event [code: {:?}, type: {:?}, value: {:?}] to \
                    //                      MIDI [command: {:?}, byte 1: {:?}, byte 2: {:?}]",
                    //                      ev.code, ev._type, ev.value,
                    //                      midi_msg[0], midi_msg[1], midi_msg[2]);

                }
                thread::sleep(Duration::from_millis(5));
            } else {
                break;
            }
        }
        conn_out.close();
    });

    // Main loop: only accept 'q' as user input to terminate correctly
    loop {
        input.clear();
        try!(stdin().read_line(&mut input));
        if input.trim() == "q" {
            println!("{:?}", "wtf");
            spinlock.store(false, Ordering::SeqCst);
            break;
        }
    }

    child.join();

    Ok(())
}
