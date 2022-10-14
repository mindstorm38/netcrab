use std::net::{Ipv4Addr, Ipv6Addr};


/// A trait implemented on both IPv4 and IPv6 to allow taking prefix
/// of an existing address and compare two address.
pub trait IpPrefixAddr: Sized + Eq + Copy {

    /// Take the prefix of this IP.
    fn take_prefix(self, prefix_len: u8) -> IpPrefix<Self>;

}

impl IpPrefixAddr for Ipv4Addr {

    #[inline]
    fn take_prefix(self, prefix_len: u8) -> IpPrefix<Self> {
        debug_assert!(prefix_len <= 32);
        let num: u32 = self.into();
        IpPrefix {
            addr: (num & (u32::MAX << (32 - prefix_len))).into(),
            prefix_len,
        }
    }

}

impl IpPrefixAddr for Ipv6Addr {

    #[inline]
    fn take_prefix(self, prefix_len: u8) -> IpPrefix<Self> {
        let num: u128 = self.into();
        IpPrefix {
            addr: (num & (u128::MAX << (128 - prefix_len))).into(),
            prefix_len,
        }
    }

}


/// An IP prefix.
#[derive(Clone, PartialEq, Eq)]
pub struct IpPrefix<T> {
    addr: T,
    prefix_len: u8,
}

impl<T: IpPrefixAddr> IpPrefix<T> {

    /// Get the IP of this prefix.
    pub fn ip(&self) -> T {
        self.addr
    }

    /// Get the prefix length.
    pub fn prefix_len(&self) -> u8 {
        self.prefix_len
    }

    /// Check if the given IP address matches the prefix.
    pub fn matches(&self, ip: T) -> bool {
        self.addr == ip.take_prefix(self.prefix_len).addr
    }

}
