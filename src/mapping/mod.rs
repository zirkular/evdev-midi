extern crate evdev;
extern crate midir;
extern crate ioctl;

use std;
use std::collections::HashMap;

// ------------------------------------------------------------------------------------------------

pub fn event_to_midi(mapping: &HashMap<u16, u8>, event: &ioctl::input_event,
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

pub fn create() -> HashMap<u16, u8> {

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
