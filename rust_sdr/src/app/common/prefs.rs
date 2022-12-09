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

const APP_INFO: AppInfo = AppInfo{name: "RustSDRprefs", author: "Bob Cowdery"};


fn main() {

    // Create a new preferences key-value map
    // (Under the hood: HashMap<String, String>)
    let mut faves: PreferencesMap<String> = PreferencesMap::new();

    // Edit the preferences (std::collections::HashMap)
    faves.insert("color".into(), "blue".into());
    faves.insert("programming language".into(), "Rust".into());

    // Store the user's preferences
    let prefs_key = "tests/docs/basic-example";
    let save_result = faves.save(&APP_INFO, prefs_key);
    assert!(save_result.is_ok());

    // ... Then do some stuff ...

    // Retrieve the user's preferences
    let load_result = PreferencesMap::<String>::load(&APP_INFO, prefs_key);
    assert!(load_result.is_ok());
    assert_eq!(load_result.unwrap(), faves);

}