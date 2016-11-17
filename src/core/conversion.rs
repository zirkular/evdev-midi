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

use ioctl;
use std::collections::HashMap;

const EV_BUTTON: u16    = 1;
const EV_ROTARY: u16    = 3;

pub fn event_to_midi(mapping: &HashMap<u16, u8>, event: &ioctl::input_event,
    midi_msg: &mut [u8; 3]) -> Result<(), &'static str> {

    midi_msg[0] = match event._type {
        EV_BUTTON =>  match event.value {
            0 => super::midi::COMMAND_NOTE_OFF,
            _ => super::midi::COMMAND_NOTE_ON,
        },
        EV_ROTARY => super::midi::COMMAND_CC,
        _ => return Err("Invalid event type.")
    };

    midi_msg[1] = match mapping.get(&event.code) {
        Some(byte_2) => *byte_2,
        None => return Err("Invalid event code.")
    };

    midi_msg[2] = match event._type {
        EV_BUTTON => match event.value {
            0 => super::midi::VELOCITY_MIN,
            _ => super::midi::VELOCITY_MAX,
        },
        EV_ROTARY => convert_range_value(event.value as f32),
        _ => return Err("Invalid event value.")
    };
    Ok(())
}

fn convert_range_value(value: f32) -> u8 {
    let slope = (127_f32 - 0_f32) / (4095_f32 - 0_f32);
    let output = 0_f32 + slope * (value - 0_f32);

    return output as u8;
}
