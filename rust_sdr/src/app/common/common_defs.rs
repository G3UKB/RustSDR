/*
common_defs.rs
module common_defs

Common defs for application

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

// UDP frame sz
pub const FRAME_SZ : u32 = 1032;
// Protocal packet sz (2 per UDP frame)
pub const PROT_SZ : u32 = 504;
// Max IQ samples array sz
pub const IQ_ARR_SZ : u32 = NUM_SMPLS_1_RADIO * MAX_RADIOS * BYTES_PER_SAMPLE;
// This is temporary as we don't have the actual size yet
pub const MIC_ARR_SZ : u32 = IQ_ARR_SZ;

// Protocol fields
pub const EP2 : u8 = 0x02;
pub const EP4 : u8 = 0x04;
pub const EP6 : u8 = 0x06;
pub const DATA_PKT : u8 = 0x01;
// Sequence number
pub const FRAME_SEQ_OFFSET : u32 = 4;

// First USB frame
pub const FRAME_SYNC_1_OFFSET : u32 = FRAME_SEQ_OFFSET + 4;
pub const FRAME_CC_1_OFFSET : u32 = FRAME_SYNC_1_OFFSET + 3;
pub const START_FRAME_1 : u32 = FRAME_CC_1_OFFSET + 5;
pub const END_FRAME_1 : u32 = START_FRAME_1 + 503;

// Second USB frame
pub const FRAME_SYNC_2_OFFSET : u32 = START_FRAME_1 + 504;
pub const FRAME_CC_2_OFFSET : u32 = FRAME_SYNC_2_OFFSET + 3;
pub const START_FRAME_2 : u32 = FRAME_CC_2_OFFSET + 5;
pub const END_FRAME_2 : u32 = START_FRAME_2 + 503;

// Samples per packet
pub const NUM_SMPLS_1_RADIO : u32 = 126;
pub const NUM_SMPLS_2_RADIO : u32 = 72;
pub const NUM_SMPLS_3_RADIO : u32 = 50;

// Sample rates
pub const SMPLS_48K : u32 = 48000;
pub const SMPLS_96K : u32 = 96000;
pub const SMPLS_192K : u32 = 192000;
pub const SMPLS_384K : u32 = 384000;

// Misc
pub const MAX_RADIOS : u32 = 3;
pub const BYTES_PER_SAMPLE : u32 = 6;   // 2*24 bits
pub const FRAMES_IN_RING : u32 = 8;     // Number of frames to accommodate in the IQ ring buffer