use std::{
    env,
    path::{Path, PathBuf},
};

use bevy::prelude::Resource;

const SOKOBAN_ASSET_DIR_ENV: &str = "SOKOBAN_ASSET_DIR";
const SOKOBAN_LEVELS_DIR_ENV: &str = "SOKOBAN_LEVELS_DIR";
const SOKOBAN_USER_DATA_DIR_ENV: &str = "SOKOBAN_USER_DATA_DIR";
const XDG_DATA_HOME_ENV: &str = "XDG_DATA_HOME";
const HOME_ENV: &str = "HOME";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BuildProfile {
    Debug,
    Release,
}

impl BuildProfile {
    fn current() -> Self {
        if cfg!(debug_assertions) {
            Self::Debug
        } else {
            Self::Release
        }
    }
}

#[derive(Resource, Debug, Clone, Eq, PartialEq)]
pub struct PathConfig {
    pub build_profile: BuildProfile,
    pub asset_root: PathBuf,
    pub levels_dir: PathBuf,
    pub builtin_default_pack: PathBuf,
    pub user_data_dir: PathBuf,
    pub imported_dir: PathBuf,
}

#[derive(Debug, Clone, Default)]
struct EnvOverrides {
    asset_dir: Option<PathBuf>,
    levels_dir: Option<PathBuf>,
    user_data_dir: Option<PathBuf>,
    xdg_data_home: Option<PathBuf>,
    home_dir: Option<PathBuf>,
}

pub fn resolve_path_config() -> PathConfig {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let env = EnvOverrides {
        asset_dir: env_path(SOKOBAN_ASSET_DIR_ENV),
        levels_dir: env_path(SOKOBAN_LEVELS_DIR_ENV),
        user_data_dir: env_path(SOKOBAN_USER_DATA_DIR_ENV),
        xdg_data_home: env_path(XDG_DATA_HOME_ENV),
        home_dir: env_path(HOME_ENV),
    };
    resolve_with_env(BuildProfile::current(), &cwd, &env)
}

fn resolve_with_env(profile: BuildProfile, cwd: &Path, env: &EnvOverrides) -> PathConfig {
    let (default_asset_root, default_levels_dir, default_user_data_dir) = match profile {
        BuildProfile::Debug => (cwd.join("assets"), cwd.join("levels"), cwd.join("levels")),
        BuildProfile::Release => (
            PathBuf::from("/usr/share/sokoban/assets"),
            PathBuf::from("/usr/share/sokoban/levels"),
            default_release_user_data_dir(env),
        ),
    };

    let asset_root = env
        .asset_dir
        .clone()
        .unwrap_or_else(|| default_asset_root.clone());
    let levels_dir = env
        .levels_dir
        .clone()
        .unwrap_or_else(|| default_levels_dir.clone());
    let user_data_dir = env
        .user_data_dir
        .clone()
        .unwrap_or_else(|| default_user_data_dir.clone());

    PathConfig {
        build_profile: profile,
        asset_root,
        builtin_default_pack: levels_dir.join("default.txt"),
        levels_dir,
        imported_dir: user_data_dir.join("imported"),
        user_data_dir,
    }
}

fn default_release_user_data_dir(env: &EnvOverrides) -> PathBuf {
    if let Some(path) = &env.xdg_data_home {
        return path.join("sokoban");
    }
    if let Some(path) = &env.home_dir {
        return path.join(".local/share/sokoban");
    }
    PathBuf::from(".local/share/sokoban")
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_defaults_use_repo_relative_paths() {
        let cwd = PathBuf::from("/repo/sokoban");
        let cfg = resolve_with_env(BuildProfile::Debug, &cwd, &EnvOverrides::default());
        assert_eq!(cfg.asset_root, PathBuf::from("/repo/sokoban/assets"));
        assert_eq!(cfg.levels_dir, PathBuf::from("/repo/sokoban/levels"));
        assert_eq!(
            cfg.builtin_default_pack,
            PathBuf::from("/repo/sokoban/levels/default.txt")
        );
        assert_eq!(cfg.user_data_dir, PathBuf::from("/repo/sokoban/levels"));
        assert_eq!(
            cfg.imported_dir,
            PathBuf::from("/repo/sokoban/levels/imported")
        );
    }

    #[test]
    fn release_defaults_use_system_and_xdg_paths() {
        let cwd = PathBuf::from("/repo/sokoban");
        let env = EnvOverrides {
            xdg_data_home: Some(PathBuf::from("/home/alex/.xdg-data")),
            ..EnvOverrides::default()
        };
        let cfg = resolve_with_env(BuildProfile::Release, &cwd, &env);
        assert_eq!(cfg.asset_root, PathBuf::from("/usr/share/sokoban/assets"));
        assert_eq!(cfg.levels_dir, PathBuf::from("/usr/share/sokoban/levels"));
        assert_eq!(
            cfg.builtin_default_pack,
            PathBuf::from("/usr/share/sokoban/levels/default.txt")
        );
        assert_eq!(
            cfg.user_data_dir,
            PathBuf::from("/home/alex/.xdg-data/sokoban")
        );
        assert_eq!(
            cfg.imported_dir,
            PathBuf::from("/home/alex/.xdg-data/sokoban/imported")
        );
    }

    #[test]
    fn release_falls_back_to_home_when_xdg_missing() {
        let cwd = PathBuf::from("/repo/sokoban");
        let env = EnvOverrides {
            home_dir: Some(PathBuf::from("/home/alex")),
            ..EnvOverrides::default()
        };
        let cfg = resolve_with_env(BuildProfile::Release, &cwd, &env);
        assert_eq!(
            cfg.user_data_dir,
            PathBuf::from("/home/alex/.local/share/sokoban")
        );
        assert_eq!(
            cfg.imported_dir,
            PathBuf::from("/home/alex/.local/share/sokoban/imported")
        );
    }

    #[test]
    fn explicit_env_overrides_take_precedence() {
        let cwd = PathBuf::from("/repo/sokoban");
        let env = EnvOverrides {
            asset_dir: Some(PathBuf::from("/opt/sokoban/assets")),
            levels_dir: Some(PathBuf::from("/opt/sokoban/levels")),
            user_data_dir: Some(PathBuf::from("/var/lib/sokoban-user")),
            xdg_data_home: Some(PathBuf::from("/home/alex/.xdg-data")),
            home_dir: Some(PathBuf::from("/home/alex")),
        };
        let cfg = resolve_with_env(BuildProfile::Release, &cwd, &env);
        assert_eq!(cfg.asset_root, PathBuf::from("/opt/sokoban/assets"));
        assert_eq!(cfg.levels_dir, PathBuf::from("/opt/sokoban/levels"));
        assert_eq!(
            cfg.builtin_default_pack,
            PathBuf::from("/opt/sokoban/levels/default.txt")
        );
        assert_eq!(cfg.user_data_dir, PathBuf::from("/var/lib/sokoban-user"));
        assert_eq!(
            cfg.imported_dir,
            PathBuf::from("/var/lib/sokoban-user/imported")
        );
    }
}
