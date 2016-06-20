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
