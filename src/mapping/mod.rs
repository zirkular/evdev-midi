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

use std;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

mod midi;

pub struct Converter {
    running:    std::sync::Arc<AtomicBool>,
    worker:     Option<std::thread::JoinHandle<std::result::Result<(), midir::InitError>>>
}

impl Converter {
    pub fn new() -> Converter {
        return Converter { running: Arc::new(AtomicBool::new(true)), worker: None };
    }

    pub fn start(&mut self, path: &AsRef<Path>) {

        let mut device      = evdev::Device::open(&path).unwrap();
        let running_clone   = self.running.clone();

        // Spawn thread which polls input events, converts and sends them via MIDI
        self.worker = Some(thread::spawn(move || {

            let mut transmitter  = match midi::Transmitter::new() {
                Ok(tx) => Arc::new(tx),
                Err(err) => {
                    println!("Could not create MIDI transmitter: {:?}", err);
                    return Err(err)
                },
            };

            let mapping = create();
            let mut message: [u8; 3] = [0, 0, 0];

            println!("Polling new events. Press q to quit!");

            loop {
                if running_clone.load(Ordering::SeqCst) {

                    // Get all new input events and filter out the ones with type 0.
                    // The filter can be seen as a workaround because the root cause of these 'false'
                    // events has to be found in the evdev.
                    for event in device.events_no_sync().unwrap().filter(|event| event._type != 0) {

                        match event_to_midi(&mapping, &event, &mut message) {
                            Ok(()) => match Arc::get_mut(&mut transmitter) {
                                Some(transmitter) => match transmitter.send(&message) {
                                    Ok(_) => (),
                                    Err(err) => println!("Could not send event. {:?}! Event: {:?}", err, event),
                                },
                                None => println!("Tx has more than one strong and / or weak references"),
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

fn event_to_midi(mapping: &HashMap<u16, u8>, event: &ioctl::input_event,
    midi_msg: &mut [u8; 3]) -> Result<(), &'static str> {

    midi_msg[0] = match event._type {
        1 =>  match event.value {
            0 => 128,
            _ => 144,
        },
        3 => 176,
        _ => return Err("Invalid event type.")
    };

    midi_msg[1] = match mapping.get(&event.code) {
        Some(byte_2) => *byte_2,
        None => return Err("Invalid event code.")
    };

    midi_msg[2] = match event._type {
        1 => match event.value {
            0 => 0,
            _ => 127,
        },
        3 => convert_range_value(event.value as f32),
        _ => return Err("Invalid event value.")
    };
    Ok(())
}

fn convert_range_value(value: f32) -> u8 {

    let slope = (127_f32 - 0_f32) / (4095_f32 - 0_f32);
    let output = 0_f32 + slope * (value - 0_f32);

    return output as u8;
}

fn create() -> HashMap<u16, u8> {

    let mut mapping = HashMap::new();

    // Creates mapping for notes, cc's

    // TRAKTOR Kontrol X1 Mk1 (bottom-up, l-to-r)
    // Transport Deck A
    mapping.insert(256, 1);
    mapping.insert(279, 2);
    mapping.insert(257, 3);
    mapping.insert(278, 4);
    mapping.insert(258, 5);
    mapping.insert(277, 6);
    mapping.insert(276, 7);
    mapping.insert(259, 8);

    // Loop / Load Deck A
    mapping.insert(282, 9);
    mapping.insert(2,   10);
    mapping.insert(265, 11);
    mapping.insert(264, 12);
    mapping.insert(280, 13);
    // mapping.insert(?, 14); // encoder dead

    // Effects Deck A
    mapping.insert(287, 15);
    mapping.insert(22,  16);
    mapping.insert(286, 17);
    mapping.insert(20,  18);
    mapping.insert(285, 19);
    mapping.insert(18,  20);
    mapping.insert(284, 21);
    mapping.insert(16,  22);

    // Transport Deck B
    mapping.insert(272, 23);
    mapping.insert(271, 24);
    mapping.insert(273, 25);
    mapping.insert(270, 26);
    mapping.insert(274, 27);
    mapping.insert(269, 28);
    mapping.insert(268, 29);
    mapping.insert(275, 30);

    // Loop / Load Deck B
    mapping.insert(283, 31);
    mapping.insert(40,  32);
    mapping.insert(294, 33);
    mapping.insert(293, 34);
    mapping.insert(281, 35);
    mapping.insert(1,   36);

    // Effects Deck B
    mapping.insert(23,  37);
    mapping.insert(291, 38);
    mapping.insert(21,  39);
    mapping.insert(290, 40);
    mapping.insert(19,  41);
    mapping.insert(289, 42);
    mapping.insert(17,  43);
    mapping.insert(288, 44);

    // Shift / Hotcue
    mapping.insert(295, 45);
    mapping.insert(292, 46);

    return mapping;
}
