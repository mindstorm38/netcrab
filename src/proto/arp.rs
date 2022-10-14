use super::{MacAddr, Ipv4Addr};


#[derive(Debug, Clone)]
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
