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

extern crate midir;

use midir::{MidiOutput, MidiOutputConnection, InitError, SendError};

pub const COMMAND_NOTE_OFF: u8  = 128;
pub const COMMAND_NOTE_ON: u8   = 144;
pub const COMMAND_CC: u8        = 176;

pub const VELOCITY_MIN: u8       = 0;
pub const VELOCITY_MAX: u8       = 127;

pub struct Transmitter {
    pub out: MidiOutputConnection,
}

impl Transmitter {
    pub fn new(name: &String) -> Result<Self, InitError> {
        let midi_out = MidiOutput::new("evdev-midi").unwrap();
        let conn_out = midi_out.connect(0, name).unwrap();

        Ok(Transmitter { out: conn_out })
    }

    pub fn send(&mut self, midi_msg: &[u8; 3]) -> Result<(), SendError> {
        return self.out.send(midi_msg);
    }
}
