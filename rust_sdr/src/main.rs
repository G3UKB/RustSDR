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

pub mod app;

/// Entry point for RustConsole SDR application
///
/// # Examples
///
fn main() {
    println!("Starting Rust Console...");

    // Create an instance of the Application manager type
    let mut i_app = app::Appdata::new();

    // This will initialise all modules, run the back-end system
    // and initialise the UI
    i_app.app_init();

    // Initialise the UI
    // This runs the UI event loop and will return when the UI is closed
    i_app.ui_run();

    // UI has closed but we still wait to close down so issues can be seen
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

