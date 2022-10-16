//! This module contains all primitive structures for
//! network simulation.

use std::collections::HashMap;
use std::marker::PhantomData;
use std::any::{TypeId, Any};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};


/// A handle to a node.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeHandle {
    index: usize,
}

/// Internally used to represent different sides of a point-to-point link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LinkSide {
    Side0,
    Side1,
}

/// A handle to a link, used internally by nodes to keep tracks
/// of links to them.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawLinkHandle {
    index: usize,
    side: LinkSide,
    ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinkHandle<T> {
    index: usize,
    side: LinkSide,
    _phantom: PhantomData<*const T>,
}

impl RawLinkHandle {
    
    pub fn new_pair<T: 'static>(index: usize) -> (Self, Self) {
        let ty = TypeId::of::<T>();
        (
            Self { index, side: LinkSide::Side0, ty },
            Self { index, side: LinkSide::Side1, ty },
        )
    }

    /// Return `true` if you can cast this raw link handle to the
    /// given `T`-typed link handle.
    pub fn is<T: 'static>(&self) -> bool {
        self.ty == TypeId::of::<T>()
    }

    /// Cast this raw link handle, if possible, to the given 
    /// `T`-typed link handle. To be further used as 
    pub fn cast<T: 'static>(&self) -> Option<LinkHandle<T>> {
        if self.ty == TypeId::of::<T>() {
            Some(LinkHandle { 
                index: self.index, 
                side: self.side,
                _phantom: PhantomData
            })
        } else {
            None
        }
    }

}


/// This structure defines a network of nodes. These nodes can
/// be later connected together between their interfaces.
pub struct Network {
    /// List of nodes contained in this network.
    nodes: Vec<Box<dyn Node>>,
    /// List of links, the type is dynamically allocated but should
    /// always be a concrete derivation of `LinkQueues<T>`.
    queues: Vec<Box<dyn Any>>,
    /// List of listeners for packets.
    listeners: Vec<Box<dyn UntypedListener>>
}

impl Network {

    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            queues: Vec::new(),
            listeners: Vec::new(),
        }
    }

    /// Add a new node to the network, its handle is returned and 
    /// can be later used to link nodes.
    pub fn push(&mut self, node: impl Node + 'static) -> NodeHandle {
        let index = self.nodes.len();
        self.nodes.push(Box::new(node));
        NodeHandle { index }
    }

    /// Link two nodes with a link.
    pub fn link<T: 'static>(&mut self, 
        node_0: NodeHandle, iface_0: usize, 
        node_1: NodeHandle, iface_1: usize,
    ) {

        let index = self.queues.len();
        
        let (
            handle_0, 
            handle_1
        ) = RawLinkHandle::new_pair::<T>(index);

        if !self.nodes.get_mut(node_0.index).unwrap().link(iface_0, handle_0) {
            panic!()
        }
        
        if !self.nodes.get_mut(node_1.index).unwrap().link(iface_1, handle_1) {
            panic!()
        }

        self.queues.push(Box::new(LinkQueues::<T> {
            queue_0: Vec::new(),
            queue_1: Vec::new(),
            node_0,
            node_1,
        }));

    }

    /// Tick each node in the network.
    pub fn tick(&mut self) {

        for node in &mut self.nodes {

            let node = &mut **node;

            let mut links = Links {
                queues: &mut self.queues,
                listeners: &mut self.listeners,
            };

            node.tick(&mut links);

        }

    }

    /// Subscribe with a listener for specific data transfers.
    pub fn subscribe<L: Listener + 'static>(&mut self, listener: L) {
        self.listeners.push(Box::new(listener))
    }

}


/// A structure defining an absolute 
struct LinkQueues<T> {
    /// Messages to be transfered to the first node of this link.
    queue_0: Vec<Box<T>>,
    /// Messages to be transfered to the second node of this link.
    queue_1: Vec<Box<T>>,
    node_0: NodeHandle,
    node_1: NodeHandle,
}

/// Temporary object given when ticking nodes, used to receive and send
/// data on link.
pub struct Links<'a> {
    queues: &'a mut Vec<Box<dyn Any>>,
    listeners: &'a mut Vec<Box<dyn UntypedListener>>,
}

impl<'a> Links<'a> {

    pub fn get<T: 'static>(&mut self, link: &LinkHandle<T>) -> Link<'_, T> {

        let queues_raw = self.queues.get_mut(link.index)
            .expect("invalid link");

        let queues = queues_raw.downcast_mut::<LinkQueues<T>>()
            .expect("incoherent link type");

        match link.side {
            LinkSide::Side0 => Link {
                tx: &mut queues.queue_0,
                rx: &mut queues.queue_1,
                tx_node: queues.node_1,
                rx_node: queues.node_0,
                listeners: self.listeners,
            },
            LinkSide::Side1 => Link {
                tx: &mut queues.queue_1,
                rx: &mut queues.queue_0,
                tx_node: queues.node_0,
                rx_node: queues.node_1,
                listeners: self.listeners,
            },
        }

    }

}

/// Temporary object returned by `Links` and used send and receive packets 
/// of the given type in the link.
pub struct Link<'a, T> {
    tx: &'a mut Vec<Box<T>>,
    rx: &'a mut Vec<Box<T>>,
    tx_node: NodeHandle,
    rx_node: NodeHandle,
    listeners: &'a mut Vec<Box<dyn UntypedListener>>,
}

impl<'a, T: 'static> Link<'a, T> {

    pub fn send(&mut self, data: Box<T>) {
        self.tx.push(data);
    }

    pub fn recv(&mut self) -> Option<Box<T>> {

        let data = self.rx.pop()?;

        for listener in &mut self.listeners[..] {
            listener.event(self.tx_node, self.rx_node, &*data);
        }

        Some(data)

    }

} 

/// Node that can be linked to other ones and ticked by the network controller.
pub trait Node {

    /// Called to link this node to a given link through the given interface.
    /// This function returns `true` when the link was successful.
    fn link(&mut self, iface: usize, link: RawLinkHandle) -> bool;

    /// Tick the node to process their links.
    fn tick(&mut self, links: &mut Links);
    
}

/// A listener to track packets sent on links.
pub trait Listener {

    /// data type that you want to capture on the link. 
    type Data;

    /// Called when an event of this type is transfered on a link.
    /// A data is considered transfered when actually received by
    /// an end.
    fn event(&mut self, src: NodeHandle, dst: NodeHandle, data: &Self::Data);

}

/// Internally used to store dynamically dispatched listeners.
trait UntypedListener {

    /// This event only triggers when the given dynamic type is
    /// valid for this listener.
    fn event(&mut self, src: NodeHandle, dst: NodeHandle, data: &dyn Any);

}

impl<L> UntypedListener for L
where
    L: Listener,
    L::Data: 'static
{
    fn event(&mut self, src: NodeHandle, dst: NodeHandle, data: &dyn Any) {
        if let Some(data) = data.downcast_ref::<L::Data>() {
            Listener::event(self, src, dst, data);
        }
    }
}

/// A basic `Listener` implementation that tracks a specific 
/// data type. You can associate node handles to a host name
/// in order to have better debugging.
pub struct DebugListener<T> {
    node_names: HashMap<NodeHandle, String>,
    _phantom: PhantomData<*const T>,
}

impl<T> DebugListener<T> {

    pub fn new() -> Self {
        Self {
            node_names: HashMap::new(),
            _phantom: PhantomData,
        }
    }

    pub fn name(&mut self, node: NodeHandle, name: impl ToString) {
        self.node_names.insert(node, name.to_string());
    }

}

impl<T: fmt::Debug> Listener for DebugListener<T> {
    type Data = T;
    fn event(&mut self, src: NodeHandle, dst: NodeHandle, data: &Self::Data) {
        match (self.node_names.get(&src), self.node_names.get(&dst)) {
            (Some(src), Some(dst)) => println!("[{src} -> {dst}] {data:?}"),
            (None, Some(dst)) => println!("[{src:?} -> {dst}] {data:?}"),
            (Some(src), None) => println!("[{src} -> {dst:?}] {data:?}"),
            (None, None) => println!("[{src:?} -> {dst:?}] {data:?}"),
        }
    }
}


/// A wrapper for node that can be mutably shared.
pub struct RcNode<N: Node> {
    inner: Rc<RefCell<N>>,
}

impl<N: Node> RcNode<N> {

    #[inline]
    pub fn new(node: N) -> Self {
        Self { inner: Rc::new(RefCell::new(node))}
    }

    #[inline]
    pub fn clone(self: &Self) -> Self {
        Self { inner: Rc::clone(&self.inner) }
    }

    #[inline]
    pub fn borrow_mut(&self) -> RefMut<N> {
        self.inner.borrow_mut()
    }

}

impl<N: Node> Node for RcNode<N> {

    fn link(&mut self, iface: usize, link: RawLinkHandle) -> bool {
        self.inner.borrow_mut().link(iface, link)
    }

    fn tick(&mut self, links: &mut Links) {
        self.inner.borrow_mut().tick(links)
    }

}


/// A wrapper for node that can be mutably shared between threads.
pub struct ArcNode<N: Node> {
    inner: Arc<Mutex<N>>,
}

impl<N: Node> ArcNode<N> {

    #[inline]
    pub fn new(node: N) -> Self {
        Self { inner: Arc::new(Mutex::new(node))}
    }

    #[inline]
    pub fn clone(self: &Self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }

    #[inline]
    pub fn borrow_mut(&self) -> MutexGuard<N> {
        self.inner.lock().unwrap()
    }

}

impl<N: Node> Node for ArcNode<N> {

    fn link(&mut self, iface: usize, link: RawLinkHandle) -> bool {
        self.inner.lock().unwrap().link(iface, link)
    }

    fn tick(&mut self, links: &mut Links) {
        self.inner.lock().unwrap().tick(links)
    }

}


// Debug impls

impl fmt::Debug for NodeHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NodeHandle")
            .field(&self.index)
            .finish()
    }
}
