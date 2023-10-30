use bevy::app::App;
use bevy::DefaultPlugins;
use quic_game::server::ServerPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        ServerPlugin::new(5000))
    );
    app.run();
}