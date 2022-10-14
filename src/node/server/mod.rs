//! Implementation of a complex server supporting an 
//! IPv4 and IPv6 stack with ARP and NDP support.

use std::collections::HashMap;

use crate::net::{LinkHandle, Node, RawLinkHandle, Links, Link};
use crate::proto::{Ipv4Addr, IpPrefixAddr, IpPrefix, EthFrame, Ipv4Packet};

mod eth;
pub use eth::*;


/// A complex node that supports whole IP stack.
/// With this type of node you need to manually register interfaces.
pub struct ServerNode {
    ifaces: HashMap<usize, Iface>,
    ipv4_routes: IpRoutes<Ipv4Addr>,
}

impl ServerNode {

    pub fn new() -> Self {
        Self {
            ifaces: HashMap::new(),
            ipv4_routes: IpRoutes::new(),
        }
    }

    #[inline]
    pub fn ipv4_routes(&mut self) -> &mut IpRoutes<Ipv4Addr> {
        &mut self.ipv4_routes
    }

    /// Specify a new interface. 
    /// This will panic if the interface is already defined.
    pub fn add_interface<T, H>(&mut self, iface: usize, handler: H)
    where
        T: 'static,
        H: ServerIface<T> + 'static,
    {

        if self.ifaces.contains_key(&iface) {
            panic!("this interface is already defined");
        }

        self.ifaces.insert(iface, Iface { 
            inner: Box::new(IfaceInner {
                link: None,
                handler
            }), 
            conf: ServerIfaceConf { 
                ipv4: None,
            }
        });

    }

    fn send_ipv4(&mut self, packet: Box<Ipv4Packet>) {
        
    }

}

impl Node for ServerNode {

    fn link(&mut self, iface: usize, link: RawLinkHandle) -> bool {
        if let Some(iface) = self.ifaces.get_mut(&iface) {
            iface.inner.link(link)
        } else {
            false
        }
    }

    fn tick(&mut self, links: &mut Links) {
        for iface in self.ifaces.values_mut() {
            iface.inner.tick(links, &mut iface.conf)
        }
    }

}

/// Basic trait for all possible interface link-layer implementations,
/// such as Ethernet.
pub trait ServerIface<T> {

    /// Called each tick when this interface is linked.
    fn tick(&mut self, link: Link<T>, conf: &mut ServerIfaceConf);

}

/// Generic protocols config for an interface. It contains configurations
/// for protocols such as IPv4 and IPv6.
pub struct ServerIfaceConf {
    pub ipv4: Option<ServerIpv4>,
}

/// IPv4 configuration for an interface.
pub struct ServerIpv4 {
    /// Configured IPv4.
    pub ip: Ipv4Addr,
    /// Configured subnet mask.
    pub mask: u8,
}

// INTERNALS //

/// Internal structure to store an interface's state.
struct Iface {
    /// Link kind of the interface.
    inner: Box<dyn IfaceInnerUntyped>,
    /// Network configuration for this interface.
    conf: ServerIfaceConf,
}

/// Internal structure for storage interface link.
struct IfaceInner<T, H: ServerIface<T>> {
    /// Link handle if linked, None if not yet linked.
    link: Option<LinkHandle<T>>,
    /// Inner implementation.
    handler: H,
}

/// Internal type to allow dynamic dispatching of calls to 
/// `IfaceLink`. It is only implemented for `IfaceLink`.
trait IfaceInnerUntyped {
    fn link(&mut self, link: RawLinkHandle) -> bool;
    fn tick(&mut self, links: &mut Links, conf: &mut ServerIfaceConf);
}

impl<T, H> IfaceInnerUntyped for IfaceInner<T, H>
where
    T: 'static,
    H: ServerIface<T>,
{

    fn link(&mut self, link: RawLinkHandle) -> bool {
        if let Some(link) = link.cast::<T>() {
            self.link = Some(link);
            true
        } else {
            false
        }
    }

    fn tick(&mut self, links: &mut Links, conf: &mut ServerIfaceConf) {
        if let Some(link) = &self.link {
            self.handler.tick(links.get(link), conf);
        }
    }

}

pub struct IpRoutes<T> {
    routes: Vec<Route<T>>,
    default: Option<IpRouteKind<T>>,
}

impl<T> IpRoutes<T>
where
    T: Copy + Eq + IpPrefixAddr
{

    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            default: None,
        }
    }

    /// Add a new route for the given address prefix.
    pub fn add_route(&mut self, prefix: IpPrefix<T>, kind: IpRouteKind<T>) {
        self.routes.push(Route { prefix, kind });
    }

    /// Set the default route.
    pub fn set_default_route(&mut self, kind: IpRouteKind<T>) {
        self.default = Some(kind);
    }

    /// Try to find a route for the given address regarding this routes table.
    /// If found, the interface index and the next hop IP is returned.
    #[inline]
    pub fn fetch(&self, ip: T) -> Option<(usize, T)> {
        self.fetch_inner(ip, 255)
    }

    fn fetch_inner(&self, ip: T, recursion: u8) -> Option<(usize, T)> {
        if recursion > 0 {
            for route in &self.routes {
                // The route IP is already the prefix itself.
                if route.prefix.matches(ip) {
                    match route.kind {
                        IpRouteKind::Iface(iface) => return Some((iface, route.prefix.ip())),
                        IpRouteKind::NextHop(next_hop) => return self.fetch_inner(next_hop, recursion - 1),
                    }
                }
            }
        }
        None
    }

}

/// Different kinds of IP routes.
pub enum IpRouteKind<T> {
    /// The packet needs to pass trough the given router.
    /// This router needs to have a valid `Iface` route to lead to it.
    NextHop(T),
    /// The packet can be sent directly via the interface.
    Iface(usize),
}

struct Route<T> {
    /// Prefix IP.
    prefix: IpPrefix<T>,
    /// The kind of route to take.
    kind: IpRouteKind<T>
}




// /// Different types of layer 2 links valid when specifying an interface.
// pub enum ServerIfaceLink {
//     Ethernet(MacAddr),
// }

// /// Internal structure to store an interface's state.
// struct Iface {
//     /// Link kind of the interface.
//     link: IfaceLink,
//     /// IPv4 configuration for this interface.
//     ipv4: Option<IfaceIpv4>,
// }

// /// IPv4 configuration for an interface.
// struct IfaceIpv4 {
//     /// Configured IPv4.
//     ip: Ipv4Addr,
//     /// Configured subnet mask.
//     mask: u8,
// }

// /// Represent the different kinds of interface links.
// enum IfaceLink {
//     /// Ethernet link type.
//     Ethernet(IfaceEthernet),
// }

// struct IfaceEthernet {
//     /// Static MAC address of the interface.
//     mac_addr: MacAddr,
//     /// We the interface is linked, it contains some handle to the link.
//     link: Option<LinkHandle<EthFrame>>,
//     /// Caching ARP association between IPv4 address and MAC.
//     /// If the ARP address is set to `Err(instant)`, this means
//     /// that a ARP request packet has been sent at this specific 
//     /// instant. A list of waiting IPv4 packets is also provided.
//     arp_cache: HashMap<Ipv4Addr, Result<MacAddr, (Instant, Vec<Box<Ipv4Packet>>)>>,
// }

// // TODO:
// enum IfaceArpEntry {
//     Known(MacAddr),
//     Pending {
//         time: Instant,
//         packets: Vec<Box<Ipv4Packet>>,
//     }
// }

// impl Iface {

//     /// Internal method used to update the internal link handle
//     /// depending on the interface type.
//     fn update_link(&mut self, link: RawLinkHandle) -> Option<()> {
//         match &mut self.link {
//             IfaceLink::Ethernet(eth) => eth.link = Some(link.cast()?),
//         }
//         Some(())
//     }

//     /// Tick this interface to receive data-link layer packets 
//     /// and process them. 
//     fn tick(&mut self, links: &mut Links) {
//         match &mut self.link {
//             IfaceLink::Ethernet(eth) => {
//                 if let Some(link) = &eth.link {

//                     let mut link = links.get(link);
//                     while let Some(frame) = link.recv() {

//                         if !frame.dst.is_multicast() && frame.dst != eth.mac_addr {
//                             // Filter incomming frames and ignore frames that don't 
//                             // target this ethernet interface.
//                             continue;
//                         }

//                         match frame.payload {
//                             EthPayload::Arp(arp) => {
//                                 if let Some(ipv4) = &self.ipv4 {
//                                     eth.recv_arp_ipv4(&mut link, &*arp, ipv4.ip);
//                                 }
//                             }
//                             EthPayload::Ipv4(ip) => {
//                                 if let Some(ipv4) = &self.ipv4 {
                                    
//                                 }
//                             }
//                             _ => {}
//                         }

//                     }
                    
//                 }
//             }
//         }
//     }

//     /// Send an IPv4 packet through this interface.
//     fn send_ipv4(&mut self, links: &mut Links, src_ip: Ipv4Addr, hop_ip: Ipv4Addr, packet: Box<Ipv4Packet>) {
//         match &mut self.link {
//             IfaceLink::Ethernet(eth) => {
//                 if let Some(link) = &eth.link {

//                     let mut link = links.get(link);

//                     // Here we need to find the correct MAC address for the destination
//                     let mut hop_mac = MacAddr::BROADCAST;

//                     if hop_ip.is_multicast() {
//                         // Multicast IPv4 addresses uses specific MAC addresses.
//                         let o = hop_ip.octets();
//                         hop_mac = MacAddr([0x01, 0x00, 0x5E, o[1] & 0x7F, o[2], o[3]]);
//                     } else if hop_ip.is_broadcast() {
//                         // Broadcast IPv4 always use the broadcast MAC address.
//                         hop_mac = MacAddr::BROADCAST;
//                     } else {

//                         // TODO: Rework using hashmap entries.

//                         const ARP_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

//                         let mut send_arp = false;
//                         match eth.arp_cache.get_mut(&hop_ip) {
//                             Some(Ok(mac)) => {
//                                 // We know the mac address from ARP cache.
//                                 hop_mac = *mac;
//                             }
//                             Some(Err((instant, queue))) 
//                             if instant.elapsed() < ARP_REQUEST_TIMEOUT => {
//                                 // A request is already in-progress, enqueue the current packet.
//                                 queue.push(packet);
//                                 return;
//                             }
//                             _ => {
//                                 // Need to (re)send an ARP request.
//                                 send_arp = true;
//                             }
//                         }

//                         if send_arp {
//                             link.send(Box::new(EthFrame { 
//                                 src: eth.mac_addr, 
//                                 dst: MacAddr::BROADCAST, 
//                                 payload: EthPayload::Arp(Box::new(ArpIpv4Packet {
//                                     op: ArpOp::Request,
//                                     sender_mac: eth.mac_addr,
//                                     target_mac: MacAddr::ZERO, // Zero because it's a request.
//                                     sender_ip: src_ip, 
//                                     target_ip: hop_ip
//                                 }))
//                             }));
//                             eth.arp_cache.insert(hop_ip, Err((Instant::now(), vec![packet])));
//                             return;
//                         }

//                     }

//                     // Actually send the packet to the right MAC address.
//                     link.send(Box::new(EthFrame { 
//                         src: eth.mac_addr, 
//                         dst: hop_mac, 
//                         payload: EthPayload::Ipv4(packet),
//                     }));

//                 }
//             }
//         }
//     }
    
// }

// impl IfaceEthernet {

//     /// Internal method to process received ARP Ipv4 packets.
//     fn recv_arp_ipv4(&mut self, link: &mut Link<EthFrame>, arp: &ArpIpv4Packet, local_ipv4: Ipv4Addr) {

//         match arp.op {
//             ArpOp::Request => {

//                 // Arp requests are only processed if we have a local
//                 // IPv4 set for the interface.
//                 if arp.target_ip == local_ipv4 {
//                     // If the local IP is the requested one, send reply.
//                     link.send(Box::new(EthFrame { 
//                         src: self.mac_addr, 
//                         dst: arp.sender_mac, 
//                         payload: EthPayload::Arp(Box::new(ArpIpv4Packet { 
//                             op: ArpOp::Reply, 
//                             sender_mac: self.mac_addr, 
//                             target_mac: arp.sender_mac, 
//                             sender_ip: local_ipv4, 
//                             target_ip: arp.sender_ip 
//                         }))
//                     }));
//                 }

//             }
//             ArpOp::Reply => {

//                 match self.arp_cache.entry(arp.sender_ip) {
//                     Entry::Occupied(mut o) => {

//                         // If we have waiting packets for this ip address,
//                         // send them before.
//                         if let Err((_, packets)) = o.get_mut() {
//                             for packet in packets.drain(..) {
//                                 link.send(Box::new(EthFrame { 
//                                     src: self.mac_addr, 
//                                     dst: arp.sender_mac, 
//                                     payload: EthPayload::Ipv4(packet)
//                                 }))
//                             }
//                         }

//                         // Then, update the mapping.
//                         let _ = o.insert(Ok(arp.sender_mac));

//                     },
//                     Entry::Vacant(v) => {
//                         // Insert a new mapping.
//                         v.insert(Ok(arp.sender_mac));
//                     }
//                 }

//             }
//         }

//     }

// }
