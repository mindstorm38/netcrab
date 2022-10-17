use super::{MacAddr, Ipv4Addr};
use std::fmt;


#[derive(Clone)]
pub struct ArpIpv4Packet {
    pub op: ArpOp,
    pub sender_mac: MacAddr,
    pub target_mac: MacAddr,
    pub sender_ip: Ipv4Addr,
    pub target_ip: Ipv4Addr,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpOp {
    Request,
    Reply,
}


impl fmt::Debug for ArpIpv4Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArpIpv4Packet")
            .field("op", &self.op)
            .field("sender_mac", &format_args!("{}", self.sender_mac))
            .field("target_mac", &format_args!("{}", self.target_mac))
            .field("sender_ip", &format_args!("{}", self.sender_ip))
            .field("target_ip", &format_args!("{}", self.target_ip))
            .finish()
    }
}