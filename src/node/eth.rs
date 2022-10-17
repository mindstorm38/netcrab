use std::collections::HashMap;

use crate::net::{LinkHandle, Node, RawLinkHandle, Links};
use crate::proto::{EthFrame, MacAddr};


/// An ethernet switch node.
pub struct EthSwitch {
    /// All registered links and their handles.
    link_handles: HashMap<usize, LinkHandle<EthFrame>>,
    /// Association of MAC addresses and the port that sent 
    /// the last frame with this source MAC addr.
    mac_to_iface: HashMap<MacAddr, usize>,
    /// Temporary vector of eth frames to broadcast and the 
    /// interface that received them.
    broadcast_queue: Vec<(Box<EthFrame>, usize)>,
    /// Temporary vector of eth frames to send to a specific
    /// interface.
    unicast_queue: Vec<(Box<EthFrame>, usize)>,
}

impl EthSwitch {
    pub fn new() -> Self {
        Self {
            link_handles: HashMap::new(),
            mac_to_iface: HashMap::new(),
            broadcast_queue: Vec::new(),
            unicast_queue: Vec::new(),
        }
    }
}

impl Node for EthSwitch {

    fn link(&mut self, iface: usize, link: RawLinkHandle) -> bool {
        if let Some(link) = link.cast::<EthFrame>() {
            self.link_handles.insert(iface, link);
            true
        } else {
            false
        }
    }

    fn tick(&mut self, links: &mut Links) {
        
        self.broadcast_queue.clear();
        self.unicast_queue.clear();

        for (iface, handle) in &self.link_handles {
            let mut link = links.get(handle);
            while let Some(frame) = link.recv() {
                // Associate the source MAC addr to the port.
                self.mac_to_iface.insert(frame.src, *iface);
                if frame.dst.is_multicast() {
                    self.broadcast_queue.push((frame, *iface));
                } else {
                    if let Some(dst_iface) = self.mac_to_iface.get(&frame.dst) {
                        self.unicast_queue.push((frame, *dst_iface));
                    } else {
                        self.broadcast_queue.push((frame, *iface));
                    }
                }
            }
        }

        for (link_iface, handle) in &self.link_handles {
            let mut link = links.get(handle);
            for (frame, frame_iface) in &self.broadcast_queue {
                // Don't send the broadcast frame to the sender iface.
                if *link_iface != *frame_iface {
                    link.send(frame.clone());
                }
            }
        }

        for (frame, iface) in self.unicast_queue.drain(..) {
            if let Some(handle) = self.link_handles.get(&iface) {
                let mut link = links.get(handle);
                link.send(frame);
            }
        }

    }

}
