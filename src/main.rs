use std::time::Duration;

use netcrab::net::{Network, DebugListener, RcNode};
use netcrab::proto::{EthFrame, MacAddr, Ipv4Addr, Ipv4Packet, Ipv4Payload};
use netcrab::node::{
    EthSwitch, 
    ServerNode, ServerEthIface, IpRouteLink, ServerIfaceConf
};


fn main() {

    const MAC0: MacAddr = MacAddr([0, 0, 0x5E, 0, 0x53, 0xAF]);
    const MAC1: MacAddr = MacAddr([0, 0, 0x5E, 0, 0x53, 0xB0]);
    const MAC2: MacAddr = MacAddr([0, 0, 0x5E, 0, 0x53, 0x52]);
    const IP0: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 10);
    const IP1: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 11);
    const IP2: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 12);

    let mut net = Network::new();
    
    let pc0_node = RcNode::new(ServerNode::with_iface_conf(0, ServerEthIface::new(MAC0), ServerIfaceConf::with_ipv4(IP0, 24)));
    let pc1_node = RcNode::new(ServerNode::with_iface_conf(0, ServerEthIface::new(MAC1), ServerIfaceConf::with_ipv4(IP1, 24)));
    let pc2_node = RcNode::new(ServerNode::with_iface_conf(0, ServerEthIface::new(MAC2), ServerIfaceConf::with_ipv4(IP2, 24)));

    pc0_node.borrow_mut().get_ipv4_routes_mut().set_default_route(0, IpRouteLink::Direct);
    pc0_node.borrow_mut().send_ipv4(Box::new(Ipv4Packet::new(IP0, IP1, Ipv4Payload::Custom(vec![1]))));
    pc0_node.borrow_mut().send_ipv4(Box::new(Ipv4Packet::new(IP0, IP1, Ipv4Payload::Custom(vec![2]))));

    let pc0 = net.push(RcNode::clone(&pc0_node));
    let pc1 = net.push(RcNode::clone(&pc1_node));
    let pc2 = net.push(RcNode::clone(&pc2_node));
    let switch = net.push(EthSwitch::new());

    net.link::<EthFrame>(pc0, 0, switch, 0);
    net.link::<EthFrame>(pc1, 0, switch, 1);
    net.link::<EthFrame>(pc2, 0, switch, 2);

    let mut debugger = DebugListener::<EthFrame>::new();
    debugger.name(pc0, "PC0");
    debugger.name(pc1, "PC1");
    debugger.name(pc2, "PC2");
    debugger.name(switch, "SWI");
    net.subscribe(debugger);

    loop {
        net.tick();
        std::thread::sleep(Duration::from_secs(1));
    }

}
