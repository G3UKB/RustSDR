/*
prefs.rs

Save/restore preferences

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

extern crate preferences;
use preferences::{AppInfo, PreferencesMap, Preferences};
use std::collections::hash_map::Entry;

const APP_INFO: AppInfo = AppInfo{name: "RustSDRprefs", author: "Bob Cowdery"};

//===========================================================================================
// State for prefs
pub struct Prefs {
    prefs: PreferencesMap<String>,
    prefs_key: String,
}

//===========================================================================================
// Implementation for UIApp
impl Prefs {
    pub fn new() -> Self{

        Self {
            prefs: PreferencesMap::new(),
            prefs_key: String::from("rustsdr.prefs"),
        }
    }

    pub fn restore(&mut self) {
        
        // Try to load prefs
        // Will store under prefs_base_dir()/BobCowdery/RustSDRPrefs/rustsdr.prefs
        let load_result = PreferencesMap::<String>::load(&APP_INFO, &self.prefs_key);
        if load_result.is_ok() {
            // Use these prefs
            self.prefs = load_result.unwrap();
        }
    }

    pub fn save(&mut self) {
        // Save prefs
        let save_result = self.prefs.save(&APP_INFO, &self.prefs_key);
        if !save_result.is_ok() {
            println!("Failed to save preferences!");
        }
    }

    pub fn store(&mut self, key: String, value: String) {
        self.prefs.insert(key.into(), value.into());
    }

    pub fn read(&mut self, key: String) -> String {
        match self.prefs.entry(key) {
            Entry::Occupied(v) => return String::from(v.get().as_str()),
            Entry::Vacant(_) => return String::from(""),
        }
    }

}
