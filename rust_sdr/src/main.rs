/*
main.rs

Entry module for the RustConsole SDR application

Copyright (C) 2022 by G3UKB Bob Cowdery

This program is free software; you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation; either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program; if not, write to the Free Software
Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA

The authors can be reached by email at:

bob@bobcowdery.plus.com
*/

use std::thread;
use std::time::Duration;
use std::io::{stdin, stdout, Read, Write};
use fltk::app as fltk_app;
use fltk::{prelude::*, window::Window};

pub mod app;

/// Entry point for RustConsole SDR application
///
/// # Examples
///
fn main() {
    println!("Starting Rust Console...");

    // Create an instance of the Application manager type
    let mut i_app = app::Appdata::new();

    // This will initialise all modules and run the system
    i_app.app_init();

    let fltk_app = fltk_app::App::default();
    let mut wind = Window::new(100, 100, 400, 300, "Hello from rust");
    wind.end();
    wind.show();
    fltk_app.run().unwrap();

    // At this point e would be doing GUI stuff
    // Temporary code to wait for Rtn then close everything and exit
    pause();
    println!("Starting shutdown...");
    i_app.app_close();

    println!("Rust console closing...");
    thread::sleep(Duration::from_millis(1000));
}

fn pause() {
    let mut stdout = stdout();
    stdout.write(b"\nPress Enter to close...\n\n").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}

