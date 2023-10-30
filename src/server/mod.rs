use bevy::prelude::{EventReader, Res, ResMut, Resource};
use bevy_quinnet::server::{QuinnetServerPlugin, Server, ServerConfiguration};
use bevy_quinnet::server::certificate::CertificateRetrievalMode;
use bevy_quinnet::shared::ClientId;
use std::net::{IpAddr, Ipv4Addr};
use bevy::app::{App, Plugin, Startup, Update};
use crate::shared::Port;
use crate::client::LOCAL_BIND_IP;

pub const SERVER_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[derive(Resource, Debug, Clone, Default)]
pub struct ServerData {
    pub connection_events_received: u64,
    pub last_connected_client_id: Option<ClientId>,
}

pub struct ServerPlugin {
    port: u16,
}

impl ServerPlugin {
    pub fn new(port: u16) -> Self {
        Self {
            port
        }
    }
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(QuinnetServerPlugin::default())
            .insert_resource(ServerData::default())
            .insert_resource(Port(self.port))
            .add_systems(Startup, start_listening)
            .add_systems(Update, handle_server_events);
    }
}

pub fn start_listening(mut server: ResMut<Server>, port: Res<Port>) {
    server
        .start_endpoint(
            ServerConfiguration::from_ip(LOCAL_BIND_IP, port.0),
            CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: SERVER_IP.to_string(),
            },
        )
        .unwrap();
}

pub fn handle_server_events(
    mut connection_events: EventReader<bevy_quinnet::server::ConnectionEvent>,
    mut test_data: ResMut<ServerData>,
) {
    for connected_event in connection_events.iter() {
        test_data.connection_events_received += 1;
        test_data.last_connected_client_id = Some(connected_event.id);
    }
}
