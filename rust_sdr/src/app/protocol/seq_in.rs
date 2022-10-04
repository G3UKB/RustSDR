/*
seq_in.rs

Module - seq_in
Manages the EP6 sequence number check

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

use std::io::{self, Write};

//========================================================================
// Implementations

// The arrays that are modified by several threads/callers are wrapped in an Arc
// allowing safe sharing.

pub struct SeqData{
	// Maximum sequence number
	seq_max: u32,
    // EP6 sequence number to check
    ep6_seq_check: u32,
    ep6_init: bool,
}

// Implementation methods on SeqData
impl SeqData {
	// Create a new instance and initialise the default arrays
	pub fn new() -> SeqData {
		SeqData {
			seq_max: u32::MAX,
            ep6_seq_check: 0,
            ep6_init: false,
		}
	}

    pub fn check_ep6_seq(&mut self, seq: [u8; 4]) -> bool {
        let mut r: bool = false;
        let new_seq = self.big_to_little_endian(seq);
        if !self.ep6_init {
            self.ep6_seq_check = new_seq;
            self.ep6_init = true;
        } else if new_seq == 0 { 
            self.ep6_seq_check = 0;
        } else if self.ep6_seq_check + 1 != new_seq {
            io::stdout().flush().unwrap();
            println!("EP6 sequence error - Ex:{}, Got:{}", self.ep6_seq_check, new_seq);
            self.ep6_seq_check = new_seq;
        } else {
            r = true;
            io::stdout().flush().unwrap();
            self.ep6_seq_check = self.next_seq(self.ep6_seq_check);
        }
        return r;
    }

    fn next_seq(&self, seq: u32) -> u32 {
        let mut new_seq = seq + 1;
        if new_seq > self.seq_max {
            new_seq = 0;
        }
        return new_seq;
    }

    fn big_to_little_endian(&mut self, big_endian: [u8; 4]) -> u32 {
        let mut little_endian: u32 = 0;
        little_endian = big_endian[0] as u32;
        little_endian = (little_endian << 8) | (big_endian[1] as u32);
        little_endian = (little_endian << 8) | (big_endian[2] as u32);
        little_endian = (little_endian << 8) | (big_endian[3] as u32);
        return little_endian;
    }
}
