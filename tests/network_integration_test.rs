use bevy::app::{App, ScheduleRunnerPlugin};
use bevy_quinnet::shared::ClientId;
use bevy_quinnet::client::Client;
use quic_game::client::ClientPlugin;
use quic_game::server::{ServerData, ServerPlugin};

#[cfg(test)]
mod game_connection_tests {
    use super::*;
    use quic_game::client::ClientData;
    use quic_game::shared::SharedMessage;

    #[test]
    fn it_connects_to_server() {
        const SERVER_PORT: u16 = 5050;
        let mut server = create_server_app(SERVER_PORT);
        let mut client = create_client_app(SERVER_PORT);
        {
            let client = client.world.resource::<Client>();
            assert!(
                client.get_connection().is_some(),
                "The default connection should exist"
            );
            let server = server.world.resource::<bevy_quinnet::server::Server>();
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
                .resource_mut::<bevy_quinnet::server::Server>()
                .endpoint_mut()
                .receive_message_from::<SharedMessage>(client_id)
                .expect("Failed to receive client.rs message")
                .expect("There should be a client.rs message");
            assert_eq!(client_message, sent_client_message);
        }

        let sent_server_message = SharedMessage::TestMessage("Server response".to_string());
        {
            let server = server.world.resource::<bevy_quinnet::server::Server>();
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

pub fn create_client_app(port: u16) -> App {
    let mut client = App::new();
    client.add_plugins((ScheduleRunnerPlugin::default(), ClientPlugin::new(port)));
    client.update();
    client
}

pub fn create_server_app(port: u16) -> App {
    let mut server = App::new();
    server.add_plugins((ScheduleRunnerPlugin::default(), ServerPlugin::new(port)));
    server.update();
    server
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
        .expect("A client.rs should have connected")
}
