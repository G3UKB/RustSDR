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
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

//===========================================================================================
// State for prefs

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Windows {
    pub main_x: f32,
    pub main_y: f32,
    pub main_w: f32,
    pub main_h: f32,

    pub vfo_x: f32,
    pub vfo_y: f32,
    pub vfo_w: f32,
    pub vfo_h: f32,

    pub mode_x: f32,
    pub mode_y: f32,
    pub mode_w: f32,
    pub mode_h: f32,

    pub filt_x: f32,
    pub filt_y: f32,
    pub filt_w: f32,
    pub filt_h: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Prefs {
    pub prefs_path: String,
    pub windows: Windows,
}

//===========================================================================================
// Implementation for Prefs
impl Prefs {
    pub fn new() -> Self{

        Self {
            prefs_path: String::from("E:\\Projects\\RustSDR\\trunk\\rust_sdr\\prefs\\rustsdr.prefs"),
            
            windows: { Windows {
                    main_x: 0.0,
                    main_y: 0.0,
                    main_w: 500.0,
                    main_h: 300.0,

                    vfo_x: 0.0,
                    vfo_y: 0.0,
                    vfo_w: 300.0,
                    vfo_h: 60.0,

                    mode_x: 0.0,
                    mode_y: 0.0,
                    mode_w: 300.0,
                    mode_h: 60.0,

                    filt_x: 0.0,
                    filt_y: 0.0,
                    filt_w: 300.0,
                    filt_h: 60.0,
                }
            }
        }
    }

    pub fn restore(&mut self) {

        // Open (and initialise if not present) the prefs file
        let mut file = self.open_file();
        // Regardless, initialise the structure from the restored file
        let path = Path::new(&self.prefs_path);
        let display = path.display();
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read prefs file! {}: {}", display, why),
            Ok(_) => {
                //print!("{} contains:\n{}\n", display, s);
                let prefs: Prefs = serde_json::from_str(&s).unwrap();
                self.windows.main_x = prefs.windows.main_x;
                self.windows.main_y = prefs.windows.main_y;
                self.windows.main_w = prefs.windows.main_w;
                self.windows.main_h = prefs.windows.main_h;

                self.windows.vfo_x = prefs.windows.vfo_x;
                self.windows.vfo_y = prefs.windows.vfo_y;
                self.windows.vfo_w = prefs.windows.vfo_w;
                self.windows.vfo_h = prefs.windows.vfo_h;

                self.windows.mode_x = prefs.windows.mode_x;
                self.windows.mode_y = prefs.windows.mode_y;
                self.windows.mode_w = prefs.windows.mode_w;
                self.windows.mode_h = prefs.windows.mode_h;

                self.windows.filt_x = prefs.windows.filt_x;
                self.windows.filt_y = prefs.windows.filt_y;
                self.windows.filt_w = prefs.windows.filt_w;
                self.windows.filt_h = prefs.windows.filt_h;
            },
        }
    }   
    
    pub fn save(&mut self) {
        // Write the new data
        let _ = self.write_file();
    }

    fn open_file(&mut self) -> File {
        let path = Path::new(&self.prefs_path);
        let _display = path.display();
        // Open the path in read-only mode, returns `io::Result<File>`
        let mut _file = match File::open(&path) {
            Err(_why) => {
                // File not present so initialise
                return self.write_file();
            },
            Ok(file) => return file,
        };
    }

    fn write_file(&mut self) -> File {
        let path = Path::new(&self.prefs_path);
        let display = path.display();
        let serialized = serde_json::to_string(&self).unwrap();

        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = match File::create(path) {
            Err(why) => panic!("couldn't create prefs file! {}: {}", display, why),
            Ok(file) => file,
        };

        // Write the data to `file`, returns `io::Result<()>`
        match file.write_all(serialized.as_bytes()) {
            Err(why) => panic!("couldn't write data to prefs file! {}: {}", display, why),
            Ok(_) => (), //println!("successfully wrote to prefs file {}", display),
        }
        return file;
    }

}

