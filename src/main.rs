use std::time::Duration;

use netcrab::net::{Network, DebugListener};
use netcrab::proto::{EthFrame, MacAddr, EthPayload};
use netcrab::node::{NoopNode, EthSwitch, EthNode, ServerNode, ServerIfaceLink};


fn main() {

    const MAC0: MacAddr = MacAddr([0, 0, 0x5E, 0, 0x53, 0xAF]);
    const MAC1: MacAddr = MacAddr([0, 0, 0x5E, 0, 0x53, 0xB0]);

    let mut net = Network::new();

    let node0 = net.push(EthNode::new(|i| {
        Box::new(EthFrame {
            src: MAC0,
            dst: MAC1,
            payload: EthPayload::Custom(vec![(i % 256) as u8])
        })
    }));

    let node1 = net.push(EthNode::new(|i| {
        Box::new(EthFrame {
            src: MAC1,
            dst: MAC0,
            payload: EthPayload::Custom(vec![(i % 256) as u8])
        })
    }));

    let node2 = net.push(NoopNode::<EthFrame>::new());
    
    let mut node3 = ServerNode::new();
    node3.add_interface(0, ServerIfaceLink::Ethernet);
    let node3 = net.push(node3);

    let switch = net.push(EthSwitch::new());

    net.link::<EthFrame>(node0, 0, switch, 0);
    net.link::<EthFrame>(node1, 0, switch, 1);
    net.link::<EthFrame>(node2, 0, switch, 2);
    net.link::<EthFrame>(node3, 0, switch, 3);

    let mut debugger = DebugListener::<EthFrame>::new();
    debugger.name(node0, "NODE0");
    debugger.name(node1, "NODE1");
    debugger.name(node2, "NODE2");
    debugger.name(node3, "NODE3");
    debugger.name(switch, "SWITCH");
    net.subscribe(debugger);

    loop {
        net.tick();
        std::thread::sleep(Duration::from_secs(1));
    }

}
