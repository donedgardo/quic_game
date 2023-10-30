use std::net::{IpAddr, Ipv4Addr};
use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use bevy_quinnet::client::certificate::{CertConnectionAbortEvent, CertificateVerificationMode, CertInteractionEvent, CertTrustUpdateEvent, CertVerificationInfo, CertVerificationStatus, CertVerifierAction};
use bevy_quinnet::client::connection::ConnectionConfiguration;
use bevy_quinnet::client::{Client, QuinnetClientPlugin};
use bevy_quinnet::shared::ClientId;
use bevy_quinnet::server::{Server, ServerConfiguration};
use bevy_quinnet::server::certificate::CertificateRetrievalMode;
use bevy_quinnet::server::QuinnetServerPlugin;
use serde::{Deserialize, Serialize};

pub const SERVER_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
pub const LOCAL_BIND_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

fn main() {
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ServerData {
    pub connection_events_received: u64,
    pub last_connected_client_id: Option<ClientId>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ClientData {
    pub connection_events_received: u64,

    pub cert_trust_update_events_received: u64,
    pub last_trusted_cert_info: Option<CertVerificationInfo>,

    pub cert_interactions_received: u64,
    pub last_cert_interactions_status: Option<CertVerificationStatus>,
    pub last_cert_interactions_info: Option<CertVerificationInfo>,

    pub cert_verif_connection_abort_events_received: u64,
    pub last_abort_cert_status: Option<CertVerificationStatus>,
    pub last_abort_cert_info: Option<CertVerificationInfo>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct Port(u16);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SharedMessage {
    TestMessage(String),
}

#[cfg(test)]
mod game_connection_tests {
    use super::*;

    #[test]
    fn it_connects_to_server() {
        let port = 5050;
        let mut server = create_server_app(port);
        let mut client = create_client_app(port);
        {
            let client = client.world.resource::<Client>();
            assert!(
                client.get_connection().is_some(),
                "The default connection should exist"
            );
            let server = server.world.resource::<Server>();
            assert!(server.is_listening(), "The server should be listening");
        }
        let client_id = wait_for_client_connected(&mut client, &mut server);
        let sent_client_message = SharedMessage::TestMessage("Test message content".to_string());
        {
            let server_test_data = server.world.resource::<ServerData>();
            assert_eq!(server_test_data.connection_events_received, 1);

            let network_client = client.world.resource::<Client>();
            let client_test_data = client.world.resource::<ClientData>();
            assert!(
                network_client.connection().is_connected(),
                "The default connection should be connected to the server"
            );
            assert_eq!(client_test_data.connection_events_received, 1);
            network_client
                .connection()
                .send_message(sent_client_message.clone())
                .unwrap();
        }
        // Client->Server Message
        std::thread::sleep(std::time::Duration::from_secs_f32(0.1));
        server.update();
        {
            let client_message = server
                .world
                .resource_mut::<Server>()
                .endpoint_mut()
                .receive_message_from::<SharedMessage>(client_id)
                .expect("Failed to receive client message")
                .expect("There should be a client message");
            assert_eq!(client_message, sent_client_message);
        }

        let sent_server_message = SharedMessage::TestMessage("Server response".to_string());
        {
            let server = server.world.resource::<Server>();
            server
                .endpoint()
                .broadcast_message(sent_server_message.clone())
                .unwrap();
        }
        // Server->Client Message
        std::thread::sleep(std::time::Duration::from_secs_f32(0.1));
        client.update();
        {
            let mut client = client.world.resource_mut::<Client>();
            let server_message = client
                .connection_mut()
                .receive_message::<SharedMessage>()
                .expect("Failed to receive server message")
                .expect("There should be a server message");
            assert_eq!(server_message, sent_server_message);
        }
    }
}

//client stuff
pub fn create_client_app(port: u16) -> App {
    let mut client = App::new();
    client
        .add_plugins((
            ScheduleRunnerPlugin::default(),
            QuinnetClientPlugin::default(),
        ))
        .insert_resource(ClientData::default())
        .add_systems(Startup, start_simple_connection)
        .add_systems(Update, handle_client_events);
    client.insert_resource(Port(port));
    client.update();
    client
}

pub fn start_simple_connection(mut client: ResMut<Client>, port: Res<Port>) {
    client
        .open_connection(
            ConnectionConfiguration::from_ips(SERVER_IP, port.0, LOCAL_BIND_IP, 0),
            CertificateVerificationMode::SkipVerification,
        )
        .unwrap();
}

pub fn handle_client_events(
    mut connection_events: EventReader<bevy_quinnet::client::connection::ConnectionEvent>,
    mut cert_trust_update_events: EventReader<CertTrustUpdateEvent>,
    mut cert_interaction_events: EventReader<CertInteractionEvent>,
    mut cert_connection_abort_events: EventReader<CertConnectionAbortEvent>,
    mut test_data: ResMut<ClientData>,
) {
    for _connected_event in connection_events.iter() {
        test_data.connection_events_received += 1;
    }
    for trust_update in cert_trust_update_events.iter() {
        test_data.cert_trust_update_events_received += 1;
        test_data.last_trusted_cert_info = Some(trust_update.cert_info.clone());
    }
    for cert_interaction in cert_interaction_events.iter() {
        test_data.cert_interactions_received += 1;
        test_data.last_cert_interactions_status = Some(cert_interaction.status.clone());
        test_data.last_cert_interactions_info = Some(cert_interaction.info.clone());

        match cert_interaction.status {
            CertVerificationStatus::UnknownCertificate => todo!(),
            CertVerificationStatus::UntrustedCertificate => {
                cert_interaction
                    .apply_cert_verifier_action(CertVerifierAction::AbortConnection)
                    .expect("Failed to apply cert verification action");
            }
            CertVerificationStatus::TrustedCertificate => todo!(),
        }
    }
    for connection_abort in cert_connection_abort_events.iter() {
        test_data.cert_verif_connection_abort_events_received += 1;
        test_data.last_abort_cert_status = Some(connection_abort.status.clone());
        test_data.last_abort_cert_info = Some(connection_abort.cert_info.clone());
    }
}

// server stuff
pub fn create_server_app(port: u16) -> App {
    let mut server = App::new();
    server.add_plugins((
        ScheduleRunnerPlugin::default(),
        QuinnetServerPlugin::default(),
    ));
    server
        .insert_resource(ServerData::default())
        .add_systems(Startup, start_listening)
        .add_systems(Update, handle_server_events);
    server.insert_resource(Port(port));
    server.update();
    server
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

pub fn wait_for_client_connected(client: &mut App, server: &mut App) -> ClientId {
    loop {
        std::thread::sleep(std::time::Duration::from_secs_f32(0.05));
        client.update();
        if client
            .world
            .resource::<Client>()
            .connection()
            .is_connected()
        {
            break;
        }
    }
    server.update();
    server
        .world
        .resource::<ServerData>()
        .last_connected_client_id
        .expect("A client should have connected")
}
