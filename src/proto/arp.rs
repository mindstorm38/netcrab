use super::{MacAddr, Ipv4Addr, Ipv6Addr};


#[derive(Debug, Clone)]
pub struct ArpPacket {
    pub op: ArpOp,
    pub sender_mac: MacAddr,
    pub target_mac: MacAddr,
    pub proto: ArpProtocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpOp {
    Request,
    Reply,
}

#[derive(Debug, Clone)]
pub enum ArpProtocol {
    Ipv4 {
        sender_ip: Ipv4Addr,
        target_ip: Ipv4Addr,
    },
    Ipv6 {
        sender_ip: Ipv6Addr,
        target_ip: Ipv6Addr,
    },
}
