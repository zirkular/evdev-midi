extern crate evdev;
extern crate midir;
extern crate ioctl;

use std::io::{stdin, Write};
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use ioctl::input_event;
use midir::MidiOutput;

use std::collections::HashMap;

// ------------------------------------------------------------------------------------------------
/*
pub struct input_event {
    pub time: ::libc::timeval,
    pub _type: u16,
    pub code: u16,
    pub value: i32,
}
*/

enum NoteType {
    On,
    Off,
}

struct MIDIMessage {
    note: u8,
    state: NoteType
}

// ------------------------------------------------------------------------------------------------

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err.description())
    }
}

// ------------------------------------------------------------------------------------------------

fn run() -> Result<(), Box<Error>> {
    let mut args = std::env::args_os();
    let mut input = String::new();

    let spinlock = Arc::new(AtomicBool::new(true));

    // evdev
    let mut device;
    if args.len() > 1 {
        device = evdev::Device::open(&args.nth(1).unwrap()).unwrap();
    } else {
        let mut devices = evdev::enumerate();
        for (i, d) in devices.iter().enumerate() {
            println!("{}: {:?}", i, d.name());
        }
        print!("Select the device [0-{}]: ", devices.len());
        let _ = std::io::stdout().flush();
        let mut chosen = String::new();
        std::io::stdin().read_line(&mut chosen).unwrap();
        device = devices.swap_remove(chosen.trim().parse::<usize>().unwrap());
    }

    println!("Press q to quit!");

    // Worker thread: poll input events and send corresponding MIDI message
    let spinlock_clone = spinlock.clone();
    let child = thread::spawn(move || {

        let mapping = create_mapping();
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

        loop {
            if spinlock_clone.load(Ordering::SeqCst) {

                // Get all new input events and filter out the ones with type 0.
                // The filter can be seen as a workaround because the root cause of these 'false'
                // events has to be found.
                for ev in device.events_no_sync().unwrap().filter(|ev| ev._type != 0) {

                    // Convert input event to MIDI byte array and try to send it.
                    match event_to_midi(&mapping, &ev, &mut midi_msg) {
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

    // match child.join() {
    //     Ok(()) => Ok(()),
    //     Err(err) => Err(Box::new("err"))
    // }
}

// ------------------------------------------------------------------------------------------------

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

// ------------------------------------------------------------------------------------------------

fn convert_range_value(value: f32) -> u8 {

    let slope = (127_f32 - 0_f32) / (4095_f32 - 0_f32);
    let output = 0_f32 + slope * (value - 0_f32);

    return output as u8;
}

// ------------------------------------------------------------------------------------------------

fn create_mapping() -> HashMap<u16, u8> {

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
