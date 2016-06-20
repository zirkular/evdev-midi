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
use std::io::Write;
use std::env::ArgsOs;

pub fn parse_args(args: ArgsOs) -> evdev::Device {
    let mut devices = evdev::enumerate();
    for (i, d) in devices.iter().enumerate() {
        println!("{}: {:?}", i, d.name());
    }
    print!("Select the device [0-{}]: ", devices.len());
    std::io::stdout().flush();
    let mut chosen = String::new();
    std::io::stdin().read_line(&mut chosen).unwrap();

    return devices.swap_remove(chosen.trim().parse::<usize>().unwrap());
}
