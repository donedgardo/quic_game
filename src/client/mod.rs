use bevy::prelude::{EventReader, Res, ResMut, Resource};
use bevy_quinnet::client::certificate::{CertConnectionAbortEvent, CertificateVerificationMode, CertInteractionEvent, CertTrustUpdateEvent, CertVerificationInfo, CertVerificationStatus, CertVerifierAction};
use std::net::{IpAddr, Ipv4Addr};
use bevy::app::{App, Plugin, Startup, Update};
use bevy_quinnet::client::{Client, QuinnetClientPlugin};
use bevy_quinnet::client::connection::ConnectionConfiguration;
use crate::shared::Port;
use crate::server::SERVER_IP;

pub const LOCAL_BIND_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

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

//client.rs stuff
pub struct ClientPlugin {
    port: u16,
}

impl ClientPlugin {
    pub fn new(port: u16) -> Self {
        Self {
            port
        }
    }
}

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(QuinnetClientPlugin::default())
            .insert_resource(ClientData::default())
            .insert_resource(Port(self.port))
            .add_systems(Startup, start_simple_connection)
            .add_systems(Update, handle_client_events);

    }
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
