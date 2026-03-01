use anyhow::Context;
use bevy::{asset::AssetPlugin, prelude::*, window::WindowResolution};
use std::fs;

fn main() -> anyhow::Result<()> {
    let path_config = sokoban::paths::resolve_path_config();
    fs::create_dir_all(&path_config.imported_dir).with_context(|| {
        format!(
            "failed to create imported levels directory '{}'",
            path_config.imported_dir.display()
        )
    })?;
    let asset_file_path = path_config.asset_root.to_string_lossy().into_owned();
    let default_pack_path = path_config
        .builtin_default_pack
        .to_string_lossy()
        .into_owned();
    println!(
        "sokoban paths: profile={:?} asset_root={} levels_dir={} default_pack={} user_data_dir={} imported_dir={}",
        path_config.build_profile,
        path_config.asset_root.display(),
        path_config.levels_dir.display(),
        path_config.builtin_default_pack.display(),
        path_config.user_data_dir.display(),
        path_config.imported_dir.display(),
    );

    App::new()
        .insert_resource(path_config)
        .insert_resource(sokoban::game::StartupConfig {
            pack_path: default_pack_path,
            start_level: 1,
        })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_file_path,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Sokoban".to_string(),
                        name: Some("sokoban".to_string()),
                        resolution: WindowResolution::new(1024.0, 768.0),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(sokoban::game::GamePlugin)
        .run();
    Ok(())
}
