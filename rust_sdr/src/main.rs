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
use std::{cell::RefCell, rc::Rc};

//extern crate preferences;
//use preferences::{AppInfo, PreferencesMap, Preferences};
use crate ::app::common::prefs;

pub mod app;

// Prefs info
//const APP_INFO: AppInfo = AppInfo{name: "RustSDRprefs", author: "BobCowdery"};

/// Entry point for RustConsole SDR application
///
/// # Examples
///
fn main() {
    println!("Starting Rust Console...");

    // Create a Prefs instance
    let prefs = prefs::Prefs::new();
    let wprefs = Rc::new(RefCell::new(prefs));

    // Create an instance of the Application manager type
    let mut i_app = app::Appdata::new(wprefs.clone());

    // This will initialise all modules and run the back-end system
    i_app.app_init(wprefs.clone());

    // Initialise the UI
    // This runs the UI event loop and will return when the UI is closed
    i_app.ui_run(wprefs.clone());

    // Close application
    println!("\n\nStarting shutdown...");
    i_app.app_close();

    // Save prefs
    wprefs.borrow_mut().save();

    println!("Rust console closing...");
    thread::sleep(Duration::from_millis(1000));
}


