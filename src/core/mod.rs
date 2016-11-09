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

use std;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

mod midi;
mod mapping;
mod conversion;

pub struct Converter {
    running:    std::sync::Arc<AtomicBool>,
    worker:     Option<std::thread::JoinHandle<std::result::Result<(), midir::InitError>>>
}

impl Converter {

    pub fn new() -> Converter {
        return Converter { running: Arc::new(AtomicBool::new(true)), worker: None };
    }

    pub fn start(&mut self, device_path: &AsRef<Path>, mapping_path: &AsRef<Path>) {
        let mut device      = evdev::Device::open(&device_path).unwrap();
        let mapping         = mapping::load(mapping_path).unwrap();
        let running_clone   = self.running.clone();

        // Spawn thread which polls input events, converts and sends them via MIDI
        self.worker = Some(thread::spawn(move || {
            let mut transmitter  = match midi::Transmitter::new(&String::from("evdev-midi-controller")) {
                Ok(tx) => Arc::new(tx),
                Err(err) => {
                    println!("Could not create MIDI transmitter: {:?}", err);
                    return Err(err)
                },
            };
            println!("Polling new events. Press q to quit!");

            let mut message: [u8; 3] = [0, 0, 0];
            loop {
                if running_clone.load(Ordering::SeqCst) {

                    // Get all new input events and filter out the ones with type 0.
                    // The filter can be seen as a workaround because the root cause of these 'false'
                    // events has to be found in the evdev.
                    for event in device.events_no_sync().unwrap().filter(|event| event._type != 0) {
                        match conversion::event_to_midi(&mapping.map, &event, &mut message) {
                            Ok(()) => match Arc::get_mut(&mut transmitter).unwrap().send(&message) {
                                Ok(_) => (),
                                Err(err) => println!("Could not send event. {:?}! Event: {:?}", err, event),
                            },
                            Err(err) => println!("Could not convert event. {:?}! Event: {:?}", err, event),
                        };
                    }
                    thread::sleep(Duration::from_millis(5));
                } else {
                    break;
                }
            }
            Ok(())
        }));
    }

    pub fn stop(self) {
        self.running.store(false, Ordering::SeqCst);

        match self.worker.unwrap().join() {
            Ok(_) => (),
            Err(err) => println!("Failed to join worker thread: {:?}!", err),
        }
    }
}
