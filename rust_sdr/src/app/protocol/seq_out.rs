/*
seq_out.rs

Module - seq_out
Manages the EP2,4 sequence number

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

//========================================================================
// Implementations

pub struct SeqData{
	// Maximum sequence number
	seq_max: u32,
    // Current EP2 and EP4 sequence number
    ep2_seq: u32,
    ep4_seq: u32,
    // Encoded 4 byte sequence number
    big_endian_seq: [u8; 4],
}

// Implementation methods on SeqData
impl SeqData {
	// Create a new instance and initialise the default arrays
	pub fn new() -> SeqData {
		SeqData {
			seq_max: u32::MAX,
            ep2_seq: 0,
            ep4_seq: 0,
            big_endian_seq: ([0,0,0,0]),
		}
	}

    pub fn next_ep2_seq(&mut self) -> [u8; 4] {
        self.ep2_seq = self.next_seq(self.ep2_seq);
        // Return this as a byte array in BE format
	    self.little_to_big_endian(self.ep2_seq);
        return self.big_endian_seq.clone();
    }

    pub fn next_ep4_seq(&mut self) -> [u8; 4] {
        self.ep4_seq = self.next_seq(self.ep4_seq);
        // Return this as a byte array in BE format
	    self.little_to_big_endian(self.ep4_seq);
        return self.big_endian_seq.clone();
    }

    fn next_seq(&self, seq: u32) -> u32 {
        let mut new_seq = seq + 1;
        if new_seq > self.seq_max {
            new_seq = 0;
        }
        return new_seq;
    }

    fn little_to_big_endian(&mut self, little_endian: u32) {
        self.big_endian_seq[3] = (little_endian & 0xff) as u8;
        self.big_endian_seq[2] = ((little_endian >> 8) & 0xff) as u8;
        self.big_endian_seq[1] = ((little_endian >> 16) & 0xff) as u8;
        self.big_endian_seq[0] = ((little_endian >> 24) & 0xff) as u8;
    }

}
