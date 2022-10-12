use crate::net::{Node, RawLinkHandle, Links, LinkHandle};


/// A node that can be used to link, it will freely accept 
/// links but will not interact. It will just recv packets
/// and ignore them.
pub struct NoopNode<T> {
    links: Vec<LinkHandle<T>>
}

impl<T> NoopNode<T> {
    pub const fn new() -> Self {
        Self { links: Vec::new() }
    }
}

impl<T: 'static> Node for NoopNode<T> {

    fn link(&mut self, _iface: usize, link: RawLinkHandle) -> bool {
        if let Some(link) = link.cast::<T>() {
            self.links.push(link);
            true
        } else {
            false
        }
    }

    fn tick(&mut self, links: &mut Links) {
        for handle in &self.links {
            let mut link = links.get(handle);
            while let Some(_) = link.recv() { }
        }
    }

}
