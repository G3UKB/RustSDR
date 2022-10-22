/*
vfo.rs

Module - vfo
User interface VFO control

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

use std::sync::{Arc, Mutex};

use fltk::app as fltk_app;
use fltk::{prelude::*, window::Window};

use crate::app::protocol;


//==================================================================================
// UI State
pub struct VFOState{
    i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>,
}


// Implementation methods on UDPRData
impl VFOState {
	// Create a new instance and initialise the default arrays
    pub fn new(i_cc : Arc<Mutex<protocol::cc_out::CCDataMutex>>) -> VFOState {

        VFOState {
            i_cc : i_cc,
        }
    }


    //=========================================================================================
    // Create main application window
    pub fn init_vfo(&mut self) {
        
    }

    

}


