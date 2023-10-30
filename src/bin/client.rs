use bevy::app::App;
use bevy::DefaultPlugins;
use quicc_game::client::ClientPlugin;

fn main () {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        ClientPlugin::new(5000)
    ));
    app.run();
}
