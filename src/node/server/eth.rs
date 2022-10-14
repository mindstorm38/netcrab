//! Implementation of the Ethernet data-link layer handler.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::net::Ipv4Addr;
use std::time::Instant;

use crate::net::Link;
use crate::proto::{
    MacAddr, EthFrame, EthPayload, 
    ArpIpv4Packet, ArpOp,
    Ipv4Packet, 
};

use super::{ServerIface, ServerIfaceConf};


/// Ethernet interface.
pub struct ServerEthIface {
    /// MAC address of the interface.
    mac_addr: MacAddr,
    arp_cache: HashMap<Ipv4Addr, ArpEntry>,
}

enum ArpEntry {
    Known(MacAddr),
    Pending {
        time: Instant,
        packets: Vec<Box<Ipv4Packet>>,
    }
}

impl ServerEthIface {

    pub fn new(mac_addr: MacAddr) -> Self {
        Self {
            mac_addr,
            arp_cache: HashMap::new(),
        }
    }

}

impl ServerIface<EthFrame> for ServerEthIface {

    fn tick(&mut self, mut link: Link<EthFrame>, conf: &mut ServerIfaceConf) {

        while let Some(frame) = link.recv() {

            if !frame.dst.is_multicast() && frame.dst != self.mac_addr {
                // Filter incomming frames and ignore frames that don't 
                // target this interface.
                continue;
            }

            match frame.payload {
                EthPayload::Arp(arp) => {
                    if let Some(ipv4) = &conf.ipv4 {
                        self.recv_arp(&mut link, &*arp, ipv4.ip);
                    }
                }
                EthPayload::Ipv4(ip) => {
                    if let Some(ipv4) = &conf.ipv4 {
                        
                    }
                }
                _ => {}
            }

        }

    }

}

impl ServerEthIface {

    /// Manually associate an IPv4 to a MAC in the ARP cache.
    fn set_arp(&mut self, link: &mut Link<EthFrame>, ip: Ipv4Addr, mac: MacAddr) {
        match self.arp_cache.entry(ip) {
            Entry::Occupied(mut o) => {
                if let ArpEntry::Pending { packets, .. } = o.get_mut() {
                    for packet in packets.drain(..) {
                        link.send(Box::new(EthFrame { 
                            src: self.mac_addr, 
                            dst: mac, 
                            payload: EthPayload::Ipv4(packet)
                        }));
                    }
                }
                o.insert(ArpEntry::Known(mac));
            }
            Entry::Vacant(v) => {
                v.insert(ArpEntry::Known(mac));
            }
        }
    }

    /// Internal function to handle ARP IPv4.
    fn recv_arp(&mut self, link: &mut Link<EthFrame>, arp: &ArpIpv4Packet, local_ipv4: Ipv4Addr) {

        match arp.op {
            ArpOp::Request => {

                // Arp requests are only processed if we have a local
                // IPv4 set for the interface.
                if arp.target_ip == local_ipv4 {
                    // If the local IP is the requested one, send reply.
                    link.send(Box::new(EthFrame { 
                        src: self.mac_addr, 
                        dst: arp.sender_mac, 
                        payload: EthPayload::Arp(Box::new(ArpIpv4Packet { 
                            op: ArpOp::Reply, 
                            sender_mac: self.mac_addr, 
                            target_mac: arp.sender_mac, 
                            sender_ip: local_ipv4, 
                            target_ip: arp.sender_ip 
                        }))
                    }));
                }

                // We also take the sender IP/MAC and save it.
                self.set_arp(link, arp.sender_ip, arp.sender_mac);

            }
            ArpOp::Reply => {
                self.set_arp(link, arp.sender_ip, arp.sender_mac);
            }
        }

    }

}