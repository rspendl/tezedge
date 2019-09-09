use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::net::SocketAddr;
use std::time::Duration;

use dns_lookup::LookupError;
use log::{info, warn};
use riker::actors::*;

use networking::p2p::network_channel::NetworkChannelMsg;
use networking::p2p::network_manager::{ConnectToPeer, NetworkManagerRef};
use networking::p2p::peer::PeerRef;

use crate::{subscribe_to_actor_terminated, subscribe_to_network_events};

/// Check peer threshold
#[derive(Clone, Debug)]
pub struct CheckThreshold;

#[derive(Clone)]
pub struct Threshold {
    low: usize,
    high: usize,
}

impl Threshold {
    pub fn new(low: usize, high: usize) -> Self {
        assert!(low <= high, "low must be less than or equal to high");
        Threshold { low, high }
    }

    pub fn mid_range(&self) -> usize {
        return self.low + ((self.high - self.low) / 2);
    }
}

#[actor(CheckThreshold, NetworkChannelMsg, SystemEvent)]
pub struct PeerManager {
    /// All events generated by the peer will end up in this channel
    event_channel: ChannelRef<NetworkChannelMsg>,
    network: NetworkManagerRef,
    threshold: Threshold,
    peers: HashMap<ActorUri, PeerRef>,
    bootstrap_addresses: Vec<String>,
    potential_peers: HashSet<SocketAddr>,
}

pub type PeerManagerRef = ActorRef<PeerManagerMsg>;

impl PeerManager {
    pub fn actor(sys: &impl ActorRefFactory,
               event_channel: ChannelRef<NetworkChannelMsg>,
               network: NetworkManagerRef,
               bootstrap_addresses: &[String],
               initial_peers: &[SocketAddr],
               threshold: Threshold) -> Result<PeerManagerRef, CreateError> {

        sys.actor_of(
            Props::new_args(PeerManager::new, (event_channel, bootstrap_addresses.to_vec(), HashSet::from_iter(initial_peers.to_vec()), network, threshold)),
            PeerManager::name())
    }

    /// The `PeerManager` is intended to serve as a singleton actor so that's why
    /// we won't support multiple names per instance.
    fn name() -> &'static str {
        "peer-manager"
    }

    fn new((event_channel, bootstrap_addresses, potential_peers, network, threshold): (ChannelRef<NetworkChannelMsg>, Vec<String>, HashSet<SocketAddr>, NetworkManagerRef, Threshold)) -> Self {
        PeerManager { event_channel, network, bootstrap_addresses, threshold, peers: HashMap::new(), potential_peers }
    }
}

impl Actor for PeerManager {
    type Msg = PeerManagerMsg;

    fn pre_start(&mut self, ctx: &Context<Self::Msg>) {
        subscribe_to_actor_terminated(ctx.system.sys_events(), ctx.myself());
        subscribe_to_network_events(&self.event_channel, ctx.myself());

        ctx.schedule::<Self::Msg, _>(
            Duration::from_secs(2),
            Duration::from_secs(30),
            ctx.myself(),
            None,
            CheckThreshold.into());
    }

    fn sys_recv(&mut self, ctx: &Context<Self::Msg>, msg: SystemMsg, sender: Option<BasicActorRef>) {
        if let SystemMsg::Event(evt) = msg {
            self.receive(ctx, evt, sender);
        }
    }

    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
        self.receive(ctx, msg, sender);
    }
}

impl Receive<SystemEvent> for PeerManager {
    type Msg = PeerManagerMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: SystemEvent, _sender: Option<BasicActorRef>) {
        if let SystemEvent::ActorTerminated(evt) = msg {
            self.peers.remove(evt.actor.uri());
        }
    }
}

impl Receive<CheckThreshold> for PeerManager {
    type Msg = PeerManagerMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, _msg: CheckThreshold, _sender: Sender) {
        if self.peers.len() < self.threshold.low {
            warn!("Peer count is too low. actual={}, required={}", self.peers.len(), self.threshold.low);
            if self.potential_peers.len() < self.threshold.low {
                info!("Looking for new peers..");
                // lookup more peers
                lookup_peers(&self.bootstrap_addresses).iter()
                    .for_each(|i| {
                        info!("found potential peer: {}", i);
                        self.potential_peers.insert(*i);
                    });
            }

            let addresses_to_connect = self.potential_peers.iter()
                .take(self.threshold.low - self.peers.len())
                .map(|address| address.clone())
                .collect::<Vec<_>>();
            addresses_to_connect.iter()
                .for_each(|address| {
                    self.potential_peers.remove(address);
                    self.network.tell(ConnectToPeer { address: address.clone() }, ctx.myself().into())
                });
        } else if self.peers.len() > self.threshold.high {
            warn!("Peer count is too high. Some peers will be stopped. actual={}, required={}", self.peers.len(), self.threshold.high);

            // stop some peers
            self.peers.values()
                .take(self.peers.len() - self.threshold.high)
                .for_each(|peer| ctx.system.stop(peer.clone()))
        }
    }
}

impl Receive<NetworkChannelMsg> for PeerManager {
    type Msg = PeerManagerMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: NetworkChannelMsg, _sender: Sender) {
        if let NetworkChannelMsg::PeerCreated(msg) = msg {
            self.peers.insert(msg.peer.uri().clone(), msg.peer);
        }
    }
}

fn lookup_peers(bootstrap_addresses: &[String]) -> HashSet<SocketAddr> {
    let mut resolved_peers = HashSet::new();
    for address in bootstrap_addresses {
        match resolve_dns_name_to_peer_address(&address) {
            Ok(peers) => {
                resolved_peers.extend(&peers)
            },
            Err(e) => {
                warn!("DNS lookup for address: {:?} error: {:?}", &address, e)
            }
        }
    }
    resolved_peers
}

fn resolve_dns_name_to_peer_address(address: &str) -> Result<Vec<SocketAddr>, LookupError> {
    let addrs = dns_lookup::getaddrinfo(Some(address), None, None)?
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .map(|mut info| {
            info.sockaddr.set_port(9732);
            info.sockaddr
        })
        .collect();
    Ok(addrs)
}
