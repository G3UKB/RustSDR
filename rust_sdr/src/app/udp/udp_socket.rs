/*
udp_socket.rs

Module - udp_socket
Manage UDP socket instance

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

    use std::net::UdpSocket;
    use std::time::Duration;
    use std::sync::Arc;

    use if_addrs;
    use socket2;

    pub struct Sockdata{
        sock2 : Arc<socket2::Socket>,
    }
  
    impl Sockdata {
        pub fn new() -> Sockdata {
            let sock = Self::udp_open_bc_socket();
            Sockdata {  
                sock2 : Arc::new(socket2::Socket::from (sock)),
            }
        }

        pub fn udp_revert_socket(&mut self) {
            self.sock2.set_broadcast(false).expect("set_broadcast call failed");
            self.sock2.set_read_timeout(Some(Duration::from_millis(100))).expect("set_read_timeout call failed");
            // Set buffer sizes?
            self.sock2.set_recv_buffer_size(192000).expect("set_recv_buffer_size call failed");
            println!("Receiver buffer sz {:?}", self.sock2.recv_buffer_size());
            self.sock2.set_send_buffer_size(192000).expect("set_send_buffer_size call failed");
            println!("Send buffer sz {:?}", self.sock2.send_buffer_size());
        }

        pub fn udp_sock_ref(&mut self) -> Arc<socket2::Socket> {
            return self.sock2.clone();
        }

        fn udp_open_bc_socket() -> UdpSocket {
            let sock = UdpSocket::bind(Self::get_ip() + ":" + "10000").expect("couldn't bind to address");
            sock.set_broadcast(true).expect("set_broadcast call failed");
            sock.set_read_timeout(Some(Duration::from_millis(500))).expect("set_read_timeout call failed");
            return sock
        }

        fn get_ip() -> String{
            let iface = if_addrs::get_if_addrs().unwrap();
            println!("My IP {}", iface[0].ip().to_string());
            return iface[0].ip().to_string();
        }
    }
