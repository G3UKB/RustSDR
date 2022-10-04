/*
cc_out_defs.rs
module cc_out_defs

Command and Control definitions for cc_out (to hardware)

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

// Speed
pub enum CCOSpeed {
	S48kHz,
	S96kHz,
	S192kHz,
	S384kHz,
}

// Alex attenuator
pub enum CCOAlexAttn {
	Attn0db,
	Attn10db,
	Attn20db,
	Attn30db
}

// Preamp
pub enum CCOPreamp {
	PreAmpOff,
	PreAmpOn
}

// Alex RX ant
pub enum CCORxAnt {
	RxAntNone,
	RxAnt1,
	RxAnt2,
	RxAntXV
}

// Duplex
pub enum CCODuplex {
	DuplexOff,
	DuplexOn
}

// No.RX
pub enum CCONumRx {
	NumRx1,
	NumRx2,
	NumRx3
}

// Alex auto
pub enum CCOAlexAuto {
	AlexAuto,
	AlexManual
}

// Alex HPF bypass
pub enum CCOAlexBypass {
	AlexHpfDisable,
	AlexHpfEnable
}

// Alex LPF/HPF select
pub enum CCOAlexHpfLpf {
	AlexFiltDisable,
	AlexFiltEnable
}

// 10MHz ref
pub enum CCO10MhzRef {
	R10MHzAtlas,
	R10MHzPen,
	R10MHzMerc
}

// 122MHz ref
pub enum CCO122MhzRef {
	R122MHzPen,
	R122MHzMerc
}

// Board config
pub enum CCOBoardConfig {
	BoardNone,
	BoardPen,
	BoardMerc,
	BoardBoth
}

// Mic src
pub enum CCOMicSrc {
	MicJanus,
	MicPen
}

// Alex RX out
pub enum CCOAlexRxOut {
	RxOutOff,
	RxOutOn
}

// Alex TX relay
pub enum CCOAlexTxRly{
	TxRlyTx1,
	TxRlyTx2,
	TxRlyTx3
}