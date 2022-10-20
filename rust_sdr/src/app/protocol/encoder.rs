/*
encoder.rs

Module - encoder
Module encoder manages encoding the protocol frame

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

use crate::app::common::common_defs:: {
    DATA_PKT,EP2,FRAME_SZ,PROT_SZ,FRAME_SEQ_OFFSET,FRAME_SYNC_1_OFFSET,FRAME_CC_1_OFFSET,START_FRAME_1,
    END_FRAME_1,FRAME_SYNC_2_OFFSET,FRAME_CC_2_OFFSET,START_FRAME_2,END_FRAME_2 };
use crate::app::protocol;

/*
*	<0xEFFE><0x01><End Point><Sequence Number>< 2 x HPSDR frames>
*	Where:
*		End point = 1 byte[0x02 – representing USB EP2]
*		Sequence Number = 4 bytes[32 bit unsigned]
*		HPSDR data = 1024 bytes[2 x 512 byte USB format frames]
*
*	The following fields are merged :
*		metis_header
*		out_seq		-- next output sequence number to use
*		cc_out 		-- round robin control bytes
*		usb_header +
*		proc_data	-- 2 frames worth of USB format frames
*
*	Data is encoded into the packet buffer
*/
pub fn encode(  i_seq: &mut protocol::seq_out::SeqData, 
                i_cc: &protocol::cc_out::CCDataMutex, 
                udp_frame: &mut [u8; FRAME_SZ as usize], 
                prot_frame: &mut [u8; PROT_SZ as usize *2]) {

    // Encode header
    udp_frame[0] = 0xef;
    udp_frame[1] = 0xfe;
    udp_frame[2] = DATA_PKT;
    udp_frame[3] = EP2;

    // Encode sequence number
    let next_seq = i_seq.next_ep2_seq();
    let mut j: usize = 0;
    for i in FRAME_SEQ_OFFSET..FRAME_SEQ_OFFSET + 4 {
        udp_frame[i as usize] = next_seq[j];
        j = j+1;
    }

    // First protocol frame
    // Header
    for i in FRAME_SYNC_1_OFFSET..FRAME_SYNC_1_OFFSET+3 {
        udp_frame[i as usize] = 0x7f;
    }
    // Encode command and control bytes
    let cc = &mut i_cc.cc_out_next_seq();
    j = 0;
    for i in FRAME_CC_1_OFFSET..FRAME_CC_1_OFFSET + 5 {
        udp_frame[i as usize] = cc[j];
        j = j+1;
    }
    // Frame data
    j = 0;
    for i in START_FRAME_1..END_FRAME_1 {
        udp_frame[i as usize] = prot_frame[j];
        j = j+1;
    }

    // Second protocol frame
    // Header
    for i in FRAME_SYNC_2_OFFSET..FRAME_SYNC_2_OFFSET+3 {
        udp_frame[i as usize] = 0x7f;
    }
    // Encode command and control bytes
    let cc = &mut i_cc.cc_out_next_seq();
    j = 0;
    for i in FRAME_CC_2_OFFSET..FRAME_CC_2_OFFSET + 5 {
        udp_frame[i as usize] = cc[j];
        j = j+1;
    }
    // Frame data
    j = PROT_SZ as usize;
    for i in START_FRAME_2..END_FRAME_2 {
        udp_frame[i as usize] = prot_frame[j];
        j = j+1;
    }
}

