use std::net::Ipv4Addr;


pub trait MaskableIp {

    /// Take the prefix of this IP.
    fn take_prefix(self, prefix_len: u8) -> Self;

    /// Return true if the two addresses have a common prefix.
    #[inline]
    fn has_same_prefix(self, other: Self, prefix_len: u8) -> bool
        where Self: Sized + Eq
    {
        self.take_prefix(prefix_len) == other.take_prefix(prefix_len)
    }

}

impl MaskableIp for Ipv4Addr {

    #[inline]
    fn take_prefix(self, prefix_len: u8) -> Self {
        debug_assert!(prefix_len <= 32);
        let mut num = u32::from_be_bytes(self.octets());
        num &= u32::MAX << (32 - prefix_len);
        let [a, b, c, d] = num.to_be_bytes();
        Self::new(a, b, c, d)
    }

}
