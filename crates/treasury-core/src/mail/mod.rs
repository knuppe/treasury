//! E-mail de ingestão (docs/email-ingest.md): uma caixa exclusiva da família (IMAP com senha
//! de app, alimentada por encaminhamento) ou uma pasta local vigiada. O que passa nos filtros
//! é identificado pelos layouts (ou pela IA, se a regra permitir), vai para a fila de revisão
//! em 04 · Lançamentos e, se configurado, sai do provedor depois (retenção, com modo de teste).
//! Sem SMTP e sem servidor: só roda com o app aberto e o cofre desbloqueado.

pub mod ai;
mod api;
pub mod auth;
pub mod body;
pub mod config;
pub mod filter;
pub mod imap;
pub mod ingest;
pub mod local;
pub mod message;
pub mod providers;
pub mod retention;
pub mod store;
pub mod sync;

pub use ai::AI_CLIENT_PREFIX;
pub use api::{API_LEVEL, MailRuleTestArgs, READ_ONLY, handle, handle_api_level};
pub use sync::run_due;

#[cfg(test)]
mod tests;
