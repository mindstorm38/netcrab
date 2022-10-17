//! Implementation of a complex server supporting an 
//! IPv4 and IPv6 stack with ARP and NDP support.

use std::collections::HashMap;

use crate::net::{LinkHandle, Node, RawLinkHandle, Links, Link};
use crate::proto::{Ipv4Addr, IpAddrExt, IpPrefix, Ipv4Packet};

mod eth;
pub use eth::*;


/// A complex node that supports whole IP stack.
/// With this type of node you need to manually register interfaces.
pub struct ServerNode {
    ifaces: HashMap<usize, Iface>,
    ipv4_queue: Vec<Box<Ipv4Packet>>,
    ipv4_routes: IpRoutes<Ipv4Addr>,
}

impl ServerNode {

    /// Construct a new server node.
    pub fn new() -> Self {
        Self {
            ifaces: HashMap::new(),
            ipv4_queue: Vec::new(),
            ipv4_routes: IpRoutes::new(),
        }
    }

    #[inline]
    pub fn with_iface_conf<T, H>(iface: usize, handler: H, conf: ServerIfaceConf) -> Self
    where
        T: 'static,
        H: ServerIface<T> + 'static,
    {
        let mut server = Self::new();
        server.add_iface_conf(iface, handler, conf);
        server
    }

    #[inline]
    pub fn with_iface<T, H>(iface: usize, handler: H) -> Self
    where
        T: 'static,
        H: ServerIface<T> + 'static,
    {
        Self::with_iface_conf(iface, handler, ServerIfaceConf::default())
    }

    /// Define a new interface with the given common configuration.
    pub fn add_iface_conf<T, H>(&mut self, iface: usize, handler: H, conf: ServerIfaceConf)
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
            conf
        });

    }

    /// Define a new interface.
    pub fn add_iface<T, H>(&mut self, iface: usize, handler: H)
    where
        T: 'static,
        H: ServerIface<T> + 'static,
    {
        self.add_iface_conf(iface, handler, ServerIfaceConf::default());
    }

    #[inline]
    pub fn get_ipv4_routes(&self) -> &IpRoutes<Ipv4Addr> {
        &self.ipv4_routes
    }

    #[inline]
    pub fn get_ipv4_routes_mut(&mut self) -> &mut IpRoutes<Ipv4Addr> {
        &mut self.ipv4_routes
    }

    /// Get a refernce to the given interface's configuration.
    pub fn get_iface_conf(&self, iface: usize) -> Option<&ServerIfaceConf> {
        self.ifaces.get(&iface).map(|iface| &iface.conf)
    }

    /// Get a mutable reference to the given interface' configuration.
    pub fn get_iface_conf_mut(&mut self, iface: usize) -> Option<&mut ServerIfaceConf> {
        self.ifaces.get_mut(&iface).map(|iface| &mut iface.conf)
    }

    /// Schedule a packet to be forwarded and sent through an interface.
    /// This function doesn't touch the source address.
    #[inline]
    pub fn send_ipv4(&mut self, packet: Box<Ipv4Packet>) {
        self.ipv4_queue.push(packet);
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
            iface.inner.tick(&mut *links, &mut iface.conf);
        }

        for packet in self.ipv4_queue.drain(..) {
            if let Some((iface_index, link_addr)) = self.ipv4_routes.fetch(packet.dst) {
                if let Some(iface) = self.ifaces.get_mut(&iface_index) {
                    if let Some(ipv4_conf) = &mut iface.conf.ipv4 {
                        iface.inner.send_ipv4(&mut *links, ipv4_conf, packet, link_addr);
                    } else {
                        // Packets that are sent to interfaces without IPv4 configuration are
                        // currently discarded silently.
                    }
                }
            }
        }

    }

}

/// Basic trait for all possible interface link-layer implementations,
/// such as Ethernet.
pub trait ServerIface<T> {

    /// Called each tick when this interface is linked. This is commonly
    /// used for polling incomming data-link frames.
    fn tick(&mut self, link: Link<T>, conf: &mut ServerIfaceConf);

    /// Send an IPv4 packet to the link address.
    /// 
    /// The link address is the IP address of the server that needs to receive this packet.
    /// In case of direct data-link connection to the destination, this link address is the
    /// same as the packet's destination.
    fn send_ipv4(&mut self, link: Link<T>, conf: &mut ServerIfaceIpv4, packet: Box<Ipv4Packet>, link_addr: Ipv4Addr);

}

/// Generic protocols config for an interface. It contains configurations
/// for protocols such as IPv4 and IPv6.
#[derive(Default)]
pub struct ServerIfaceConf {
    pub ipv4: Option<ServerIfaceIpv4>,
}

impl ServerIfaceConf {

    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_ipv4(ip: Ipv4Addr, prefix_len: u8) -> Self {
        Self {
            ipv4: Some(ServerIfaceIpv4 { ip, prefix_len }),
        }
    }

}

/// IPv4 configuration for an interface.
pub struct ServerIfaceIpv4 {
    /// Configured IPv4.
    pub ip: Ipv4Addr,
    /// Configured subnet mask.
    pub prefix_len: u8,
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
    fn send_ipv4(&mut self, links: &mut Links, conf: &mut ServerIfaceIpv4, packet: Box<Ipv4Packet>, link_addr: Ipv4Addr);
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

    fn send_ipv4(&mut self, links: &mut Links, conf: &mut ServerIfaceIpv4, packet: Box<Ipv4Packet>, link_addr: Ipv4Addr) {
        if let Some(link) = &self.link {
            self.handler.send_ipv4(links.get(link), conf, packet, link_addr);
        }
    }

}



pub struct IpRoutes<T: IpAddrExt> {
    routes: Vec<IpRoute<T>>,
    default: Option<IpRoute<T>>,
}

impl<T: IpAddrExt> IpRoutes<T> {

    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            default: None,
        }
    }
 
    /// Add a new route for the given address prefix.
    pub fn add_route(&mut self, prefix: IpPrefix<T>, iface: usize, link: IpRouteLink<T>) {
        self.routes.push(IpRoute { prefix, iface, link });
    }

    /// Set the default route.
    pub fn set_default_route(&mut self, iface: usize, link: IpRouteLink<T>) {
        self.default = Some(IpRoute { prefix: IpPrefix::ZERO, iface, link });
    }

    /// Try to find a route for the given address regarding this routes table.
    /// If found, the interface index and the next hop IP is returned.
    #[inline]
    pub fn fetch(&self, ip: T) -> Option<(usize, T)> {

        let route = self.routes.iter()
            .find(|route| route.prefix.matches(ip));
        
        // Take default route into account.
        let route = match route {
            Some(route) => Some(route),
            None => self.default.as_ref(),
        };

        route.map(|route| {
            (route.iface, route.link.ip_or_default(ip))
        })

    }

}

/// Different kinds of IP routes.
pub enum IpRouteLink<T: IpAddrExt> {
    /// The packet needs to pass trough the given router.
    Indirect(T),
    /// The packet's destination must be on the local link.
    Direct,
}

impl<T: IpAddrExt> IpRouteLink<T> {

    /// Get the next router IP address or take default if 
    /// this is a direct link.
    #[inline]
    pub fn ip_or_default(&self, default: T) -> T {
        match self {
            IpRouteLink::Indirect(ip) => *ip,
            IpRouteLink::Direct => default,
        }
    }

}

struct IpRoute<T: IpAddrExt> {
    /// Prefix IP.
    prefix: IpPrefix<T>,
    /// The interface to find the 
    iface: usize,
    /// The kind of route to take.
    link: IpRouteLink<T>
}
