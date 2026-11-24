//! Armazenamento: SQLCipher em arquivo único, journal sem WAL, migrações embutidas.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags, OptionalExtension};

use crate::error::{Error, Result};
use crate::vault::{self, DbKey, KdfParams};

mod embedded {
    refinery::embed_migrations!("migrations");
}

/// Migrações que reconstroem tabelas referenciadas por chaves estrangeiras: rodam com
/// `foreign_keys = OFF` e são conferidas com `PRAGMA foreign_key_check` em seguida.
const FK_OFF_MIGRATIONS: &[i32] = &[];

/// O histórico das migrações de antes da 1.0.0 (V1 a V164), juntadas na V1 atual
/// (`migrations/V1__esquema.sql`). Um cofre com exatamente este histórico tem o esquema da V1 e
/// só troca o histórico ao abrir ([`Db::migrate`]); com parte dele, foi gravado por uma versão
/// mais antiga que a 0.9.38 e precisa ser aberto nela antes.
const LEGACY_HISTORY: &[(i32, &str)] = &[
    (1, "fundacao"),
    (2, "fluxo_de_caixa"),
    (10, "indices_do_livro"),
    (20, "acertos_de_regras"),
    (40, "auditoria_ia"),
    (50, "indices_e_carteira"),
    (51, "benchmarks"),
    (52, "apuracao_ir"),
    (60, "analise_de_ativos"),
    (70, "propostas_e_memoria_ia"),
    (71, "motores_de_ia"),
    (80, "navegador_assistido"),
    (81, "dividas"),
    (82, "patrimonio_fisico"),
    (84, "provedores_globais"),
    (85, "bolsas_e_intraday"),
    (86, "chaves_mcp"),
    (87, "memoria_de_procedimentos"),
    (89, "numero_do_contrato"),
    (90, "emprestimos_pos_fixados"),
    (91, "correcao_dos_contratos"),
    (92, "anexos_e_retencao_do_agente"),
    (93, "cheque_especial"),
    (94, "bancos_e_conta"),
    (95, "seguros_do_financiamento"),
    (96, "cache_do_fundamentus"),
    (97, "sincronia_cambio_e_cotas"),
    (98, "feedback_do_agente"),
    (99, "dados_do_score"),
    (100, "score_de_ativos"),
    (101, "proventos_oficiais"),
    (102, "previdencia"),
    (103, "seguros_fixos_do_financiamento"),
    (104, "extrato_do_contrato"),
    (105, "projetos"),
    (106, "itens_do_projeto"),
    (107, "propostas_retiradas_e_revisao"),
    (108, "remove_navegador"),
    (109, "estimativa_de_impostos"),
    (110, "remove_ia_local"),
    (111, "alertas_de_score_uteis"),
    (112, "conta_de_beneficio"),
    (113, "previdencia_fechada"),
    (114, "fatura_pdf_e_amortizacao"),
    (115, "memoria_em_grafo"),
    (116, "pasta_do_google_drive"),
    (117, "remove_pasta_do_google_drive"),
    (118, "indices_de_desempenho"),
    (119, "juros_de_mercado_e_ipca_por_grupo"),
    (120, "rendas_e_holerites"),
    (121, "vinculos_de_lancamentos"),
    (122, "membro_herdado_da_conta"),
    (123, "pais_da_familia"),
    (124, "cupons_fiscais"),
    (125, "email_de_ingestao"),
    (126, "iptu"),
    (127, "chave_das_categorias"),
    (128, "chave_dos_alertas"),
    (129, "grupo_das_categorias"),
    (130, "dados_dos_ativos"),
    (131, "sexo_do_membro"),
    (132, "estudos_de_ativos"),
    (133, "limite_com_indexador"),
    (134, "contas_do_fgts"),
    (135, "alimentacao"),
    (136, "consumo"),
    (137, "contas_de_aposentadoria"),
    (138, "gtin_dos_produtos"),
    (139, "mcp_chave_obrigatoria"),
    (140, "nome_oficial_do_ativo"),
    (141, "aportes_de_previdencia"),
    (142, "pendencias_dos_aportes"),
    (143, "score_do_mercado"),
    (144, "conversa_do_ativo"),
    (145, "alertas_de_tela"),
    (146, "pedido_de_estudo"),
    (147, "conta_de_custodia"),
    (148, "meta_de_faculdade"),
    (149, "renda_fixa_rdb"),
    (150, "quadro_de_feedback"),
    (151, "meta_com_rendimento"),
    (152, "metodo_de_custo"),
    (153, "aportes_programados"),
    (154, "mcp_extras"),
    (155, "cartoes_adicionais"),
    (156, "limite_ciclo_e_anuidade"),
    (157, "encargos_e_compras_no_exterior"),
    (158, "cashback"),
    (159, "pontos_e_milhas"),
    (160, "fontes_da_meta"),
    (161, "etiquetas_de_ativos"),
    (162, "caixa_de_entrada"),
    (163, "parcelas_detectadas"),
    (164, "series_de_alimentos"),
];

/// O histórico de migrações do banco é o de antes da 1.0.0?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Legacy {
    /// Histórico atual (ou banco novo).
    No,
    /// Exatamente o histórico da 0.9.38: o esquema é o da V1, só o histórico muda.
    Complete,
    /// Parte dele: gravado por uma versão anterior à 0.9.38.
    Partial,
}

/// O histórico do banco é o de antes da 1.0.0 (a primeira migração antiga era a `fundacao`)?
/// Usado também para conferir backups antes de restaurar.
pub(crate) fn is_legacy_history(conn: &Connection) -> Result<bool> {
    Ok(legacy_of(&history(conn)?) != Legacy::No)
}

/// Versões e nomes do histórico do refinery, em ordem (vazio sem a tabela).
fn history(conn: &Connection) -> Result<Vec<(i32, String)>> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'refinery_schema_history')",
        [],
        |r| r.get(0),
    )?;
    if !exists {
        return Ok(vec![]);
    }
    let mut stmt =
        conn.prepare("SELECT version, name FROM refinery_schema_history ORDER BY version")?;
    Ok(stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?)
}

fn legacy_of(h: &[(i32, String)]) -> Legacy {
    if !h.iter().any(|(v, n)| *v == 1 && n == "fundacao") {
        return Legacy::No;
    }
    let complete = h.len() == LEGACY_HISTORY.len()
        && h.iter()
            .zip(LEGACY_HISTORY)
            .all(|((v, n), (lv, ln))| v == lv && n == ln);
    if complete {
        Legacy::Complete
    } else {
        Legacy::Partial
    }
}

/// A V1 embutida.
fn v1_migration() -> Result<refinery::Migration> {
    embedded::migrations::runner()
        .get_migrations()
        .iter()
        .find(|m| m.version() == 1)
        .cloned()
        .ok_or_else(|| Error::Migration("a V1 não está embutida".into()))
}

/// Objetos do esquema (tipo, nome, SQL) que a V1 cria, num banco em memória.
fn v1_objects() -> Result<Vec<(String, String, String)>> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch(v1_migration()?.sql().unwrap_or_default())?;
    objects(&conn)
}

fn objects(conn: &Connection) -> Result<Vec<(String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT type, name, sql FROM sqlite_master
         WHERE sql IS NOT NULL AND name NOT LIKE 'sqlite_%' AND name <> 'refinery_schema_history'
         ORDER BY type, name",
    )?;
    Ok(stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?)
}

/// Situação do esquema de um banco em relação às migrações embutidas neste binário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaStatus {
    pub applied: Vec<i32>,
    pub pending: Vec<i32>,
    pub unknown: Vec<i32>,
}

fn embedded_versions() -> Vec<i32> {
    let mut v: Vec<i32> = embedded::migrations::runner()
        .get_migrations()
        .iter()
        .map(|m| m.version())
        .collect();
    v.sort_unstable();
    v
}

/// Versão mais nova do esquema embutida neste binário.
pub fn latest_schema_version() -> i32 {
    embedded::migrations::runner()
        .get_migrations()
        .iter()
        .map(|m| m.version())
        .max()
        .unwrap_or(0)
}

pub struct Db {
    conn: Connection,
    key: DbKey,
    path: PathBuf,
    read_only: bool,
}

impl std::fmt::Debug for Db {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Db")
            .field("path", &self.path)
            .field("read_only", &self.read_only)
            .finish()
    }
}

impl Db {
    /// Cria um banco novo, cifrado, com as migrações aplicadas.
    pub fn create(path: &Path, password: &str, kdf: KdfParams) -> Result<Self> {
        if path.exists() {
            return Err(Error::VaultExists(path.display().to_string()));
        }
        if let Some(dir) = path.parent() {
            crate::paths::create_private_dir(dir)?;
        }
        let salt = DbKey::random_salt()?;
        let key = DbKey::derive(password, salt, kdf)?;
        let conn = Connection::open(path)?;
        crate::paths::set_mode(path, 0o600)?;
        apply_key(&conn, &key)?;
        let mut db = Self {
            conn,
            key,
            path: path.to_path_buf(),
            read_only: false,
        };
        db.configure()?;
        db.migrate()?;
        Ok(db)
    }

    /// Abre com a senha mestra. Tenta as versões de KDF conhecidas.
    pub fn open(path: &Path, password: &str) -> Result<Self> {
        if !path.exists() {
            return Err(Error::NoVault);
        }
        let salt = vault::read_salt(path)?;
        for params in vault::kdf_versions() {
            let key = DbKey::derive(password, salt, params)?;
            match Self::open_with_key(path, key, false) {
                Ok(db) => return Ok(db),
                Err(Error::WrongPassword) => continue,
                Err(e) => return Err(e),
            }
        }
        Err(Error::WrongPassword)
    }

    /// Abre com uma chave já derivada (keychain, kit de recuperação, testes).
    pub fn open_with_key(path: &Path, key: DbKey, read_only: bool) -> Result<Self> {
        if !path.exists() {
            return Err(Error::NoVault);
        }
        let flags = if read_only {
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX
        } else {
            // CREATE é necessário para o ATTACH criar arquivos (backup com senha própria);
            // o banco em si já existe (conferido acima).
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
        };
        let conn = Connection::open_with_flags(path, flags)?;
        apply_key(&conn, &key)?;
        if !key_is_valid(&conn) {
            return Err(Error::WrongPassword);
        }
        let mut db = Self {
            conn,
            key,
            path: path.to_path_buf(),
            read_only,
        };
        db.configure()?;
        db.check_compatible()?;
        Ok(db)
    }

    fn configure(&mut self) -> Result<()> {
        self.conn.busy_timeout(std::time::Duration::from_secs(5))?;
        self.conn.pragma_update(None, "foreign_keys", "ON")?;
        // Build de release: o SQLCipher zera toda memória que libera e trava as páginas
        // decifradas (docs/local-security.md, item 8). Custa desempenho, por isso fica fora
        // do build de desenvolvimento e dos testes.
        if !cfg!(debug_assertions)
            && let Err(e) = self
                .conn
                .execute_batch("PRAGMA cipher_memory_security = ON")
        {
            tracing::warn!("cipher_memory_security: {e}");
        }
        if !self.read_only {
            // Sem WAL: em repouso existe um único arquivo (D12).
            let _: String = self
                .conn
                .query_row("PRAGMA journal_mode = DELETE", [], |r| r.get(0))?;
            self.conn.pragma_update(None, "secure_delete", "ON")?;
        }
        Ok(())
    }

    /// Versão atual do esquema gravada no banco (0 se nenhuma migração rodou).
    pub fn schema_version(&self) -> Result<i32> {
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'refinery_schema_history')",
            [],
            |r| r.get(0),
        )?;
        if !exists {
            return Ok(0);
        }
        Ok(self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM refinery_schema_history",
            [],
            |r| r.get(0),
        )?)
    }

    /// Migrações aplicadas, pendentes e desconhecidas (aplicadas por uma versão mais nova). Um
    /// histórico de antes da 1.0.0 conta como aplicado e deixa a V1 pendente: ao migrar, ele é
    /// trocado pela V1 ([`Db::migrate`]).
    pub fn schema_status(&self) -> Result<SchemaStatus> {
        let h = history(&self.conn)?;
        if legacy_of(&h) != Legacy::No {
            return Ok(SchemaStatus {
                applied: h.iter().map(|(v, _)| *v).collect(),
                pending: embedded_versions(),
                unknown: vec![],
            });
        }
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'refinery_schema_history')",
            [],
            |r| r.get(0),
        )?;
        let applied: Vec<i32> = if exists {
            let mut stmt = self
                .conn
                .prepare("SELECT version FROM refinery_schema_history ORDER BY version")?;
            stmt.query_map([], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?
        } else {
            vec![]
        };
        let known = embedded_versions();
        Ok(SchemaStatus {
            pending: known
                .iter()
                .copied()
                .filter(|v| !applied.contains(v))
                .collect(),
            unknown: applied
                .iter()
                .copied()
                .filter(|v| !known.contains(v))
                .collect(),
            applied,
        })
    }

    pub fn needs_migration(&self) -> Result<bool> {
        Ok(!self.schema_status()?.pending.is_empty())
    }

    /// Recusa bancos gravados por uma versão mais nova do app: só se evolui para a frente;
    /// voltar de versão é restaurar um backup feito pela versão antiga.
    pub fn check_compatible(&self) -> Result<()> {
        if legacy_of(&history(&self.conn)?) == Legacy::Partial {
            return Err(Error::Migration(crate::i18n::t(
                "nucleo-base-erro-esquema-antigo",
            )));
        }
        let st = self.schema_status()?;
        if !st.unknown.is_empty() {
            return Err(Error::SchemaTooNew(crate::i18n::t_args(
                "nucleo-base-erro-esquema-novo",
                &crate::i18n::args(&[
                    (
                        "esquema",
                        st.applied.last().copied().unwrap_or(0).to_string().into(),
                    ),
                    ("conhecido", latest_schema_version().to_string().into()),
                ]),
            )));
        }
        Ok(())
    }

    /// Aplica as migrações pendentes, sempre para a frente e numa transação por migração.
    pub fn migrate(&mut self) -> Result<()> {
        if self.read_only {
            return Err(Error::Forbidden(crate::i18n::t(
                "nucleo-base-erro-banco-leitura",
            )));
        }
        self.check_compatible()?;
        if legacy_of(&history(&self.conn)?) == Legacy::Complete {
            self.rebaseline_legacy()?;
        }
        let st = self.schema_status()?;
        let max_applied = st.applied.last().copied().unwrap_or(0);
        if let Some(v) = st.pending.iter().find(|v| **v < max_applied) {
            return Err(Error::Migration(format!(
                "a migração V{v} é anterior à última aplicada (V{max_applied}); migrações novas precisam de número maior que o da última publicada"
            )));
        }
        // Migrações que reconstroem tabelas rodam com as chaves estrangeiras desligadas
        // (procedimento de 12 passos do SQLite): o PRAGMA não vale dentro da transação de
        // cada migração, por isso o executor para antes delas.
        let run = |conn: &mut Connection, target: refinery::Target| {
            embedded::migrations::runner()
                .set_target(target)
                .run(conn)
                .map(|_| ())
                .map_err(|e| Error::Migration(e.to_string()))
        };
        for v in FK_OFF_MIGRATIONS {
            if !st.pending.contains(v) {
                continue;
            }
            run(&mut self.conn, refinery::Target::Version((*v - 1) as _))?;
            self.conn.pragma_update(None, "foreign_keys", "OFF")?;
            let res = run(&mut self.conn, refinery::Target::Version(*v as _));
            self.conn.pragma_update(None, "foreign_keys", "ON")?;
            res?;
            let broken: Option<String> = self
                .conn
                .query_row("PRAGMA foreign_key_check", [], |r| r.get(0))
                .optional()?;
            if let Some(t) = broken {
                return Err(Error::Migration(format!(
                    "a migração V{v} deixou referências quebradas na tabela {t}"
                )));
            }
        }
        run(&mut self.conn, refinery::Target::Latest)
    }

    /// Cofre da 0.9.38 (histórico V1 a V164): confere que cada objeto da V1 existe com o mesmo
    /// SQL e troca o histórico por uma linha da V1, numa transação. Tabelas e dados não mudam.
    /// O backup verificado antes da migração (`api::system::open_session`) cobre esta troca.
    fn rebaseline_legacy(&mut self) -> Result<()> {
        let have: std::collections::HashSet<(String, String, String)> =
            objects(&self.conn)?.into_iter().collect();
        if let Some((kind, name, _)) = v1_objects()?.into_iter().find(|o| !have.contains(o)) {
            return Err(Error::Migration(crate::i18n::t_args(
                "nucleo-base-erro-esquema-diferente",
                &crate::i18n::args(&[("objeto", format!("{kind} {name}").into())]),
            )));
        }
        let v1 = v1_migration()?;
        let applied_on = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM refinery_schema_history", [])?;
        tx.execute(
            "INSERT INTO refinery_schema_history (version, name, applied_on, checksum)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                v1.version(),
                v1.name(),
                applied_on,
                v1.checksum().to_string()
            ],
        )?;
        tx.commit()?;
        tracing::info!("cofre da 0.9.38: histórico de migrações trocado pela V1");
        Ok(())
    }

    /// Troca a senha mestra recifrando o banco (`PRAGMA rekey`). O salt do arquivo é mantido.
    pub fn change_password(&mut self, new_password: &str, kdf: KdfParams) -> Result<()> {
        let new_key = DbKey::derive(new_password, self.key.salt(), kdf)?;
        self.rekey(new_key)
    }

    pub fn rekey(&mut self, new_key: DbKey) -> Result<()> {
        if new_key.salt() != self.key.salt() {
            return Err(Error::Crypto(
                "rekey precisa manter o salt do arquivo".into(),
            ));
        }
        let v = new_key.pragma_value();
        self.conn.pragma_update(None, "rekey", v.as_str())?;
        // Segredos cifrados em nível de campo usam subchave da chave do banco: recifra.
        if let Err(e) = crate::ai::secret::rewrap(&self.conn, &[&self.key], &new_key) {
            tracing::warn!(
                "segredos: falha ao recifrar com a chave nova ({})",
                e.code()
            );
        }
        self.key = new_key;
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    pub fn key(&self) -> &DbKey {
        &self.key
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Move o banco recém-criado para o lugar definitivo e reabre com a mesma chave. A criação
    /// é feita num arquivo à parte: se o app for fechado no meio dela, nada fica no lugar do
    /// cofre (um arquivo pela metade não abriria e impediria criar de novo).
    pub fn relocate(self, to: &Path) -> Result<Self> {
        let from = self.path.clone();
        let key = self.key.clone();
        let read_only = self.read_only;
        self.close()?;
        std::fs::rename(&from, to)?;
        for ext in ["-wal", "-shm", "-journal"] {
            let side = PathBuf::from(format!("{}{ext}", from.display()));
            if side.exists() {
                let _ = std::fs::remove_file(&side);
            }
        }
        Self::open_with_key(to, key, read_only)
    }

    /// Fecha a conexão (as chaves são zeradas no drop).
    pub fn close(self) -> Result<()> {
        self.conn.close().map_err(|(_, e)| Error::Db(e))
    }
}

pub(crate) fn apply_key(conn: &Connection, key: &DbKey) -> Result<()> {
    let v = key.pragma_value();
    conn.pragma_update(None, "key", v.as_str())?;
    Ok(())
}

fn key_is_valid(conn: &Connection) -> bool {
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| {
        r.get::<_, i64>(0)
    })
    .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::KDF_TEST;

    #[test]
    fn create_open_and_wrong_password() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("data").join("treasury.db");
        let db = Db::create(&p, "correta", KDF_TEST).unwrap();
        assert_eq!(db.schema_version().unwrap(), latest_schema_version());
        let salt = db.key().salt();
        db.conn()
            .execute("INSERT INTO setting VALUES ('k', '1', 'now')", [])
            .unwrap();
        db.close().unwrap();

        assert_eq!(
            vault::read_salt(&p).unwrap(),
            salt,
            "salt in the file header"
        );
        assert!(matches!(Db::open(&p, "errada"), Err(Error::WrongPassword)));
        let db = Db::open(&p, "correta").unwrap();
        let v: String = db
            .conn()
            .query_row("SELECT value FROM setting WHERE key = 'k'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(v, "1");
        let mode: String = db
            .conn()
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(mode, "delete");
    }

    #[test]
    fn file_is_not_plaintext() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.db");
        let db = Db::create(&p, "s", KDF_TEST).unwrap();
        db.conn()
            .execute(
                "INSERT INTO setting VALUES ('segredo', '\"MARCADOR-VISIVEL\"', 'now')",
                [],
            )
            .unwrap();
        db.close().unwrap();
        let bytes = std::fs::read(&p).unwrap();
        assert!(!bytes.windows(16).any(|w| w == b"SQLite format 3\0"));
        assert!(!bytes.windows(16).any(|w| w == b"MARCADOR-VISIVEL"));
    }

    #[test]
    fn change_password_keeps_data() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.db");
        let mut db = Db::create(&p, "antiga", KDF_TEST).unwrap();
        db.conn()
            .execute("INSERT INTO setting VALUES ('k', '2', 'now')", [])
            .unwrap();
        db.change_password("nova", KDF_TEST).unwrap();
        db.close().unwrap();
        assert!(matches!(Db::open(&p, "antiga"), Err(Error::WrongPassword)));
        let db = Db::open(&p, "nova").unwrap();
        let v: String = db
            .conn()
            .query_row("SELECT value FROM setting WHERE key = 'k'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(v, "2");
    }

    #[test]
    fn recovery_code_opens_database() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.db");
        let db = Db::create(&p, "senha", KDF_TEST).unwrap();
        let code = db.key().recovery_code();
        db.close().unwrap();
        let key = DbKey::from_recovery_code(&code, vault::read_salt(&p).unwrap()).unwrap();
        let db = Db::open_with_key(&p, key, false).unwrap();
        assert!(db.schema_version().unwrap() > 0);
    }

    #[test]
    fn read_only_open_rejects_writes() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.db");
        let db = Db::create(&p, "senha", KDF_TEST).unwrap();
        let key = db.key().clone();
        db.close().unwrap();
        let ro = Db::open_with_key(&p, key, true).unwrap();
        assert!(
            ro.conn()
                .execute("INSERT INTO setting VALUES ('a', '1', 'now')", [])
                .is_err()
        );
    }

    /// Vault written by 0.1.0 and vault written by 0.9.38, the last version with the old chain
    /// of migrations (test KDF, passwords below). Never regenerate: the point is to prove that
    /// the current code opens real old databases.
    const V010_FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/vault-v0.1.0.db"
    );
    const V010_PASSWORD: &str = "cofre-da-versao-0.1.0";
    const V0938_FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/vault-v0.9.38.db"
    );
    const V0938_PASSWORD: &str = "cofre-da-versao-0.9.38";

    fn copy(fixture: &str, dir: &tempfile::TempDir) -> PathBuf {
        let p = dir.path().join("treasury.db");
        std::fs::copy(fixture, &p).unwrap();
        p
    }

    /// V1 creates the same schema the old chain left: every object of a 0.9.38 vault
    /// exists in V1 with the same SQL, and vice versa.
    #[test]
    fn v1_is_the_old_chain_schema() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(&copy(V0938_FIXTURE, &dir), V0938_PASSWORD).unwrap();
        assert_eq!(objects(db.conn()).unwrap(), v1_objects().unwrap());
        // A new vault gets V1 and every migration published after it.
        let fresh = Db::create(&dir.path().join("novo.db"), "senha", KDF_TEST).unwrap();
        assert_eq!(objects(fresh.conn()).unwrap(), all_objects());
        assert_eq!(fresh.schema_version().unwrap(), latest_schema_version());
    }

    /// A 0.9.38 vault opens, swaps the V1-V164 history for V1 and keeps the data; opening
    /// again has nothing to migrate.
    #[test]
    fn a_0_9_38_vault_moves_to_the_v1_history() {
        let dir = tempfile::tempdir().unwrap();
        let p = copy(V0938_FIXTURE, &dir);
        let mut db = Db::open(&p, V0938_PASSWORD).unwrap();
        assert_eq!(db.schema_version().unwrap(), 164);
        assert!(db.needs_migration().unwrap());
        db.migrate().unwrap();
        // V1 replaces the old chain; the migrations after 1.0.0 run on top of it.
        let h = history(db.conn()).unwrap();
        assert_eq!(h[0], (1, "esquema".to_string()));
        assert_eq!(
            h.iter().map(|(v, _)| *v).collect::<Vec<_>>(),
            embedded_versions()
        );
        assert_eq!(objects(db.conn()).unwrap(), all_objects());
        let st = db.schema_status().unwrap();
        assert!(st.pending.is_empty() && st.unknown.is_empty(), "{st:?}");
        let (desc, amount): (String, i64) = db
            .conn()
            .query_row(
                "SELECT description, amount FROM txn WHERE description LIKE 'Mercado%'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((desc.as_str(), amount), ("Mercado da versão 0.9.38", -3990));
        drop(db);
        let db = Db::open(&p, V0938_PASSWORD).unwrap();
        assert!(!db.needs_migration().unwrap());
        assert_eq!(db.schema_version().unwrap(), latest_schema_version());
    }

    /// Schema objects after every embedded migration, in order, in an in-memory database.
    fn all_objects() -> Vec<(String, String, String)> {
        let conn = Connection::open_in_memory().unwrap();
        let mut ms = embedded::migrations::runner().get_migrations().to_vec();
        ms.sort_by_key(|m| m.version());
        for m in ms {
            conn.execute_batch(m.sql().unwrap_or_default()).unwrap();
        }
        objects(&conn).unwrap()
    }

    /// A 0.9.38 vault with a tampered schema (one index fewer) does not swap the history.
    #[test]
    fn a_0_9_38_vault_with_another_schema_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let p = copy(V0938_FIXTURE, &dir);
        let mut db = Db::open(&p, V0938_PASSWORD).unwrap();
        let index: String = db
            .conn()
            .query_row(
                "SELECT name FROM sqlite_master WHERE type = 'index' AND sql IS NOT NULL LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        db.conn()
            .execute_batch(&format!("DROP INDEX \"{index}\""))
            .unwrap();
        let e = db.migrate().unwrap_err();
        assert_eq!(e.code(), "migration", "{e}");
        assert_eq!(history(db.conn()).unwrap().len(), LEGACY_HISTORY.len());
    }

    /// A vault from before 0.9.38 (here, the 0.1.0 one) is refused with the guidance to open it in
    /// 0.9.38 first, without touching it.
    #[test]
    fn a_vault_older_than_0_9_38_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let p = copy(V010_FIXTURE, &dir);
        let e = Db::open(&p, V010_PASSWORD).unwrap_err();
        assert_eq!(e.code(), "migration", "{e}");
        assert!(e.to_string().contains("0.9.38"), "{e}");
    }

    #[test]
    fn refuses_a_vault_from_a_newer_version() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("treasury.db");
        let db = Db::create(&p, "senha", KDF_TEST).unwrap();
        db.conn()
            .execute(
                "INSERT INTO refinery_schema_history (version, name, applied_on, checksum) VALUES (9999, 'futuro', '2030-01-01', '0')",
                [],
            )
            .unwrap();
        drop(db);
        let e = Db::open(&p, "senha").unwrap_err();
        assert_eq!(e.code(), "schema_too_new");
        assert!(e.to_string().contains("restaure um backup"), "{e}");
    }

    #[test]
    fn released_migrations_are_frozen() {
        use sha2::{Digest, Sha256};
        let root = env!("CARGO_MANIFEST_DIR");
        let lock = std::fs::read_to_string(format!("{root}/migrations.lock")).unwrap();
        let mut released = std::collections::BTreeMap::new();
        for line in lock
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        {
            let mut parts = line.split_whitespace();
            let (v, file, hash) = (
                parts.next().unwrap().parse::<i32>().unwrap(),
                parts.next().unwrap(),
                parts.next().unwrap(),
            );
            let bytes = std::fs::read(format!("{root}/migrations/{file}"))
                .unwrap_or_else(|_| panic!("published migration {file} is missing"));
            assert_eq!(
                hex::encode(Sha256::digest(&bytes)),
                hash,
                "published migration {file} was changed: create a new migration instead"
            );
            assert!(
                released.insert(v, file.to_string()).is_none(),
                "version {v} repeated"
            );
        }
        let max_released = *released.keys().last().unwrap();
        for v in embedded_versions() {
            if !released.contains_key(&v) {
                assert!(
                    v > max_released,
                    "unpublished migration V{v} has a number lower than V{max_released}, the last published one"
                );
            }
        }
    }
}
