import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { mockCall, mockCallLog } from '@/api/mock';
import { mockChatStarts } from '@/api/mock/ai';
import type { Proposal } from '@/api/generated/Proposal';
import { renderApp } from '@/test/renderApp';
import { chartOption } from '@/test/echartsMock';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

function openAgente(search = '') {
  return renderApp('unlocked', {}, `#/agente${search}`);
}

async function pending(): Promise<Proposal[]> {
  return (await mockCall('ai.list_proposals', { status: 'pending' })) as Proposal[];
}

describe('Agente com o mock', () => {
  it('conversa em streaming: motor visível, raciocínio e ferramentas recolhidos, proposta com link', async () => {
    const user = userEvent.setup();
    openAgente();
    const indicator = await screen.findByRole('status', { name: 'Motor que recebe os dados' });
    await waitFor(() => expect(within(indicator).getByText('Claude (API)')).toBeTruthy());
    expect(within(indicator).getByText(/dados enviados à API do provider/)).toBeTruthy();

    await user.type(
      screen.getByLabelText('Mensagem para o agente'),
      'categorize o que falta{Enter}',
    );
    expect(await screen.findByText(/Propus a categoria Transporte/)).toBeTruthy();
    const thread = screen.getByRole('region', { name: 'Conversa com o agente' });
    // A pergunta aparece uma vez só (gravada na conversa, sem repetir a do streaming).
    const log = within(thread).getByRole('log', { name: 'Mensagens da conversa' });
    await waitFor(() => expect(within(log).getAllByText('categorize o que falta')).toHaveLength(1));
    // Raciocínio e ferramentas ficam recolhidos; a resposta final fica em destaque.
    // Sob carga, a resposta do streaming é trocada pela gravada: espera o bloco final.
    const activity = await waitFor(() => {
      const last = within(thread).getAllByRole('group', { name: 'Atividade do agente' }).at(-1)!;
      within(last).getByRole('button', { name: /^Raciocínio · 6 s$/ });
      within(last).getByRole('button', { name: 'Consultou 2 ferramentas' });
      return last;
    });
    const reasoning = within(activity).getByRole('button', { name: /^Raciocínio · 6 s$/ });
    expect(reasoning.getAttribute('aria-expanded')).toBe('false');
    expect(within(thread).queryByText(/Vou olhar os lançamentos/)).toBeNull();
    const tools = within(activity).getByRole('button', { name: 'Consultou 2 ferramentas' });
    expect(within(thread).queryByText('query_transactions')).toBeNull();
    await user.click(tools);
    const list = await within(thread).findByRole('list', { name: 'Ferramentas usadas' });
    expect(within(list).getByText('query_transactions')).toBeTruthy();
    expect(within(list).getByText('1 lançamento')).toBeTruthy();
    await user.click(reasoning);
    expect(
      within(thread).getByText(/Vou olhar os lançamentos sem categoria deste mês\./),
    ).toBeTruthy();
    expect(within(thread).getByText(/começo pelos lançamentos sem categoria/)).toBeTruthy();
    const link = within(thread).getAllByRole('link', { name: 'Revisar na fila' })[0]!;
    expect(link.getAttribute('href')).toMatch(/aba=aprovacao&proposta=mock-ai-proposal-/);
    expect((await pending()).length).toBe(4);
    // A conversa nova entra na lista (recolhida por padrão: abre pelo botão do chat).
    await user.click(within(thread).getByRole('button', { name: 'Mostrar as conversas' }));
    expect(await screen.findByRole('button', { name: /^categorize o que falta/ })).toBeTruthy();
    expect(mockCallLog().filter((c) => c === 'ai.chat_poll').length).toBeGreaterThan(3);
  });

  it('motor escolhido que depois foi apagado: a conversa volta ao automático', async () => {
    const user = userEvent.setup();
    openAgente();
    await screen.findByRole('status', { name: 'Motor que recebe os dados' });
    await user.click(screen.getByRole('combobox', { name: 'Motor' }));
    await user.click(await screen.findByRole('option', { name: /Ollama/ }));
    // O motor é excluído em Configurar.
    await user.click(screen.getByRole('tab', { name: /Configurar/ }));
    const row = await screen.findByRole('listitem', { name: /Motor Ollama/ });
    await user.click(within(row).getByRole('button', { name: 'Excluir' }));
    const confirm = await screen.findByRole('dialog', { name: /Excluir Ollama/ });
    await user.click(within(confirm).getByRole('button', { name: 'Excluir' }));
    await waitFor(() =>
      expect(screen.queryByRole('listitem', { name: /Motor Ollama/ })).toBeNull(),
    );
    await user.click(screen.getByRole('tab', { name: /Conversa/ }));
    await waitFor(() =>
      expect(screen.getByRole('combobox', { name: 'Motor' }).textContent).toContain('Automático'),
    );
    await user.type(screen.getByLabelText('Mensagem para o agente'), 'oi{Enter}');
    await waitFor(() => expect(mockChatStarts().length).toBeGreaterThan(0));
    expect(mockChatStarts().at(-1)!.engine_id).toBeUndefined();
  });

  it('mostra o uso e o custo da resposta enquanto o histórico está desligado', async () => {
    const user = userEvent.setup();
    openAgente();
    await mockCall('ai.settings.set', { keep_history: false });
    await user.type(await screen.findByLabelText('Mensagem para o agente'), 'oi{Enter}');
    const usage = await screen.findByLabelText('Uso desta resposta');
    expect(usage.textContent).toMatch(/USD 0,0177/);
    expect(screen.getByText('oi')).toBeTruthy();
  });

  it('pede confirmação de custo acima do aviso e envia com confirm_cost', async () => {
    const user = userEvent.setup();
    openAgente();
    await mockCall('ai.limits.set', { provider: 'anthropic', confirm_above_micros: 0 });
    await user.type(
      await screen.findByLabelText('Mensagem para o agente'),
      'relatório completo{Enter}',
    );
    const dialog = await screen.findByRole('dialog', { name: 'Confirmar custo' });
    expect(within(dialog).getByText(/USD 0,0309/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Enviar mesmo assim' }));
    expect(await screen.findByText(/Propus a categoria Transporte/)).toBeTruthy();
  });

  it('sem motor configurado: aviso com link para configurar e erro ao enviar', async () => {
    const user = userEvent.setup();
    openAgente();
    for (const id of [
      'mock-ai-engine-claude',
      'mock-ai-engine-ollama',
      'mock-ai-engine-claude-code',
    ]) {
      await mockCall('ai.engines.delete', { id });
    }
    await user.click(await screen.findByRole('tab', { name: /Alertas/ }));
    await user.click(screen.getByRole('tab', { name: /Conversa/ }));
    expect(await screen.findByText('Nenhum motor de IA configurado')).toBeTruthy();
    const link = screen.getAllByRole('link', { name: 'Configurar motores de IA' })[0]!;
    expect(link.getAttribute('href')).toContain('aba=configurar');
    await user.type(screen.getByLabelText('Mensagem para o agente'), 'oi{Enter}');
    expect(await screen.findByText(/cadastre um em Ajustes › IA e agente/)).toBeTruthy();
    // O atalho gratuito aparece no aviso.
    expect(screen.getAllByText(/o Groq tem plano gratuito/).length).toBeGreaterThan(0);
  });

  it('Groq: atalho do plano gratuito abre o cadastro com o guia da chave', async () => {
    const user = userEvent.setup();
    openAgente('?aba=configurar');
    await screen.findByRole('listitem', { name: 'Motor Claude (API)' });
    await user.click(screen.getByRole('button', { name: 'Usar o Groq (plano gratuito)' }));
    const dialog = await screen.findByRole('dialog', { name: 'Novo motor de IA' });
    const guide = within(dialog).getByRole('region', {
      name: 'Como pegar a chave gratuita do Groq',
    });
    expect(within(guide).getByText(/console.groq.com e crie uma conta/)).toBeTruthy();
    expect(within(guide).getByText(/até 30 dias/)).toBeTruthy();
    expect(
      within(guide).getByRole('link', { name: 'Abrir o console do Groq' }).getAttribute('href'),
    ).toBe('https://console.groq.com/keys');
    expect(within(dialog).getByLabelText('Modelo')).toHaveProperty(
      'value',
      'llama-3.3-70b-versatile',
    );
    await user.type(within(dialog).getByLabelText('API key'), 'gsk_teste_1234');
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar' }));
    expect(await screen.findByRole('listitem', { name: 'Motor Groq' })).toBeTruthy();
    // Com o Groq cadastrado, o atalho some.
    await waitFor(() =>
      expect(screen.queryByRole('button', { name: 'Usar o Groq (plano gratuito)' })).toBeNull(),
    );
  });

  it('fila: texto amigável, diff exato nos detalhes, lote só para reversíveis', async () => {
    const user = userEvent.setup();
    openAgente('?aba=aprovacao');
    const cat = await screen.findByRole('article', { name: /Categorizar 2 compras no cartão/ });
    // O que muda, com nomes no lugar dos ids (o lançamento vem do livro-caixa pelo id).
    const summary = await within(cat).findByRole('list', { name: 'O que muda' });
    expect(summary.textContent).toContain(
      'Altera a categoria de 2 lançamentos: Sem categoria → Alimentação › Mercado',
    );
    const row = await within(cat).findByRole('region', {
      name: 'Alteração · Lançamento: COMPRA *MERCPAGO 4421',
    });
    expect(within(row).getByText('Categoria')).toBeTruthy();
    expect(within(row).getByText('Alimentação › Mercado')).toBeTruthy();
    expect(cat.textContent).not.toMatch(/cat-alimentacao|txn-\d|category_id/);
    // Diff exato, com ids e valores brutos, a um clique.
    await user.click(within(cat).getByRole('button', { name: 'Ver detalhes técnicos' }));
    const diff = within(cat).getAllByRole('table', { name: /Alteração · Lançamento txn-/ })[0]!;
    expect(within(diff).getByText('category_id')).toBeTruthy();
    expect(within(diff).getByText('"cat-alimentacao-mercado"')).toBeTruthy();
    expect(within(diff).getByText('∅')).toBeTruthy();

    // Regra e operação: condições, ações, ativo e conta pelo nome.
    const rule = screen.getByRole('article', { name: /Regra: “PAO DE ACUCAR”/ });
    expect(within(rule).getByText('Cria 1 regra: “Pão de Açúcar → Mercado”')).toBeTruthy();
    expect(within(rule).getByText('Descrição contém “PAO DE ACUCAR”')).toBeTruthy();
    expect(within(rule).getByText('categoria Alimentação › Mercado')).toBeTruthy();
    const trade = screen.getByRole('article', { name: /Compra de 100 PETR4/ });
    expect(
      await within(trade).findByRole('region', { name: 'Criação · Operação: Compra de PETR4' }),
    ).toBeTruthy();
    expect(within(trade).getByText('XP Investimentos')).toBeTruthy();
    expect(screen.getByText(/Modo autônomo desligado/)).toBeTruthy();

    expect(within(trade).queryByRole('checkbox')).toBeNull();
    expect(within(trade).getByText('Sensível · item a item')).toBeTruthy();

    await user.click(screen.getByRole('button', { name: 'Selecionar reversíveis' }));
    await user.click(screen.getByRole('button', { name: 'Aprovar selecionadas (2)' }));
    expect(await screen.findByText('2 propostas aprovadas')).toBeTruthy();
    await waitFor(async () =>
      expect((await pending()).map((p) => p.id)).toEqual(['mock-ai-proposal-trade']),
    );

    await user.click(within(trade).getByRole('button', { name: 'Aprovar este item' }));
    expect(await screen.findByText('Nenhuma proposta esperando aprovação.')).toBeTruthy();
    expect(screen.getAllByRole('button', { name: 'Desfazer' }).length).toBeGreaterThan(0);
  });

  it('alertas: contador na barra lateral, filtros e resolver', async () => {
    const user = userEvent.setup();
    openAgente('?aba=alertas');
    const nav = await screen.findByRole('link', { name: /Agente/ });
    await waitFor(() =>
      expect(within(nav).getByText(/3 propostas pendentes, 2 alertas novos/)).toBeTruthy(),
    );
    const list = await screen.findByRole('list', { name: 'Alertas' });
    expect(within(list).getAllByRole('listitem')).toHaveLength(3);
    const budget = within(list).getByRole('listitem', { name: /Mercado passou de 90%/ });
    // Rodapé em itálico (sem os sublinhados) e as ferramentas só em "Avançado".
    const note = within(budget).getByText(/Análise baseada nos seus dados/);
    expect(note.tagName).toBe('EM');
    expect(note.textContent?.includes('_')).toBe(false);
    expect(
      within(budget)
        .queryByText(/get_budget_status/)
        ?.closest('details')?.open,
    ).toBe(false);
    // A origem abre na tela dela; o rodapé mostra o nome do tipo, não o id.
    expect(within(budget).getByRole('link', { name: 'Abrir orçamento' }).getAttribute('href')).toBe(
      '#/orcamento',
    );
    expect(within(budget).getByText(/^Orçamento estourado · /)).toBeTruthy();
    expect(within(budget).queryByText(/budget_over/)).toBeNull();
    const due = within(list).getByRole('listitem', { name: /Fatura do Nubank/ });
    expect(within(due).queryByRole('link')).toBeNull();
    await user.click(within(budget).getByText('Avançado'));
    expect(
      within(budget)
        .getByText(/get_budget_status/)
        .closest('details')?.open,
    ).toBe(true);
    await user.click(within(budget).getByRole('button', { name: 'Resolver' }));
    await waitFor(() => expect(within(list).queryByText(/Mercado passou de 90%/)).toBeNull());
    await waitFor(() => expect(within(nav).getByText(/1 alerta novo/)).toBeTruthy());
    expect(
      screen.getByRole('listitem', { name: 'Alerta: Fatura do Nubank vence em 2 dias' }),
    ).toBeTruthy();
    expect(screen.getByText('Rotinas do gestor')).toBeTruthy();
  });

  it('histórico de ferramentas e memória do gestor', async () => {
    const user = userEvent.setup();
    openAgente('?aba=ferramentas');
    const grid = await screen.findByRole('grid', { name: 'Histórico de ferramentas' });
    expect(within(grid).getByText('generate_darf')).toBeTruthy();
    await user.click(screen.getByLabelText('Só chamadas com erro'));
    await waitFor(() =>
      expect(
        within(screen.getByRole('grid', { name: 'Histórico de ferramentas' })).queryByText(
          'spending_summary',
        ),
      ).toBeNull(),
    );

    await user.click(screen.getByRole('tab', { name: /Memória/ }));
    expect(await screen.findByText('O aluguel vence dia 5')).toBeTruthy();
    const row = screen.getByText('O aluguel vence dia 5').closest('li')!;
    await user.click(within(row).getByRole('button', { name: 'Esquecer' }));
    await user.click(
      within(await screen.findByRole('dialog')).getByRole('button', { name: 'Esquecer' }),
    );
    await waitFor(() => expect(screen.queryByText('O aluguel vence dia 5')).toBeNull());
  });

  it('memória de procedimentos: origem, uso, busca, edição e exclusão', async () => {
    const user = userEvent.setup();
    openAgente('?aba=memoria');
    const list = await screen.findByRole('list', { name: 'Procedimentos' });
    const itau = within(list).getByRole('listitem', {
      name: 'Procedimento: Extrato OFX da conta corrente Itaú',
    });
    expect(within(itau).getByText('Extrato')).toBeTruthy();
    expect(within(itau).getByText('funcionou 3×')).toBeTruthy();
    expect(within(itau).getByText('www.itau.com.br')).toBeTruthy();
    expect(within(itau).getByText(/Gravado pelo agente \(CLI de teste\) em/)).toBeTruthy();
    const conv = within(itau).getByRole('link', { name: 'Conversa: Restaurantes em setembro' });
    expect(conv.getAttribute('href')).toContain('conversa=mock-ai-conv-1');
    const b3 = within(list).getByRole('listitem', { name: /Área do Investidor B3/ });
    expect(within(b3).getByText('falhou 1×')).toBeTruthy();
    expect(within(b3).getByText(/Última observação: O botão de baixar/)).toBeTruthy();

    // Busca sem acento e filtro por assunto.
    await user.type(screen.getByLabelText('Buscar'), 'itau');
    await waitFor(() =>
      expect(
        within(screen.getByRole('list', { name: 'Procedimentos' })).getAllByRole('listitem'),
      ).toHaveLength(1),
    );
    await user.clear(screen.getByLabelText('Buscar'));
    await waitFor(() =>
      expect(
        within(screen.getByRole('list', { name: 'Procedimentos' })).getAllByRole('listitem'),
      ).toHaveLength(2),
    );

    // Editar os passos.
    const row = screen.getByRole('listitem', {
      name: 'Procedimento: Extrato OFX da conta corrente Itaú',
    });
    await user.click(within(row).getByRole('button', { name: 'Editar' }));
    const dialog = await screen.findByRole('dialog', { name: 'Editar procedimento' });
    const steps = within(dialog).getByLabelText('Passos');
    await user.clear(steps);
    await user.type(steps, 'Menu Extratos › Exportar OFX');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar' }));
    expect(await screen.findByText('Menu Extratos › Exportar OFX')).toBeTruthy();

    // Adicionar e apagar.
    await user.click(screen.getByRole('button', { name: 'Adicionar' }));
    const add = await screen.findByRole('dialog', { name: 'Novo procedimento' });
    await user.type(within(add).getByLabelText('Título'), 'Fatura do Nubank');
    await user.type(within(add).getByLabelText('Passos'), 'App › Cartão › Faturas › PDF');
    await user.click(within(add).getByRole('button', { name: 'Salvar' }));
    const nubank = await screen.findByRole('listitem', { name: 'Procedimento: Fatura do Nubank' });
    expect(within(nubank).getByText(/Gravado por você/)).toBeTruthy();
    await user.click(within(nubank).getByRole('button', { name: 'Apagar' }));
    await user.click(
      within(await screen.findByRole('dialog', { name: 'Apagar este procedimento' })).getByRole(
        'button',
        { name: 'Apagar' },
      ),
    );
    await waitFor(() =>
      expect(screen.queryByRole('listitem', { name: 'Procedimento: Fatura do Nubank' })).toBeNull(),
    );
  });

  it('configurar: motor local, rota com fallback, limite e gráfico de uso num só eixo', async () => {
    const user = userEvent.setup();
    openAgente('?aba=configurar');
    const claude = await screen.findByRole('listitem', { name: 'Motor Claude (API)' });
    expect(within(claude).getByText(/cadastrada · …a1b2/)).toBeTruthy();

    await user.click(screen.getByRole('button', { name: 'Adicionar motor' }));
    const dialog = await screen.findByRole('dialog', { name: 'Novo motor de IA' });
    await user.click(within(dialog).getAllByRole('combobox')[0]!);
    await user.click(await screen.findByRole('option', { name: /LM Studio/ }));
    await user.type(within(dialog).getByLabelText('Nome'), 'Estúdio');
    await user.type(within(dialog).getByLabelText('Modelo'), 'qwen3');
    await user.type(
      within(dialog).getByLabelText('URL do servidor local'),
      'http://127.0.0.1:1234/v1',
    );
    expect(within(dialog).queryByLabelText('API key')).toBeNull();
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar' }));
    const local = await screen.findByRole('listitem', { name: 'Motor Estúdio' });
    expect(within(local).getByText('pronto')).toBeTruthy();
    await user.click(within(local).getByRole('button', { name: 'Testar' }));
    expect(await within(local).findByText(/respondeu em 640 ms/)).toBeTruthy();

    const pdf = screen.getByRole('listitem', { name: 'Rota: Leitura de PDF' });
    await user.click(within(pdf).getByRole('combobox', { name: /Adicionar motor à rota/ }));
    await user.click(await screen.findByRole('option', { name: /Estúdio/ }));
    await user.click(within(pdf).getByRole('combobox', { name: /Adicionar motor à rota/ }));
    await user.click(await screen.findByRole('option', { name: /Ollama local/ }));
    await waitFor(async () => {
      const routes = (await mockCall('ai.routes.list', {})) as {
        task: string;
        engine_ids: string[];
      }[];
      expect(routes.find((r) => r.task === 'pdf')!.engine_ids).toHaveLength(2);
    });
    await user.click(await within(pdf).findByRole('button', { name: 'Subir Ollama local' }));
    await waitFor(async () => {
      const routes = (await mockCall('ai.routes.list', {})) as {
        task: string;
        engine_ids: string[];
      }[];
      expect(routes.find((r) => r.task === 'pdf')!.engine_ids[0]).toBe('mock-ai-engine-ollama');
    });

    // Limites, uso e preços ficam na aba Custos.
    await user.click(screen.getByRole('tab', { name: 'Custos' }));
    expect(window.location.hash).toContain('ia=custos');
    const limit = await screen.findByRole('listitem', { name: 'Limites: Anthropic (Claude)' });
    const daily = within(limit).getByLabelText('Por dia (USD)');
    await user.clear(daily);
    await user.type(daily, '2,5');
    await user.click(within(limit).getByRole('button', { name: 'Salvar' }));
    await waitFor(async () => {
      const limits = (await mockCall('ai.limits.list', {})) as { daily_micros: number }[];
      expect(limits[0]!.daily_micros).toBe(2_500_000);
    });

    const option = chartOption(/Custo estimado de IA por mês/) as {
      series: unknown[];
      yAxis: unknown;
    };
    expect(option.series).toHaveLength(1);
    expect(Array.isArray(option.yAxis)).toBe(false);
    expect(screen.getByRole('grid', { name: 'Preços por modelo' })).toBeTruthy();
  });
});

describe('Agente: chat em destaque, conversas recolhidas e exportar', () => {
  it('esconde o agente companheiro só na tela do Agente', async () => {
    openAgente();
    await screen.findByRole('region', { name: 'Conversa com o agente' });
    expect(screen.queryByRole('button', { name: 'Abrir o agente' })).toBeNull();
    renderApp('unlocked', {}, '#/lancamentos');
    expect(await screen.findByRole('button', { name: 'Abrir o agente' })).toBeTruthy();
  });

  it('conversas recolhidas por padrão; abrir e recolher fica lembrado', async () => {
    const user = userEvent.setup();
    openAgente();
    const thread = await screen.findByRole('region', { name: 'Conversa com o agente' });
    expect(screen.queryByRole('heading', { name: /Conversas/ })).toBeNull();
    const open = within(thread).getByRole('button', { name: 'Mostrar as conversas' });
    expect(open.getAttribute('aria-expanded')).toBe('false');
    await user.click(open);
    expect(await screen.findByRole('button', { name: /^Restaurantes em setembro/ })).toBeTruthy();
    const stored = JSON.parse(window.localStorage.getItem('treasury.ui') ?? '{}') as {
      state?: { agentSessionsOpen?: boolean };
    };
    expect(stored.state?.agentSessionsOpen).toBe(true);
    await user.click(within(thread).getByRole('button', { name: 'Recolher as conversas' }));
    await waitFor(() =>
      expect(screen.queryByRole('button', { name: /^Restaurantes em setembro/ })).toBeNull(),
    );
  });

  it('exporta a conversa aberta em Markdown e PDF', async () => {
    const user = userEvent.setup();
    const list = (await mockCall('ai.conversations.list', {})) as { id: string }[];
    openAgente(`?conversa=${list[0]!.id}`);
    const thread = await screen.findByRole('region', { name: 'Conversa com o agente' });
    const group = await within(thread).findByRole('group', { name: 'Exportar' });
    await user.click(within(group).getByRole('button', { name: 'PDF' }));
    expect((await screen.findAllByText('Conversa exportada')).length).toBeGreaterThan(0);
    // O aviso aparece no cartão e, às vezes, também na região de anúncios (leitor de tela).
    expect(screen.getAllByText(/Restaurantes-em-setembro\.pdf/).length).toBeGreaterThan(0);
    await user.click(within(group).getByRole('button', { name: 'Markdown' }));
    await waitFor(() =>
      expect(mockCallLog().filter((c) => c === 'ai.conversations.export')).toHaveLength(2),
    );
  });
});
