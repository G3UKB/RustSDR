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

use serde:: {Serialize, Deserialize};
use std::collections::hash_map::Entry;

//===========================================================================================
// State for prefs

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Windows {
    main_x: u32,
    main_y: u32,
    main_w: u32,

    vfo_x: u32,
    vfo_y: u32,

    mode_x: u32,
    mode_y: u32,

    filt_x: u32,
    filt_y: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Prefs {
    prefs_key: String,
    windows: Windows,
}

//===========================================================================================
// Implementation for Prefs
impl Prefs {
    pub fn new() -> Self{

        Self {
            prefs_key: String::from("rustsdr.prefs"),
            
            windows: { Windows {
                    main_x: 0,
                    main_y: 0,
                    main_w: 500,
                    vfo_x: 0,
                    vfo_y: 0,

                    mode_x: 0,
                    mode_y: 0,

                    filt_x: 0,
                    filt_y: 0,
                }
            }
        }
    }

    pub fn restore(&mut self) {
        
        let serialized = serde_json::to_string(&self).unwrap();
        println!("serialized = {}", serialized);

        let deserialized: Prefs = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        
    }   
    
    pub fn save(&mut self) {
        // Save prefs
        
    }
}

