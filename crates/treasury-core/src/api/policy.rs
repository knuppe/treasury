//! Política de permissões para escritas da IA e de clientes MCP (docs/ai-agent.md
//! §Permissões e controle). Toda escrita de um cliente MCP vira **proposta** com o diff
//! exato e só é aplicada depois da aprovação do usuário; o modo autônomo (desligado por
//! padrão) aplica direto apenas escritas reversíveis.

/// Nível de uma escrita feita pela IA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Aprovação em lote (categorizar, criar regra, lançamento previsto, orçamento…).
    Reversible,
    /// Aprovação item a item (operações de investimento, importação, apagar…).
    Sensitive,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Reversible => "reversible",
            Level::Sensitive => "sensitive",
        }
    }
}

/// Comandos de escrita que a IA pode propor e o nível de cada um. Tudo o que não está aqui
/// (cofre, backups, ajustes, chaves, exportação, DARF, câmbio manual…) é proibido para a IA.
pub const AI_WRITES: &[(&str, Level)] = &[
    ("transactions.create", Level::Reversible),
    ("transactions.update", Level::Reversible),
    ("transactions.set_category", Level::Reversible),
    ("transactions.set_member", Level::Reversible),
    ("transactions.edit_tags", Level::Reversible),
    ("transactions.split", Level::Reversible),
    ("transactions.create_transfer", Level::Reversible),
    ("transactions.link_transfers", Level::Reversible),
    ("transactions.unlink_transfer", Level::Reversible),
    ("transactions.link_operation", Level::Reversible),
    ("transactions.unlink_operation", Level::Reversible),
    ("fx.exchange", Level::Reversible),
    ("transactions.delete", Level::Sensitive),
    ("transactions.create_tag", Level::Reversible),
    ("categories.create", Level::Reversible),
    ("categories.update", Level::Reversible),
    ("categories.add_defaults", Level::Reversible),
    // Grupo de gasto de várias categorias de uma vez (essenciais, dívidas, futuro, não essencial).
    ("categories.set_group", Level::Reversible),
    // Cadastro (companheiro): instituições, contas e cartões. Alterar uma conta existente
    // (saldo inicial, arquivar, dados do cartão) muda o histórico: item a item.
    ("household.create_institution", Level::Reversible),
    ("household.update_institution", Level::Reversible),
    ("accounts.create", Level::Reversible),
    ("accounts.update", Level::Sensitive),
    ("accounts.sync_balance", Level::Sensitive),
    ("accounts.resolve_sync", Level::Reversible),
    ("accounts.capital_share_set", Level::Reversible),
    ("accounts.capital_share_add_balance", Level::Reversible),
    ("rules.create", Level::Reversible),
    ("rules.update", Level::Reversible),
    ("rules.apply", Level::Reversible),
    ("recurrences.create", Level::Reversible),
    ("recurrences.update", Level::Reversible),
    ("budget.set_lines", Level::Reversible),
    ("budget.copy", Level::Reversible),
    ("goals.create", Level::Reversible),
    ("projects.create", Level::Reversible),
    ("projects.update", Level::Reversible),
    ("projects.link", Level::Reversible),
    ("projects.add_item", Level::Reversible),
    // Estimativa de impostos: grupo da categoria e informes de rendimentos.
    ("tax_burden.set_category", Level::Reversible),
    ("tax_burden.save_income_report", Level::Reversible),
    ("projects.update_item", Level::Reversible),
    ("goals.update", Level::Reversible),
    ("goals.link_source", Level::Reversible),
    ("goals.unlink_source", Level::Reversible),
    ("assets.create", Level::Reversible),
    // Mudar classe, moeda ou renda fixa de um ativo com posição muda a carteira.
    ("assets.update", Level::Sensitive),
    // Dados do ativo com fonte (setor, ISIN, CNPJ…): nunca no modo autônomo; só valores que
    // estão numa leitura das fontes (ver `assets::enrich::apply`).
    ("assets.enrich_apply", Level::Sensitive),
    // Estudo com IA pedido no chat: gasta o motor de estudo, aprovação item a item.
    ("assets.study_request", Level::Sensitive),
    ("portfolio.watch", Level::Reversible),
    ("portfolio.create_trade", Level::Sensitive),
    ("portfolio.create_income", Level::Sensitive),
    // Proventos pelo ticker: uma proposta só com o que foi deduzido de cada linha.
    ("portfolio.quick_income", Level::Sensitive),
    ("portfolio.create_action", Level::Sensitive),
    // A prévia não grava dados (fica na memória da sessão); o commit vira proposta.
    ("import.preview", Level::Sensitive),
    ("import.commit", Level::Sensitive),
    // Lançamentos lidos de uma página pelo agente (deduplicados como um extrato).
    ("import.rows", Level::Sensitive),
    ("import.file", Level::Sensitive),
    // Investimentos em lote (conta, ativos, operações, posição numa data, proventos): uma
    // proposta só com o diff de tudo.
    ("portfolio.bulk", Level::Sensitive),
    // Importação da Área do Investidor B3: mesma política da importação de extratos.
    ("b3.preview", Level::Sensitive),
    ("b3.commit", Level::Sensitive),
    // Notas de corretagem em PDF: a prévia só lê; o commit grava operações.
    ("notes.commit", Level::Sensitive),
    ("ai.remember", Level::Reversible),
    // Telas salvas do screener (análise de ativos).
    ("research.screens.create", Level::Reversible),
    // Observar um papel do Mercado (cria o ativo, se faltar, e põe na watchlist).
    ("research.observe", Level::Reversible),
    // Etiquetas de ativos e limites de concentração (V161).
    ("assets.labels_set", Level::Reversible),
    ("assets.label_update", Level::Reversible),
    ("portfolio.set_concentration_limit", Level::Reversible),
    ("research.screens.update", Level::Reversible),
    ("research.screens.delete", Level::Sensitive),
    // Patrimônio físico: cadastro, benfeitorias, avaliações e financiamento ligado.
    ("physical.iptu_save", Level::Reversible),
    ("physical.create", Level::Reversible),
    ("physical.update", Level::Reversible),
    ("physical.add_improvement", Level::Reversible),
    ("physical.update_improvement", Level::Reversible),
    ("physical.add_valuation", Level::Reversible),
    ("physical.link_loan", Level::Reversible),
    ("physical.unlink_loan", Level::Reversible),
    ("physical.sell", Level::Sensitive),
    ("physical.delete", Level::Sensitive),
    ("physical.delete_improvement", Level::Sensitive),
    ("physical.delete_valuation", Level::Sensitive),
    // CRUD completo (agentes): todo apagar e toda alteração que muda o histórico ou a
    // carteira é sensível (item a item, nunca no modo autônomo); cadastros simples são
    // reversíveis.
    ("crud.delete", Level::Sensitive),
    ("accounts.delete", Level::Sensitive),
    ("assets.delete", Level::Sensitive),
    ("categories.delete", Level::Sensitive),
    ("categories.merge", Level::Sensitive),
    ("rules.delete", Level::Sensitive),
    ("recurrences.delete", Level::Sensitive),
    ("goals.delete", Level::Sensitive),
    ("projects.delete", Level::Sensitive),
    // Apaga os itens lançados à mão (os lançamentos só saem do projeto).
    ("projects.remove_entries", Level::Sensitive),
    ("tax_burden.delete_income_report", Level::Sensitive),
    ("portfolio.update_trade", Level::Sensitive),
    ("portfolio.delete_trade", Level::Sensitive),
    ("portfolio.delete_income", Level::Sensitive),
    ("portfolio.delete_action", Level::Sensitive),
    ("portfolio.delete_trades", Level::Sensitive),
    ("portfolio.delete_incomes", Level::Sensitive),
    ("portfolio.delete_actions", Level::Sensitive),
    ("household.delete_institution", Level::Sensitive),
    ("household.archive_member", Level::Sensitive),
    ("household.create_member", Level::Reversible),
    ("household.update_member", Level::Reversible),
    ("income.create_source", Level::Reversible),
    ("income.update_source", Level::Reversible),
    ("income.delete_source", Level::Sensitive),
    ("income.payslip_import", Level::Sensitive),
    ("income.delete_payslip", Level::Sensitive),
    ("receipts.import", Level::Reversible),
    ("transactions.delete_tag", Level::Sensitive),
    ("transactions.update_tag", Level::Reversible),
    ("transactions.set_status", Level::Reversible),
    ("debt.create", Level::Reversible),
    ("pension.set_plan", Level::Reversible),
    ("debt.update", Level::Sensitive),
    ("debt.import_statement", Level::Sensitive),
    // Extrato de previdência: a prévia só lê; a importação grava cotas, cotações e o plano.
    ("pension.statement_import", Level::Sensitive),
    ("pension.payments_import", Level::Sensitive),
    // FGTS (docs/brazil/fgts.md): a importação e os saques mexem no saldo, item a item.
    ("fgts.statement_import", Level::Sensitive),
    ("fgts.record_withdrawal", Level::Sensitive),
    ("fgts.use_for_housing", Level::Sensitive),
    ("fgts.set_modality", Level::Sensitive),
    ("fgts.update_account", Level::Reversible),
    ("debt.record_payment", Level::Sensitive),
    ("debt.delete", Level::Sensitive),
    ("debt.mark_paid", Level::Sensitive),
    ("debt.unmark_paid", Level::Sensitive),
    ("debt.mark_paid_until", Level::Sensitive),
    ("debt.add_extra_payment", Level::Sensitive),
    ("debt.anticipate", Level::Sensitive),
    ("debt.link_payment", Level::Reversible),
    ("cards.pay_statement", Level::Sensitive),
    ("cards.set_informed_total", Level::Reversible),
    ("cards.move_to_statement", Level::Reversible),
    ("cards.create_installments", Level::Reversible),
    ("installments.ignore", Level::Reversible),
    ("installments.unignore", Level::Reversible),
    ("cards.delete_installment_plan", Level::Sensitive),
    ("cards.create_plastic", Level::Reversible),
    ("cards.update_plastic", Level::Reversible),
    ("cards.delete_plastic", Level::Reversible),
    ("cards.update_statement", Level::Reversible),
    ("cards.add_limit_change", Level::Reversible),
    ("cards.delete_limit_change", Level::Reversible),
    ("cards.set_annual_fee", Level::Reversible),
    ("cards.end_annual_fee", Level::Reversible),
    ("cards.set_reward_program", Level::Reversible),
    ("cards.end_reward_program", Level::Reversible),
    ("cards.loyalty_create", Level::Reversible),
    ("cards.loyalty_update", Level::Reversible),
    ("cards.loyalty_delete", Level::Sensitive),
    ("cards.loyalty_add_entry", Level::Reversible),
    ("cards.loyalty_transfer", Level::Reversible),
    ("cards.loyalty_delete_entry", Level::Reversible),
    // Feedback ao desenvolvedor: o agente apaga ou descarta só por proposta (o resto do
    // quadro grava direto, ver `DEV_FEEDBACK_WRITES`).
    ("devfeedback.delete", Level::Sensitive),
    ("devfeedback.discard", Level::Sensitive),
];

pub fn level(cmd: &str) -> Option<Level> {
    AI_WRITES.iter().find(|(c, _)| *c == cmd).map(|(_, l)| *l)
}

/// Escritas na memória de procedimentos do próprio agente: aplicadas direto, sem proposta,
/// mas com conjunto de alterações (histórico e desfazer) e visíveis e editáveis pelo
/// usuário em Agente › Memória. As ferramentas só existem para o agente do app (grupo
/// `AgentMemory` do MCP); clientes externos só leem.
pub const AGENT_MEMORY_WRITES: &[&str] = &["ai.procedures.save", "ai.procedures.outcome"];

pub fn is_agent_memory(cmd: &str) -> bool {
    AGENT_MEMORY_WRITES.contains(&cmd)
}

/// Leituras do contexto do próprio agente (o histórico e os anexos da conversa, a memória
/// do gestor por assunto): só para o agente do app (grupo `AgentMemory` do MCP), sem
/// escrever nada.
pub const AGENT_CONTEXT_READS: &[&str] =
    &["ai.chat_history", "ai.attachment_read", "ai.memory_recall"];

pub fn is_agent_context(cmd: &str) -> bool {
    AGENT_CONTEXT_READS.contains(&cmd)
}

/// Feedback do agente ao desenvolvedor (`devfeedback`): adicionar, reforçar, resolver com o
/// motivo, mover no quadro, reordenar, editar e vincular, direto e sem proposta (informação
/// técnica, não dado da família), com histórico e desfazer, e só com o modo desenvolvedor
/// ligado (`developer_tools`; o próprio comando confere). Apagar e descartar viram proposta
/// (`AI_WRITES`).
pub const DEV_FEEDBACK_WRITES: &[&str] = &[
    "devfeedback.add",
    "devfeedback.bump",
    "devfeedback.resolve",
    "devfeedback.set_status",
    "devfeedback.set_priority",
    "devfeedback.update",
    "devfeedback.link",
    "devfeedback.unlink",
];

pub fn is_dev_feedback(cmd: &str) -> bool {
    DEV_FEEDBACK_WRITES.contains(&cmd)
}

/// O agente retira uma proposta que ele mesmo fez e ainda está pendente (um teste, um
/// engano): direto, sem outra proposta, e só a do próprio cliente (`ai.withdraw_proposal`
/// confere).
pub const OWN_PROPOSAL_WRITES: &[&str] = &["ai.withdraw_proposal"];

pub fn is_own_proposal(cmd: &str) -> bool {
    OWN_PROPOSAL_WRITES.contains(&cmd)
}

/// Ajuste: aplicar sem perguntar as escritas reversíveis da IA (desligado por padrão).
pub const AUTONOMOUS_SETTING: &str = "ai.autonomous";
