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

use crate::app::common::cache; 
use crate::app::common::prefs;

pub mod app;

/// Entry point for RustConsole SDR application
fn main() {
    println!("Starting Rust Console...");

    // Create a Prefs instance
    // This is passed to anything that requires persistent data
    let prefs = prefs::Prefs::new();
    let wprefs = Rc::new(RefCell::new(prefs));
    wprefs.borrow_mut().restore();

    // Create an instance of the Application manager type
    let mut i_app = app::Appdata::new(wprefs.clone());
    let wapp = Rc::new(RefCell::new(i_app));

    // Create a cache instance and cache the main objects
    let cache = cache::ObjCache::new(wapp.clone(), wprefs.clone());
    let wcache = Rc::new(RefCell::new(cache));

    // This will initialise all modules and run the back-end and DSP system
    wapp.borrow_mut().app_init();

    // Initialise the UI
    // This runs the UI event loop and will return only when the UI is closed
    wapp.borrow_mut().ui_run(wcache.clone());

    // Tidy up
    // Close application
    println!("\n\nStarting shutdown...");
    wapp.borrow_mut().app_close();

    // Save prefs
    wprefs.borrow_mut().save();

    println!("Rust console closing...");
    thread::sleep(Duration::from_millis(1000));
}


