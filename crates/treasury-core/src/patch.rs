//! Utilitários para DTOs de atualização parcial ("patch").
//!
//! Campos `Option<Option<T>>` distinguem "não enviado" (`None`) de "limpar" (`Some(None)`,
//! JSON `null`). Use sempre com `#[serde(default, deserialize_with = "crate::patch::double")]`.

use serde::{Deserialize, Deserializer};

pub fn double<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(de).map(Some)
}

#[cfg(test)]
mod tests {
    #[derive(serde::Deserialize)]
    struct P {
        #[serde(default, deserialize_with = "super::double")]
        color: Option<Option<String>>,
    }

    #[test]
    fn distinguishes_missing_and_null() {
        let a: P = serde_json::from_str("{}").unwrap();
        let b: P = serde_json::from_str(r#"{"color": null}"#).unwrap();
        let c: P = serde_json::from_str(r#"{"color": "x"}"#).unwrap();
        assert_eq!(a.color, None);
        assert_eq!(b.color, Some(None));
        assert_eq!(c.color, Some(Some("x".into())));
    }
}
