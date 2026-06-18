use crate::types::PersistState;
use std::fs;
use std::path::PathBuf;

// Хранилище состояния в JSON-файле (~/Library/Application Support/opdeck/deck-state.json).
pub struct StateStore {
    pub data: PersistState,
    pub path: PathBuf,
}

fn data_root() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn state_path() -> PathBuf {
    // Папку можно переопределить при сборке (env DECK_STATE_DIR) — так у копии «opdeck dev»
    // отдельное состояние и она не мешает рабочему экземпляру. По умолчанию — "opdeck".
    let folder = option_env!("DECK_STATE_DIR").unwrap_or("opdeck");
    let dir = data_root().join(folder);
    let _ = fs::create_dir_all(&dir);
    dir.join("deck-state.json")
}

// Старый путь (имя Pult) — для разовой миграции воркспейсов.
fn legacy_path() -> PathBuf {
    data_root().join("pult").join("pilotry-state.json")
}

// Прежнее имя папки — «Deck» (до переименования в opdeck). Читаем для разовой миграции
// ТОЛЬКО в прод-сборке: у dev задан DECK_STATE_DIR, и фолбэк на «Deck» подхватил бы прод-данные.
fn deck_legacy_path() -> Option<PathBuf> {
    if option_env!("DECK_STATE_DIR").is_some() {
        return None;
    }
    Some(data_root().join("Deck").join("deck-state.json"))
}

impl StateStore {
    pub fn load() -> Self {
        let path = state_path();
        // читаем новый файл; если его нет — мигрируем из «Deck», затем из «Pult».
        let mut raw = fs::read_to_string(&path).ok();
        if raw.is_none() {
            if let Some(p) = deck_legacy_path() {
                raw = fs::read_to_string(p).ok();
            }
        }
        let raw = raw
            .or_else(|| fs::read_to_string(legacy_path()).ok())
            .unwrap_or_default();
        let data = serde_json::from_str(&raw).unwrap_or_default();
        StateStore { data, path }
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            let _ = fs::write(&self.path, json);
        }
    }
}
