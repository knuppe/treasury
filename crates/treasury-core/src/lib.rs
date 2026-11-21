//! Núcleo do Treasury: domínio, cálculos, armazenamento cifrado, histórico de alterações,
//! backups, API para a interface e servidor MCP.

pub mod api;
pub mod backup;
pub mod changes;
pub mod config;
pub mod country;
pub mod datasets;
pub mod error;
pub mod events;
pub mod finance;
pub mod i18n;
pub mod ids;
pub mod keychain;
pub mod maintenance;
pub mod mcp;
pub mod money;
pub mod patch;
pub mod paths;
pub mod settings;
pub mod sources;
pub mod storage;
pub mod support;
pub mod tasks;
pub mod terms;
pub mod time;
pub mod trash;
pub mod unlock;
pub mod vault;
pub mod vault_versions;
pub mod vaults;

// Domínio — fase 1 (fluxo de caixa familiar).
pub mod accounts;
pub mod attention;
pub mod budget;
pub mod cards;
pub mod categories;
pub mod export;
pub mod fgts;
pub mod food;
pub mod fx;
pub mod goals;
pub mod household;
pub mod import;
pub mod insights;
// Linha da vida (docs/life-expectancy.md).
pub mod longevity;
pub mod mail;
pub mod receipts;
pub mod recurrence;
pub mod reports;
pub mod retirement;
pub mod rules;
pub mod transactions;

// Domínio — fase 2 (índices e carteira).
pub mod assets;
pub mod indices;
pub mod market;
pub mod portfolio;

// Domínio — fase 3 (análise de ativos e IR).
pub mod research;
pub mod score;
pub mod tax;
pub mod tax_burden;

// Fase 5 (dívidas e financiamentos).
pub mod debt;
pub mod demo;
pub mod forecast;
pub mod income;
pub mod installments;
pub mod notify;
pub mod scenarios;
pub mod subscriptions;

// Patrimônio físico (imóveis, veículos, bens).
pub mod layouts;
pub mod ocr;
pub mod payments;
pub mod pdf;
pub mod pdf_markdown;
pub mod pension;
pub mod physical;
pub mod projects;

// Saúde financeira (I-SFB e indicadores clássicos).
pub mod health;

// Fase 4 (agente de IA).
pub mod ai;

// Perfil de quem usa o app: nível, tom e o que a interface mostra.
pub mod profile;

// Feedback do agente para o desenvolvedor (modo desenvolvedor).
pub mod devfeedback;

pub use api::{Actor, Api, SharedApi};
pub use error::{ApiError, Error, Result};
