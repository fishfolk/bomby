//! This module is where the netcode lives.
//!
//! For now at least, we are using a listen-server model. [`bevy_replicon`] provides the replication
//! abstractions on top of [`bevy_replicon_renet`] for the transport layer.

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket},
    time::SystemTime,
};

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    netcode::{
        ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
        ServerConfig,
    },
    renet::ConnectionConfig,
    RenetChannelsExt, RenetClient, RenetServer,
};

/// TODO: Use the commit hash.
const PROTOCOL_ID: u64 = 0;

#[derive(Resource, Clone)]
pub enum NetworkMode {
    Server { port: u16 },
    Client { ip: IpAddr, port: u16 },
}

impl NetworkingPlugin {
    pub fn new(mode: NetworkMode) -> Self {
        Self(mode)
    }

    pub fn server(port: u16) -> Self {
        Self::new(NetworkMode::Server { port })
    }

    pub fn client(ip: IpAddr, port: u16) -> Self {
        Self::new(NetworkMode::Client { ip, port })
    }
}

pub struct NetworkingPlugin(NetworkMode);

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RepliconPlugins)
            .insert_resource(self.0.clone())
            .add_systems(PreStartup, init_renet);
    }
}

fn init_renet(mut commands: Commands, mode: Res<NetworkMode>, channels: Res<RepliconChannels>) {
    let connection_config = ConnectionConfig {
        server_channels_config: channels.server_configs(),
        client_channels_config: channels.client_configs(),
        ..default()
    };

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    match *mode {
        NetworkMode::Server { port } => {
            let server = RenetServer::new(connection_config);

            let server_config = ServerConfig {
                current_time,
                max_clients: 3,
                protocol_id: PROTOCOL_ID,
                public_addresses: Default::default(),
                authentication: ServerAuthentication::Unsecure,
            };

            // For now, only IPv4. In the future we should have a flag for IPv6.
            let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
            let socket = UdpSocket::bind(socket_addr)
                .unwrap_or_else(|err| panic!("failed to bind UDP socket on {socket_addr}: {err}"));
            let transport = NetcodeServerTransport::new(server_config, socket)
                .unwrap_or_else(|err| panic!("failed to initialise transport layer: {err}"));
            commands.insert_resource(server);
            commands.insert_resource(transport);
            info!("successfully initialised server, listening on {socket_addr}");
        }
        NetworkMode::Client { ip, port } => {
            let client = RenetClient::new(connection_config);
            let client_id = current_time.as_millis() as u64;

            let socket_addr = SocketAddr::new(
                match ip {
                    IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                    IpAddr::V6(_) => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                },
                0,
            );
            let socket = UdpSocket::bind(socket_addr)
                .unwrap_or_else(|err| panic!("failed to bind UDP socket on {socket_addr}: {err}"));

            let server_addr = match ip {
                IpAddr::V4(addr) => SocketAddr::V4(SocketAddrV4::new(addr, port)),
                // TODO: Investigate `flowinfo` and `scope_id`.
                IpAddr::V6(addr) => SocketAddr::V6(SocketAddrV6::new(addr, port, 0, 0)),
            };

            let authentication = ClientAuthentication::Unsecure {
                client_id,
                protocol_id: PROTOCOL_ID,
                server_addr,
                user_data: None,
            };
            let transport = NetcodeClientTransport::new(current_time, authentication, socket)
                .unwrap_or_else(|err| panic!("failed to initialise transport layer: {err}"));

            commands.insert_resource(client);
            commands.insert_resource(transport);
            info!("initialised client transport for {server_addr}");
        }
    }
}
