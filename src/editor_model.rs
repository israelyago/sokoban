use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};

use crate::level::LevelPack;

pub const COLLECTION_NAME_MAX_CHARS: usize = 64;
pub const MAP_NAME_MAX_CHARS: usize = 64;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EditableMap {
    pub name: String,
    pub raw_lines: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EditableCollection {
    pub source: PathBuf,
    pub collection_name: String,
    pub maps: Vec<EditableMap>,
}

impl EditableCollection {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let source = path.as_ref().to_path_buf();
        let text = fs::read_to_string(&source)
            .with_context(|| format!("failed to read '{}'", source.display()))?;
        let pack = LevelPack::load(&source)?;
        let collection_name = parse_collection_name(&text).unwrap_or_else(|| default_name(&source));

        let maps = pack
            .levels
            .into_iter()
            .enumerate()
            .map(|(idx, raw)| EditableMap {
                name: normalized_existing_map_name(raw.name, idx),
                raw_lines: raw.lines,
            })
            .collect();

        Ok(Self {
            source,
            collection_name,
            maps,
        })
    }

    pub fn create_in_imported_dir(
        imported_dir: impl AsRef<Path>,
        raw_name: &str,
    ) -> anyhow::Result<Self> {
        let imported_dir = imported_dir.as_ref();
        fs::create_dir_all(imported_dir).with_context(|| {
            format!(
                "failed to create imported dir '{}'",
                imported_dir.display()
            )
        })?;

        let file_name = normalize_collection_file_name(raw_name)?;
        let collection_name = normalize_display_name(raw_name, COLLECTION_NAME_MAX_CHARS)?;
        let source = imported_dir.join(file_name);
        if source.exists() {
            bail!("name already exists");
        }

        let collection = Self {
            source,
            collection_name,
            maps: Vec::new(),
        };
        collection.save()?;
        Ok(collection)
    }

    pub fn rename_collection(&mut self, raw_name: &str) -> anyhow::Result<()> {
        self.collection_name = normalize_display_name(raw_name, COLLECTION_NAME_MAX_CHARS)?;
        Ok(())
    }

    pub fn add_map(&mut self, raw_name: &str, raw_lines: Vec<String>) -> anyhow::Result<usize> {
        let name = normalize_map_name(raw_name)?;
        ensure_map_name_is_unique(&self.maps, &name, None)?;
        self.maps.push(EditableMap { name, raw_lines });
        Ok(self.maps.len() - 1)
    }

    pub fn rename_map(&mut self, index: usize, raw_name: &str) -> anyhow::Result<()> {
        if index >= self.maps.len() {
            bail!("map index out of bounds");
        }
        let name = normalize_map_name(raw_name)?;
        ensure_map_name_is_unique(&self.maps, &name, Some(index))?;
        self.maps[index].name = name;
        Ok(())
    }

    pub fn delete_map(&mut self, index: usize) -> anyhow::Result<EditableMap> {
        if index >= self.maps.len() {
            bail!("map index out of bounds");
        }
        Ok(self.maps.remove(index))
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let mut seen = Vec::<String>::new();
        for map in &self.maps {
            let normalized = normalize_map_name(&map.name)?;
            if seen
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&normalized))
            {
                bail!("duplicate map name '{}'", map.name);
            }
            seen.push(normalized);
        }

        let text = self.serialize();
        save_text_atomically(&self.source, &text)
    }

    fn serialize(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Collection: {}\n\n", self.collection_name));
        for map in &self.maps {
            for line in &map.raw_lines {
                out.push_str(line);
                out.push('\n');
            }
            out.push_str(&format!("Title: {}\n\n", map.name));
        }
        out
    }
}

pub fn normalize_collection_file_name(raw_name: &str) -> anyhow::Result<String> {
    let trimmed = raw_name.trim();
    if trimmed.is_empty() {
        bail!("name cannot be empty");
    }

    if trimmed.chars().any(is_invalid_filename_char) {
        bail!("name contains invalid filename characters");
    }

    let without_extension = trimmed
        .strip_suffix(".txt")
        .or_else(|| trimmed.strip_suffix(".TXT"))
        .unwrap_or(trimmed);
    if without_extension.trim().is_empty() {
        bail!("name cannot be empty");
    }
    if without_extension.chars().count() > COLLECTION_NAME_MAX_CHARS {
        bail!(
            "name cannot exceed {} characters",
            COLLECTION_NAME_MAX_CHARS
        );
    }

    let file_name = format!("{without_extension}.txt");
    if file_name.eq_ignore_ascii_case("default.txt") {
        bail!("name 'default' is reserved");
    }

    Ok(file_name)
}

pub fn normalize_map_name(raw_name: &str) -> anyhow::Result<String> {
    let trimmed = raw_name.trim();
    if trimmed.is_empty() {
        bail!("name cannot be empty");
    }
    if trimmed.chars().count() > MAP_NAME_MAX_CHARS {
        bail!("name cannot exceed {} characters", MAP_NAME_MAX_CHARS);
    }
    if trimmed.chars().any(|ch| ch.is_control()) {
        bail!("name contains invalid control characters");
    }
    Ok(trimmed.to_string())
}

fn ensure_map_name_is_unique(
    maps: &[EditableMap],
    candidate: &str,
    skip_index: Option<usize>,
) -> anyhow::Result<()> {
    if maps.iter().enumerate().any(|(idx, map)| {
        Some(idx) != skip_index && map.name.eq_ignore_ascii_case(candidate)
    }) {
        bail!("name already exists");
    }
    Ok(())
}

fn normalize_display_name(raw_name: &str, max_chars: usize) -> anyhow::Result<String> {
    let trimmed = raw_name
        .trim()
        .strip_suffix(".txt")
        .or_else(|| raw_name.trim().strip_suffix(".TXT"))
        .unwrap_or(raw_name.trim())
        .trim();
    if trimmed.is_empty() {
        bail!("name cannot be empty");
    }
    if trimmed.chars().count() > max_chars {
        bail!("name cannot exceed {} characters", max_chars);
    }
    if trimmed.chars().any(|ch| ch.is_control()) {
        bail!("name contains invalid control characters");
    }
    Ok(trimmed.to_string())
}

fn save_text_atomically(path: &Path, text: &str) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .context("target file must have a parent directory")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create '{}'", parent.display()))?;

    let tmp_path = atomic_tmp_path(path);
    fs::write(&tmp_path, text)
        .with_context(|| format!("failed to write '{}'", tmp_path.display()))?;
    fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "failed to move '{}' to '{}'",
            tmp_path.display(),
            path.display()
        )
    })?;
    Ok(())
}

fn atomic_tmp_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("collection.txt");
    let tmp_name = format!("{file_name}.{}.tmp", std::process::id());
    path.with_file_name(tmp_name)
}

fn parse_collection_name(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("Collection:")
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn normalized_existing_map_name(raw_name: Option<String>, index: usize) -> String {
    match raw_name {
        Some(name) => normalize_map_name(&name).unwrap_or_else(|_| format!("Map {}", index + 1)),
        None => format!("Map {}", index + 1),
    }
}

fn default_name(path: &Path) -> String {
    let file_stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("Collection");
    if file_stem.trim().is_empty() {
        "Collection".to_string()
    } else {
        file_stem.trim().to_string()
    }
}

fn is_invalid_filename_char(ch: char) -> bool {
    matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
}

#[cfg(test)]
mod tests {
    use super::EditableCollection;
    use std::path::{Path, PathBuf};

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "sokoban_editor_model_{prefix}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).expect("failed to create temp dir");
        path
    }

    fn write_text(path: &Path, text: &str) {
        std::fs::write(path, text).expect("failed to write test data");
    }

    #[test]
    fn loads_collection_and_map_names() {
        let dir = unique_temp_dir("load");
        let path = dir.join("numbers.txt");
        write_text(
            &path,
            "\
Collection: Numbers

####
#@.#
#$ #
####
Title: One

####
# @#
#$.#
####
Title: Two
",
        );

        let collection = EditableCollection::load(&path).expect("load should work");
        assert_eq!(collection.collection_name, "Numbers");
        assert_eq!(collection.maps.len(), 2);
        assert_eq!(collection.maps[0].name, "One");
        assert_eq!(collection.maps[1].name, "Two");
        assert_eq!(collection.maps[0].raw_lines[0], "####");

        std::fs::remove_dir_all(dir).expect("cleanup should work");
    }

    #[test]
    fn creates_new_collection_file_in_imported_dir() {
        let dir = unique_temp_dir("create");
        let collection = EditableCollection::create_in_imported_dir(&dir, "My Collection")
            .expect("create should work");

        assert!(collection.source.exists());
        assert_eq!(
            collection.source.file_name().and_then(|v| v.to_str()),
            Some("My Collection.txt")
        );
        assert_eq!(collection.collection_name, "My Collection");
        assert!(collection.maps.is_empty());

        let loaded = EditableCollection::load(&collection.source).expect("reload should work");
        assert_eq!(loaded.collection_name, "My Collection");
        assert!(loaded.maps.is_empty());

        std::fs::remove_dir_all(dir).expect("cleanup should work");
    }

    #[test]
    fn add_rename_delete_map_persists() {
        let dir = unique_temp_dir("persist");
        let mut collection = EditableCollection::create_in_imported_dir(&dir, "Pack A")
            .expect("create should work");

        collection
            .add_map(
                "Map 1",
                vec![
                    "#####".to_string(),
                    "#@$.#".to_string(),
                    "#####".to_string(),
                ],
            )
            .expect("add map 1 should work");
        collection
            .add_map(
                "Map 2",
                vec![
                    "#####".to_string(),
                    "#@$.#".to_string(),
                    "#####".to_string(),
                ],
            )
            .expect("add map 2 should work");
        collection
            .rename_map(0, "Map One")
            .expect("rename should work");
        collection.delete_map(1).expect("delete should work");
        collection.save().expect("save should work");

        let reloaded = EditableCollection::load(&collection.source).expect("reload should work");
        assert_eq!(reloaded.maps.len(), 1);
        assert_eq!(reloaded.maps[0].name, "Map One");

        std::fs::remove_dir_all(dir).expect("cleanup should work");
    }

    #[test]
    fn save_replaces_previous_content() {
        let dir = unique_temp_dir("atomic");
        let path = dir.join("pack.txt");
        write_text(
            &path,
            "\
Collection: Old Name

####
#@.#
#$ #
####
Title: Old Map
",
        );

        let mut collection = EditableCollection::load(&path).expect("load should work");
        collection
            .rename_collection("New Name")
            .expect("rename collection should work");
        collection
            .rename_map(0, "New Map")
            .expect("rename map should work");
        collection.save().expect("save should work");

        let text = std::fs::read_to_string(&path).expect("file should be readable");
        assert!(text.contains("Collection: New Name"));
        assert!(text.contains("Title: New Map"));
        assert!(!text.contains("Collection: Old Name"));
        assert!(!text.contains("Title: Old Map"));

        std::fs::remove_dir_all(dir).expect("cleanup should work");
    }
}
