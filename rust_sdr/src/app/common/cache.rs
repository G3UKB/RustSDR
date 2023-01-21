/*
cache.rs

Module - cache
Cache for object refs

Copyright (C) 2023 by G3UKB Bob Cowdery

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

use std::{cell::RefCell, rc::Rc};

use crate::app;
use crate::app::common::prefs;

//========================================================================
// Implementations

// The arrays that are modified by several threads/callers are wrapped in an Arc
// allowing safe sharing.

pub struct ObjCache{
	iapp: Rc<RefCell<app::Appdata>>,
    iprefs: Rc<RefCell<prefs::Prefs>>,
}

// Implementation methods on ObjCache
impl ObjCache {
	// Create a new instance and initialise the default arrays
	pub fn new(iapp: Rc<RefCell<app::Appdata>>, iprefs: Rc<RefCell<prefs::Prefs>>) -> ObjCache {
		
        ObjCache {
            iapp: iapp,
            iprefs: iprefs,
        }
	}

    pub fn app_ref(&self) -> Rc<RefCell<app::Appdata>>{
        return self.iapp.clone();
    }

    pub fn prefs_ref(&self) -> Rc<RefCell<prefs::Prefs>>{
        return self.iprefs.clone();
    }
}
