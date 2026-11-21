import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import type { DevFeedback } from '@/api/generated/DevFeedback';
import { mockCall } from '@/api/mock';
import { titleSimilarity } from '@/api/mock/devfeedback';
import { renderApp } from '@/test/renderApp';

import { allMarkdown, feedbackMarkdown, sections, taskMarkdown } from './format';

async function developerMode(queryClient: ReturnType<typeof renderApp>['queryClient']) {
  await act(async () => {
    await mockCall('settings.set', { developer_tools: true });
    await queryClient.invalidateQueries();
  });
}

function rows() {
  const list = screen.getByRole('list', { name: 'Feedback' });
  return within(list)
    .getAllByRole('listitem')
    .filter((li) => li.getAttribute('aria-label')?.startsWith('Feedback: '));
}

describe('Desenvolvedor', () => {
  it('fica fora do menu sem o modo desenvolvedor e explica como ligar', async () => {
    renderApp('unlocked', {}, '#/desenvolvedor');
    const nav = await screen.findByRole('navigation', { name: 'Telas' });
    expect(within(nav).getByText('Ajustes')).toBeTruthy();
    expect(within(nav).queryByText('Desenvolvedor')).toBeNull();
    expect(await screen.findByText('Modo desenvolvedor desligado')).toBeTruthy();
    expect(screen.getByRole('link', { name: 'Abrir Ajustes' })).toBeTruthy();
    expect(screen.queryByRole('list', { name: 'Feedback' })).toBeNull();
  });

  it('quadro por padrão: colunas por status, próximo da fila, painel em seções e mover', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/desenvolvedor');
    await developerMode(queryClient);
    const board = await screen.findByRole('group', { name: 'Quadro de feedback' });
    const col = (name: RegExp) => within(board).getByRole('region', { name });
    const cards = (name: RegExp) =>
      within(col(name))
        .queryAllByRole('listitem')
        .map((li) => li.getAttribute('aria-label'))
        .filter(Boolean);
    expect(cards(/^Aberto/)).toEqual([
      'Feedback: Falta ferramenta para mover uma compra para outra fatura',
      'Feedback: Guia de importação não explica o CSV do Inter',
    ]);
    expect(cards(/^Em andamento/)).toHaveLength(1);
    expect(cards(/^Resolvido/)).toHaveLength(1);
    expect(col(/^Descartado/).textContent).toContain('Arraste um card para cá.');
    // Card compacto: prioridade legível, chips, reforços; sem o texto corrido.
    const top = within(col(/^Aberto/)).getAllByRole('listitem')[0]!;
    expect(within(top).getByText('Prioridade 3')).toBeTruthy();
    expect(within(top).getByText('MCP')).toBeTruthy();
    expect(within(top).getByText('Ferramenta faltando')).toBeTruthy();
    expect(within(top).getByText('2 reforços')).toBeTruthy();
    expect(within(top).queryByText(/Pediram para passar/)).toBeNull();
    // O próximo da fila: o topo de Em andamento, no destaque e no card.
    const next = screen.getByRole('region', { name: 'Próximo da fila' });
    expect(within(next).getByRole('button', { name: /get_dashboard não diz/ })).toBeTruthy();
    expect(within(next).getByText(/Prioridade 2 · Em andamento/)).toBeTruthy();
    expect(within(col(/^Em andamento/)).getByText('Próximo da fila')).toBeTruthy();
    expect(within(top).queryByText('Próximo da fila')).toBeNull();

    // Filtros por chip e busca.
    await user.click(within(screen.getByRole('group', { name: 'Tipo' })).getByText('Bug'));
    await waitFor(() => expect(cards(/^Aberto/)).toHaveLength(0));
    expect(cards(/^Resolvido/)).toHaveLength(1);
    await user.click(
      within(screen.getByRole('group', { name: 'Tipo' })).getByText('Todos os tipos'),
    );
    await user.type(screen.getByRole('searchbox', { name: 'Buscar nos feedbacks' }), 'csv INTER');
    await waitFor(() => expect(cards(/^Aberto/)).toHaveLength(1));
    await user.clear(screen.getByRole('searchbox', { name: 'Buscar nos feedbacks' }));

    // Painel lateral com as seções; a origem só aqui.
    await waitFor(() => expect(cards(/^Aberto/)).toHaveLength(2));
    await user.click(
      within(within(col(/^Aberto/)).getAllByRole('listitem')[0]!).getByRole('button'),
    );
    const panel = await screen.findByRole('dialog', { name: /Falta ferramenta para mover/ });
    for (const s of [
      'Problema',
      'Proposta',
      'Critério de aceite',
      'Onde mexer',
      'Reforços',
      'Relacionados',
    ])
      expect(within(panel).getByRole('region', { name: s })).toBeTruthy();
    expect(
      within(within(panel).getByRole('region', { name: 'Proposta' })).getByText(
        /move_to_statement/,
      ),
    ).toBeTruthy();
    expect(
      within(panel).getByRole('button', { name: 'crates/treasury-core/src/cards/mod.rs' }),
    ).toBeTruthy();
    expect(within(panel).getByText(/De novo, ao conciliar a fatura/)).toBeTruthy();
    expect(within(panel).getByText(/Registrado por agente \(CLI de teste\)/)).toBeTruthy();
    expect(within(panel).getByRole('button', { name: 'Copiar Markdown' })).toBeTruthy();

    // Descer na coluna e depois começar (vai para Em andamento); o quadro acompanha.
    const open = async () =>
      ((await mockCall('devfeedback.list', { status: 'aberto' })) as DevFeedback[]).map(
        (x) => x.title,
      );
    await user.click(within(panel).getByRole('button', { name: 'Descer' }));
    await waitFor(async () =>
      expect((await open())[0]).toBe('Guia de importação não explica o CSV do Inter'),
    );
    await user.click(within(panel).getByRole('button', { name: 'Começar' }));
    await waitFor(async () => expect(await open()).toHaveLength(1));
    const f = (await mockCall('devfeedback.get', { id: 'fb-1' })) as { feedback: DevFeedback };
    expect(f.feedback.status).toBe('em_andamento');
    expect(f.feedback.notes.at(-1)?.kind).toBe('status');
    await user.click(within(panel).getByRole('button', { name: 'Fechar' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    await waitFor(() => expect(cards(/^Em andamento/)).toHaveLength(2));
  });

  it('a lista continua a um clique: filtros, status, edição e exclusão', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked');
    const nav = await screen.findByRole('navigation', { name: 'Telas' });
    await developerMode(queryClient);
    await user.click(await within(nav).findByRole('link', { name: /Desenvolvedor/ }));
    expect(await screen.findByRole('heading', { level: 1, name: /Desenvolvedor/ })).toBeTruthy();
    await user.click(
      within(await screen.findByRole('group', { name: 'Visão' })).getByRole('button', {
        name: 'Lista',
      }),
    );

    // Abertos e em andamento, do mais importante ao menos.
    await screen.findByRole('list', { name: 'Feedback' });
    const titles = rows().map((r) => r.getAttribute('aria-label'));
    expect(titles).toEqual([
      'Feedback: Falta ferramenta para mover uma compra para outra fatura',
      'Feedback: get_dashboard não diz em que moeda vem o saldo das contas no exterior',
      'Feedback: Guia de importação não explica o CSV do Inter',
    ]);
    const top = rows()[0]!;
    expect(within(top).getByText('3')).toBeTruthy();
    expect(within(top).getByText('MCP')).toBeTruthy();
    expect(within(top).getByText('2 reforços')).toBeTruthy();
    const timeline = within(top).getByRole('list', { name: 'Reforços' });
    expect(within(timeline).getByText(/De novo, ao conciliar a fatura/)).toBeTruthy();
    expect(within(timeline).getByText('(sem nota)')).toBeTruthy();

    // Busca na tela: título, texto e reforços, sem acento nem maiúscula; limpar volta tudo.
    const search = screen.getByRole('searchbox', { name: 'Buscar nos feedbacks' });
    await user.type(search, 'GUIA importacao');
    await waitFor(() => expect(rows()).toHaveLength(1));
    await user.clear(search);
    await user.type(search, 'nada parecido com isto');
    expect(await screen.findByText(/Nenhum feedback com esse filtro/)).toBeTruthy();
    await user.clear(search);
    await waitFor(() => expect(rows()).toHaveLength(3));

    // Filtros: todos os status traz o resolvido; área filtra.
    await user.click(screen.getByRole('combobox', { name: 'Status' }));
    await user.click(await screen.findByRole('option', { name: 'Todos' }));
    await waitFor(() => expect(rows()).toHaveLength(4));
    await user.click(within(screen.getByRole('group', { name: 'Área' })).getByText('Carteira'));
    await waitFor(() => expect(rows()).toHaveLength(1));
    expect(within(rows()[0]!).getByText('Resolvido')).toBeTruthy();
    await user.click(within(rows()[0]!).getByRole('button', { name: 'Reabrir' }));
    await waitFor(() => expect(within(rows()[0]!).getByText('Aberto')).toBeTruthy());
    await user.click(
      within(screen.getByRole('group', { name: 'Área' })).getByText('Todas as áreas'),
    );
    await user.click(screen.getByRole('combobox', { name: 'Status' }));
    await user.click(await screen.findByRole('option', { name: 'Abertos e em andamento' }));
    await waitFor(() => expect(rows()).toHaveLength(4));

    // Marcar como resolvido tira da lista de abertos.
    const guia = screen.getByRole('listitem', { name: /Guia de importação/ });
    await user.click(within(guia).getByRole('button', { name: 'Resolvido' }));
    await waitFor(() =>
      expect(screen.queryByRole('listitem', { name: /Guia de importação/ })).toBeNull(),
    );

    // Editar a importância e os campos estruturados.
    const moeda = screen.getByRole('listitem', { name: /get_dashboard/ });
    await user.click(within(moeda).getByRole('button', { name: 'Editar' }));
    const dialog = await screen.findByRole('dialog', { name: 'Editar feedback' });
    const imp = within(dialog).getByLabelText('Importância');
    await user.clear(imp);
    await user.type(imp, '7');
    await user.type(within(dialog).getByLabelText('Critério de aceite'), 'Teste com conta em USD.');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar' }));
    await waitFor(() => expect(rows()[0]!.getAttribute('aria-label')).toMatch(/get_dashboard/));
    const edited = (await mockCall('devfeedback.list', {})) as DevFeedback[];
    expect(edited.find((x) => x.title.startsWith('get_dashboard'))?.acceptance_criteria).toBe(
      'Teste com conta em USD.',
    );

    // Apagar.
    const first = screen.getByRole('listitem', { name: /get_dashboard/ });
    await user.click(within(first).getByRole('button', { name: 'Apagar' }));
    await user.click(
      within(await screen.findByRole('dialog', { name: 'Apagar este feedback' })).getByRole(
        'button',
        { name: 'Apagar' },
      ),
    );
    await waitFor(() =>
      expect(screen.queryByRole('listitem', { name: /get_dashboard/ })).toBeNull(),
    );
  });
});

describe('feedback em Markdown e deduplicação do mock', () => {
  const f: DevFeedback = {
    id: 'fb-1',
    title: 'Falta ferramenta de metas',
    body: 'Precisei e não havia.',
    area: 'mcp',
    kind: 'ferramenta_faltando',
    importance: 2,
    status: 'aberto',
    created_by: 'agente (CLI de teste)',
    created_at: '2026-10-15T12:00:00Z',
    updated_at: '2026-10-15T12:00:00Z',
    last_bumped_at: '2026-10-15T13:00:00Z',
    bumps: 1,
    notes: [
      {
        id: 'n1',
        kind: 'reforco',
        note: 'De novo.',
        client: 'user',
        created_at: '2026-10-15T13:00:00Z',
      },
    ],
    position: null,
    proposal: null,
    acceptance_criteria: null,
    code_pointers: [],
    labels: [],
    assignee: null,
    expected: null,
    actual: null,
    repro_steps: null,
    user_impact: null,
    confirmed_by_user: false,
    client_version: null,
    app_version: null,
    tool_calls: [],
    closed_at: null,
    links: [],
  };

  it('vira o corpo de uma issue e um arquivo com todos', () => {
    const md = feedbackMarkdown(f);
    expect(md.startsWith('## Falta ferramenta de metas\n')).toBe(true);
    expect(md).toContain('**Área:** MCP · **Tipo:** Ferramenta faltando · **Status:** Aberto');
    expect(md).toContain('**Importância:** 2 (1 reforço)');
    expect(md).toContain('Precisei e não havia.');
    expect(md).toContain('### Reforços');
    expect(md).toMatch(/· você: De novo\./);
    const all = allMarkdown([f, { ...f, id: 'fb-2', title: 'Outro' }]);
    expect(all.startsWith('# Feedback do agente · Treasury')).toBe(true);
    expect(all).toContain('2 feedbacks.');
    expect(all.split('\n---\n')).toHaveLength(2);
  });

  it('separa as seções do texto livre e monta o Markdown para um agente de código', () => {
    const g: DevFeedback = {
      ...f,
      body: 'Problema (tela 15, visto pelo usuário): blocos de texto.\n\nProposta:\n1. Quadro.\n\nAceite: em 5 segundos.',
      notes: [
        {
          id: 'n2',
          kind: 'reforco',
          note: 'Instrução: em mcp/tools.rs aceitar from/to. Aceite: teste em mcp/tests.rs.',
          client: 'grok',
          created_at: '2026-10-15T13:00:00Z',
        },
      ],
    };
    const s = sections(g);
    expect(s.problem).toBe('blocos de texto.');
    expect(s.proposal).toBe('1. Quadro.');
    expect(s.acceptance).toBe('em 5 segundos.');
    expect(s.instructions).toEqual(['em mcp/tools.rs aceitar from/to.']);
    expect(s.pointers).toEqual(['mcp/tools.rs', 'mcp/tests.rs']);
    // Sem aceite no texto, vale o do reforço.
    expect(sections({ ...g, body: 'Só o problema.' }).acceptance).toBe('teste em mcp/tests.rs.');
    const md = taskMarkdown(g);
    expect(md.startsWith('# Falta ferramenta de metas\n')).toBe(true);
    expect(md).toContain('## Problema\n\nblocos de texto.');
    expect(md).toContain('## Critério de aceite\n\nem 5 segundos.');
    expect(md).toContain('- `mcp/tools.rs`');
    expect(md).toContain('`resolve_dev_feedback` com `id` = `fb-1`');
  });

  it('título parecido reforça em vez de duplicar', async () => {
    renderApp('unlocked');
    expect(titleSimilarity('Falta ferramenta de fatura', 'falta FERRAMENTA da fatura')).toBe(1);
    expect(titleSimilarity('Erro ao importar OFX', 'Falta ferramenta de fatura')).toBeLessThan(0.6);
    const r = (await mockCall('devfeedback.add', {
      title: 'Falta ferramenta para mover compra para outra fatura',
      body: 'De novo.',
      area: 'mcp',
      kind: 'ferramenta_faltando',
    })) as { deduplicated: boolean; feedback: DevFeedback };
    expect(r.deduplicated).toBe(true);
    expect(r.feedback.importance).toBe(4);
  });
});
