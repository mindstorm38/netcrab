//! Implementation of a complex server supporting an 
//! IPv4 and IPv6 stack with ARP and NDP support.

use std::collections::HashMap;

use crate::net::{LinkHandle, Node, RawLinkHandle, Links};
use crate::proto::{EthFrame, Ipv4Addr, MacAddr};


/// A simple node that generate a packet each tick and broadcast it
/// to all of its ethernet frames. Received packets are ignored.
pub struct EthNode<S> {
    sender: S,
    index: usize,
    links: Vec<LinkHandle<EthFrame>>,
}

impl<G> EthNode<G>
where
    G: FnMut(usize) -> Box<EthFrame>,
{
    
    pub fn new(sender: G) -> Self {
        Self {
            sender,
            index: 0,
            links: Vec::new(),
        }
    }

}

impl<G> Node for EthNode<G>
where
    G: FnMut(usize) -> Box<EthFrame>,
{
    
    fn link(&mut self, _iface: usize, link: RawLinkHandle) -> bool {
        if let Some(link) = link.cast::<EthFrame>() {
            self.links.push(link);
            true
        } else {
            false
        }
    }

    fn tick(&mut self, links: &mut Links) {

        let frame = (self.sender)(self.index);
        self.index += 1;

        for handle in &self.links {
            let mut link = links.get(handle);
            link.send(frame.clone());
            while let Some(_) = link.recv() {}
        }

    }

}


/// A complex node that supports whole IP stack.
/// With this type of node you need to manually register interfaces.
pub struct ServerNode {
    ifaces: HashMap<usize, Iface>,
}

impl ServerNode {

    pub fn new() -> Self {
        Self {
            ifaces: HashMap::new(),
        }
    }

    /// Specify a new interface. 
    /// This will panic if the interface is already defined.
    pub fn add_interface(&mut self, iface: usize, link: ServerIfaceLink) {

        if self.ifaces.contains_key(&iface) {
            panic!("this interface is already defined");
        }

        self.ifaces.insert(iface, Iface {
            link: match link {
                ServerIfaceLink::Ethernet => IfaceLink::Ethernet(None),
            },
            ipv4: None,
        });

    }


}

impl Node for ServerNode {

    fn link(&mut self, iface: usize, link: RawLinkHandle) -> bool {
        if let Some(iface) = self.ifaces.get_mut(&iface) {
            iface.link.update_link(link).is_some()
        } else {
            false
        }
    }

    fn tick(&mut self, links: &mut Links) {

        for iface in self.ifaces.values_mut() {
            iface.link.receive_from(links)
        }

    }

}

/// Different types of layer 2 links valid when specifying an interface.
pub enum ServerIfaceLink {
    Ethernet,
}

/// Internal structure to store an interface's state.
struct Iface {
    /// Link kind of the interface.
    link: IfaceLink,
    /// IPv4 configuration for this interface.
    ipv4: Option<IfaceIpv4>,
}

/// Represent the different kinds of interface links.
enum IfaceLink {
    Ethernet(Option<LinkHandle<EthFrame>>),
}

impl IfaceLink {

    /// Internal method used to update the internal link handle
    /// depending on the interface type.
    fn update_link(&mut self, link: RawLinkHandle) -> Option<()> {
        match self {
            Self::Ethernet(h) => *h = Some(link.cast()?),
        }
        Some(())
    }

    fn receive_from(&mut self, links: &mut Links) {

    }
    
}

struct IfaceIpv4 {
    ip: Ipv4Addr,
    mask: u8,
    arp: HashMap<Ipv4Addr, MacAddr>,
}
