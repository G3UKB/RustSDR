/*
ringb.rs

Ring buffer implementation based on VecDeque
Code provided by H2CO3

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

use std::io::{self, Read, Write};
use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard, PoisonError, TryLockError, TryLockResult};
//use std::io::ReadBuf;

#[derive(Default, Debug)]
pub struct SyncByteRingBuf {
    q: Mutex<VecDeque<u8>>,
}

impl SyncByteRingBuf {
    pub fn with_capacity(cap: usize) -> Self {
        SyncByteRingBuf {
            q: Mutex::new(VecDeque::with_capacity(cap)),
        }
    }
    
    pub fn read(&self) -> SyncByteRingBufReadGuard<'_> {
        SyncByteRingBufReadGuard(self.q.lock().expect("poisoned"))
    }

    pub fn write(&self) -> SyncByteRingBufWriteGuard<'_> {
        SyncByteRingBufWriteGuard(self.q.lock().expect("poisoned"))
    }
    
    pub fn try_read(&self) -> TryLockResult<SyncByteRingBufReadGuard<'_>> {
        self.q
            .try_lock()
            .map(SyncByteRingBufReadGuard)
            .map_err(|e| map_lock_error(e, SyncByteRingBufReadGuard))
    }
    
    pub fn try_write(&self) -> TryLockResult<SyncByteRingBufWriteGuard<'_>> {
        self.q
            .try_lock()
            .map(SyncByteRingBufWriteGuard)
            .map_err(|e| map_lock_error(e, SyncByteRingBufWriteGuard))
    }
}

fn map_lock_error<T, U, F>(error: TryLockError<T>, f: F) -> TryLockError<U>
where
    F: FnOnce(T) -> U
{
    match error {
        TryLockError::WouldBlock => TryLockError::WouldBlock,
        TryLockError::Poisoned(p) => TryLockError::Poisoned(PoisonError::new(f(p.into_inner()))),
    }
}

pub struct SyncByteRingBufReadGuard<'a>(MutexGuard<'a, VecDeque<u8>>);
pub struct SyncByteRingBufWriteGuard<'a>(MutexGuard<'a, VecDeque<u8>>);

impl SyncByteRingBufReadGuard<'_> {
    pub fn available(&self) -> usize {
        self.0.len()
    }
}

impl Read for SyncByteRingBufReadGuard<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() <= self.0.len() {
            self.0.read_exact(buf).map(|_| buf.len())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "not enough data available in ring buffer"))
        }
    }
}

impl SyncByteRingBufWriteGuard<'_> {
    pub fn remaining(&self) -> usize {
        self.0.capacity() - self.0.len()
    }
}

impl Write for SyncByteRingBufWriteGuard<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() <= self.remaining() {
            self.0.write_all(buf).map(|_| buf.len())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "write data exceeds ring buffer capacity"))
        }
    }
    
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

/* 
fn main() -> io::Result<()> {
    let buf = SyncByteRingBuf::with_capacity(1024);
    
    for i in 0..10 {
        let outdata: Vec<u8> = (0..=255).cycle().take(200).collect();
        let r = buf.try_write();
        match r {
            Ok(mut m) => {
                let r = m.write(&outdata);
                match r {
                    Err(e) => println!("Error on write {:?}", e),
                    Ok(sz) => println!("Wrote {:?}", sz)
                }
            }
            Err(e) => println!("Lock error {:?}", e),
        }
        
        let mut indata = vec![0; 300];
        let r1 = buf.try_read();   
        match r1 {
            Ok(mut m) => {
                let r2 = m.read(&mut indata);
                match r2 {
                    Ok(sz) => println!("Read {:?}", sz),
                    Err(e) => println!("Error on read {:?}", e),
                    
                }
            }
            Err(e) => println!("Lock error {:?}", e),
        }
    }
    
    Ok(())
}
*/