//! Identificadores: UUID v7 (ordenáveis por tempo) gravados como texto.

pub type Id = String;

pub fn new_id() -> Id {
    uuid::Uuid::now_v7().to_string()
}

pub fn is_valid_id(s: &str) -> bool {
    uuid::Uuid::parse_str(s).is_ok()
}
