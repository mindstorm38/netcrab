//! Implementation of simple nodes for testing data-link layer.

use crate::net::{LinkHandle, Node, RawLinkHandle, Links};
use crate::proto::EthFrame;


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
