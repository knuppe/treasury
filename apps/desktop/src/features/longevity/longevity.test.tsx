/**
 * Life line (docs/life-expectancy.md) against the mock: off by default; when on, each member's
 * bar with the numbers from the life table (in the profile's tone and in the text alternative),
 * the request for what is missing in the profile, the settings, the suggested retirement
 * horizon, the 8th question of the questionnaire, the ⌘K bar, English and reduced motion.
 */
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { QueryClient } from '@tanstack/react-query';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { FamilyOutlook } from '@/api/generated/FamilyOutlook';
import type { LongevitySettings } from '@/api/generated/LongevitySettings';
import type { MemberOutlook } from '@/api/generated/MemberOutlook';
import type { RetirementDefaults } from '@/api/generated/RetirementDefaults';
import { mockCall } from '@/api/mock';
import { applyLocale } from '@/i18n';
import { renderApp } from '@/test/renderApp';

import { LifeBar } from './LifeBar';
import lifebarCss from './lifebar.css?raw';
import { clusterMilestones, outlookSentence, segments } from './model';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(async () => {
  vi.restoreAllMocks();
  await applyLocale('pt-BR');
});

const OFF: LongevitySettings = {
  enabled: false,
  hidden_members: [],
  show_children_end: false,
  show_advanced_later_life: false,
};

async function enable(client: QueryClient, patch: Partial<LongevitySettings> = {}) {
  await mockCall('longevity.set_settings', { ...OFF, enabled: true, ...patch });
  await client.invalidateQueries();
}

const family = async () => (await mockCall('longevity.family', {})) as FamilyOutlook;
const settings = async () => (await mockCall('longevity.settings', {})) as LongevitySettings;
const floor = Math.floor;

describe('life line', () => {
  it('off by default: Família does not show the panel', async () => {
    renderApp('unlocked', {}, '#/familia');
    expect((await screen.findAllByText('Salário · Alfa Engenharia')).length).toBeGreaterThan(0);
    expect(screen.queryByRole('region', { name: 'Linha da vida' })).toBeNull();
    expect(await settings()).toEqual(OFF);
    expect((await family()).members).toEqual([]);
  });

  it("on: each member's bar with the life table numbers, in the profile's tone, and the source", async () => {
    const { queryClient } = renderApp('unlocked', {}, '#/familia');
    await mockCall('profile.set', { tone: 'simples' });
    await enable(queryClient);
    const card = await screen.findByRole('region', { name: 'Linha da vida' });
    const f = await family();
    const ana = f.members.find((m) => m.name === 'Ana')!;
    expect(ana.median_age).toBeGreaterThan(ana.age);
    expect(ana.p25_age).toBeGreaterThan(ana.median_age);
    expect(ana.p10_age).toBeGreaterThan(ana.p25_age);

    const bar = await within(card).findByRole('group', { name: 'Linha da vida de Ana' });
    const alt = within(bar).getByRole('img').getAttribute('aria-label')!;
    expect(alt).toContain(`Linha da vida de Ana: ${ana.age} anos, Vida adulta.`);
    expect(alt).toContain(
      `Metade das pessoas da mesma geração passa dos ${floor(ana.median_age)}, 1 em 4 passa dos ${floor(ana.p25_age)} e 1 em 10 passa dos ${floor(ana.p10_age)}.`,
    );
    expect(
      within(bar).getByText(
        `Pelas médias do Brasil, de cada 100 mulheres nascidas em 1988 que chegaram aos ${ana.age}, metade passa dos ${floor(ana.median_age)} e 1 em 4 passa dos ${floor(ana.p25_age)}.`,
      ),
    ).toBeTruthy();
    expect(within(card).getByRole('group', { name: 'Linha da vida de Bruno' })).toBeTruthy();
    // Couple: the chance that at least one of the two is alive, and the source on one line.
    expect(
      within(card).getByText(
        `Pelo menos um de vocês dois: pelas médias, a chance cai a 25% em ${f.any_alive_p25_year} e a 10% em ${f.any_alive_p10_year}.`,
      ),
    ).toBeTruthy();
    expect(
      within(card).getByText('Médias da população. Fonte: ONU, WPP 2024 e IBGE 2024.'),
    ).toBeTruthy();
    expect(within(card).getByText('Como calculamos')).toBeTruthy();

    // Technical tone: median with one decimal and the mean conditioned on age.
    await mockCall('profile.set', { tone: 'tecnico' });
    await queryClient.invalidateQueries();
    expect(
      await within(card).findAllByText(/^Mediana .* · P25 .* · P10 .* · e\(\d+\) = /),
    ).toHaveLength(2);

    // Timeline: the axis becomes the calendar year.
    const user = userEvent.setup();
    await user.click(within(card).getByRole('radio', { name: 'Linha do tempo' }));
    expect(await within(card).findByText('Vida adulta')).toBeTruthy();
    expect(within(card).getByRole('group', { name: 'Linha da vida de Ana' })).toBeTruthy();
  });

  it('asks for what is missing (birth and sex, with "prefiro não informar") and a child does not show the end', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/familia');
    await mockCall('household.create_member', { name: 'Leo', profile: { relationship: 'filho' } });
    await enable(queryClient);
    const prompt = await screen.findByRole('region', { name: 'Para completar a linha da vida' });
    expect(within(prompt).getByText('Leo: falta data de nascimento e sexo')).toBeTruthy();
    await user.type(within(prompt).getByLabelText('Nascimento de Leo'), '15032018');
    await user.click(within(prompt).getByRole('combobox', { name: 'Sexo de Leo' }));
    await user.click(await screen.findByRole('option', { name: 'Prefiro não informar' }));
    await user.click(within(prompt).getByRole('button', { name: 'Salvar' }));

    const bar = await screen.findByRole('group', { name: 'Linha da vida de Leo' });
    await waitFor(() =>
      expect(screen.queryByRole('region', { name: 'Para completar a linha da vida' })).toBeNull(),
    );
    const alt = within(bar).getByRole('img').getAttribute('aria-label')!;
    expect(alt).toMatch(/^Linha da vida de Leo: \d+ anos, Infância\./);
    expect(alt).not.toContain('Metade das pessoas');
    expect(within(bar).getByText(/o final da barra fica escondido/)).toBeTruthy();
    const leo = (await family()).members.find((m) => m.name === 'Leo')!;
    expect(leo).toMatchObject({ child: true, show_end: false, sex: 'unspecified' });
  });

  it("Ajustes › Perfil: turns on, hides a member's bar, shows the end for children and turns off", async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=perfil');
    const on = await screen.findByRole('switch', { name: 'Mostrar a linha da vida' });
    expect(on.getAttribute('aria-checked')).toBe('false');
    expect(screen.queryByRole('checkbox', { name: 'Mostrar a barra de Bruno' })).toBeNull();
    await user.click(on);
    expect(await screen.findByText('Linha da vida ligada')).toBeTruthy();

    const bruno = await screen.findByRole('checkbox', { name: 'Mostrar a barra de Bruno' });
    expect((bruno as HTMLInputElement).checked).toBe(true);
    await user.click(bruno);
    await waitFor(() => expect((bruno as HTMLInputElement).checked).toBe(false));
    const kids = screen.getByRole('switch', { name: 'Mostrar o final da barra para crianças' });
    await waitFor(() => expect((kids as HTMLButtonElement).disabled).toBe(false));
    await user.click(kids);
    await waitFor(async () =>
      expect(await settings()).toEqual({
        enabled: true,
        hidden_members: ['mem-bruno'],
        show_children_end: true,
        show_advanced_later_life: false,
      }),
    );

    // In Família, Bruno's bar disappears; Ana's stays.
    window.location.hash = '#/familia';
    await screen.findByRole('heading', { level: 1, name: /Família/ });
    const card = await screen.findByRole('region', { name: 'Linha da vida' });
    expect(await within(card).findByRole('group', { name: 'Linha da vida de Ana' })).toBeTruthy();
    expect(within(card).queryByRole('group', { name: 'Linha da vida de Bruno' })).toBeNull();

    // Turning off erases nothing but the preference: the panel disappears.
    window.location.hash = '#/ajustes?secao=perfil';
    await screen.findByRole('heading', { level: 1, name: /Ajustes/ });
    const again = await screen.findByRole('switch', { name: 'Mostrar a linha da vida' });
    await waitFor(() => expect(again.getAttribute('aria-checked')).toBe('true'));
    await user.click(again);
    expect(await screen.findByText('Linha da vida desligada')).toBeTruthy();
    expect((await settings()).enabled).toBe(false);
    expect((await settings()).hidden_members).toEqual(['mem-bruno']);
  });

  it('retirement: the age comes from the birth date, the suggested horizon is the "1 in 10" and the typed value wins', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/orcamento?aba=aposentadoria');
    expect(await screen.findByText('Informe a idade de hoje para simular.')).toBeTruthy();
    await enable(queryClient);
    const d = (await mockCall('goals.retirement_defaults', {})) as RetirementDefaults;
    expect(d.from_longevity).toBe(true);
    expect(d.life_expectancy).toBeGreaterThan(95);

    const age = screen.getByLabelText<HTMLInputElement>('Idade hoje');
    await waitFor(() => expect(age.value).toBe(String(d.current_age)));
    const horizon = screen.getByLabelText<HTMLInputElement>('Expectativa de vida');
    await waitFor(() => expect(horizon.placeholder).toBe(String(d.life_expectancy)));
    // Couple (Ana and Bruno): the suggestion talks about at least one of the two.
    expect(
      screen.getByText(
        `Sugestão pela linha da vida: ${d.life_expectancy} (a chance de pelo menos um de vocês dois passar dessa idade é de 1 em 10).`,
      ),
    ).toBeTruthy();
    expect(
      await screen.findByText(
        new RegExp(`^Seu plano vai até os ${d.life_expectancy}\\. Pelas médias`),
      ),
    ).toBeTruthy();
    expect(screen.getByRole('group', { name: 'Linha da vida de Ana' })).toBeTruthy();

    // The typed value wins; "Usar" goes back to the suggestion.
    await user.type(horizon, '90');
    expect(await screen.findByText(/^Seu plano vai até os 90\. Pelas médias/)).toBeTruthy();
    await user.click(screen.getByRole('button', { name: `Usar ${d.life_expectancy}` }));
    await waitFor(() => expect(horizon.value).toBe(String(d.life_expectancy)));

    // INSS by the rules: Ana's sex and birth as the default.
    await user.click(screen.getByRole('combobox', { name: 'INSS' }));
    await user.click(await screen.findByRole('option', { name: 'Calcular pelas regras' }));
    expect(screen.getByRole('combobox', { name: 'Sexo' }).textContent).toContain('Mulher');
    expect(screen.getByLabelText<HTMLInputElement>('Data de nascimento').value).toBe('12/04/1988');
  });

  it('off, the planner keeps 95 and no bar', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/orcamento?aba=aposentadoria');
    await user.type(await screen.findByLabelText('Idade hoje'), '40');
    expect(await screen.findByText('Chance de o dinheiro durar')).toBeTruthy();
    expect(screen.getByLabelText<HTMLInputElement>('Expectativa de vida').placeholder).toBe('95');
    expect(screen.queryByText(/Sugestão pela linha da vida/)).toBeNull();
    expect(screen.queryByText(/Seu plano vai até os/)).toBeNull();
    expect(screen.queryByRole('group', { name: /Linha da vida de/ })).toBeNull();
  });

  it('family questionnaire: the 8th question, with no points, turns on the life line', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/boas-vindas');
    await user.click(
      await screen.findByRole('button', { name: /Quero personalizar o app para mim/ }),
    );
    for (let i = 1; i <= 7; i++) {
      await screen.findByText(`Pergunta ${i} de 8`);
      await user.click(screen.getByRole('button', { name: 'Pular esta pergunta' }));
    }
    expect(await screen.findByText('Quer ver a linha da vida da família?')).toBeTruthy();
    await user.click(screen.getByRole('radio', { name: 'Sim, quero ver' }));
    await user.click(screen.getByRole('button', { name: 'Concluir' }));
    expect(await screen.findByText(/A linha da vida está ligada/)).toBeTruthy();
    expect((await settings()).enabled).toBe(true);
  });

  it('⌘K bar: "Linha da vida" only with the feature on', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/painel');
    await screen.findByRole('navigation', { name: 'Telas' });
    await user.keyboard('{Control>}k{/Control}');
    let dialog = await screen.findByRole('dialog', { name: 'Barra de comando' });
    await user.type(within(dialog).getByRole('combobox'), 'expectativa de vida');
    expect(within(dialog).queryByRole('option', { name: /Família › Linha da vida/ })).toBeNull();
    await user.keyboard('{Escape}');

    await enable(queryClient);
    await user.keyboard('{Control>}k{/Control}');
    dialog = await screen.findByRole('dialog', { name: 'Barra de comando' });
    await user.type(within(dialog).getByRole('combobox'), 'expectativa de vida');
    await user.click(
      await within(dialog).findByRole('option', { name: /Família › Linha da vida/ }),
    );
    expect(await screen.findByRole('region', { name: 'Linha da vida' })).toBeTruthy();
    expect(window.location.hash).toContain('ver=linha-da-vida');
  });

  it('in English: the panel, the sentence and the source', async () => {
    await applyLocale('en');
    const { queryClient } = renderApp('unlocked', {}, '#/familia');
    await mockCall('profile.set', { tone: 'simples' });
    await enable(queryClient);
    const card = await screen.findByRole('region', { name: 'Life timeline' });
    const ana = (await family()).members.find((m) => m.name === 'Ana')!;
    const bar = await within(card).findByRole('group', { name: "Ana's life timeline" });
    expect(within(bar).getByRole('img').getAttribute('aria-label')).toContain(
      `Ana's life timeline: ${ana.age} years, Adulthood.`,
    );
    expect(
      within(bar).getByText(
        new RegExp(
          `^By the averages for .+, out of every 100 women born in 1988 who reached ${ana.age}, half live past ${floor(ana.median_age)} and 1 in 4 lives past ${floor(ana.p25_age)}\\.$`,
        ),
      ),
    ).toBeTruthy();
    expect(within(card).getByRole('radio', { name: 'Timeline' })).toBeTruthy();
    expect(within(card).getByText(/^Population averages\. Source: /)).toBeTruthy();
  });
});

// ── The bar on its own ─────────────────────────────────────────

const SAMPLE: MemberOutlook = {
  member_id: 'm1',
  name: 'Ana',
  age: 41,
  birth_year: 1985,
  sex: 'male',
  stage: 'adulthood',
  median_age: 84.2,
  p25_age: 91.4,
  p10_age: 95.8,
  mean_age: 84.1,
  remaining_mean: 43.1,
  cohort_e0: 73.6,
  period_ref: null,
  curve: [
    { age: 41, survival: 1 },
    { age: 84, survival: 0.5 },
  ],
  milestones: [
    { kind: 'retirement', age: 62, year: 2047, label: 'Aposentadoria', amount: null },
    { kind: 'inss', age: 65, year: 2050, label: 'Início do INSS', amount: 350_000 },
    { kind: 'plan_end', age: 95, year: 2080, label: 'Plano vai até', amount: null },
  ],
  child: false,
  show_end: true,
};

describe('<LifeBar>', () => {
  it('text in the three tones and the alternative with the same numbers', () => {
    const place = { code: 'BR', name: 'Brasil' };
    expect(outlookSentence(SAMPLE, 'simples', place, 'WPP 2024')).toBe(
      'Pelas médias do Brasil, de cada 100 homens nascidos em 1985 que chegaram aos 41, metade passa dos 84 e 1 em 4 passa dos 91.',
    );
    expect(outlookSentence(SAMPLE, 'equilibrado', place, 'WPP 2024')).toContain(
      '1 em 10 passa dos 95.',
    );
    expect(outlookSentence(SAMPLE, 'tecnico', place, 'WPP 2024')).toMatch(
      /^Mediana 84,2 · P25 91 · P10 96 · e\(41\) = 43,1 \(coorte, WPP 2024, BR, masculino\)/,
    );
    expect(outlookSentence({ ...SAMPLE, sex: null }, 'simples', place, '')).toContain(
      'de cada 100 pessoas nascidas em 1985',
    );
    render(<LifeBar outlook={SAMPLE} tone="simples" place={place} />);
    const alt = screen.getByRole('img').getAttribute('aria-label')!;
    expect(alt).toContain('Linha da vida de Ana: 41 anos, Vida adulta.');
    expect(alt).toContain('passa dos 84, 1 em 4 passa dos 91 e 1 em 10 passa dos 95.');
    expect(alt).toContain(
      'Aposentadoria aos 62 (2047), Início do INSS aos 65 (2050) e Plano vai até aos 95 (2080)',
    );
    // Milestones as labeled pins; the faded end (never a wall) through the mask.
    expect(screen.getByRole('button', { name: 'Aposentadoria: aos 62, em 2047' })).toBeTruthy();
    const track = document.querySelector<HTMLElement>('.life-bar-track')!;
    expect(track.style.maskImage).toContain('transparent');
  });

  it('phases in segments (80+ only with the setting) and close milestones on a single pin', () => {
    const stages = [
      { stage: 'childhood', from: 0, to: 12 },
      { stage: 'adolescence', from: 12, to: 18 },
      { stage: 'youth', from: 18, to: 30 },
      { stage: 'adulthood', from: 30, to: 60 },
      { stage: 'later_life', from: 60, to: null },
    ] as const;
    expect(segments(stages, 100).map((s) => [s.key, s.from, s.to])).toEqual([
      ['childhood', 0, 12],
      ['adolescence', 12, 18],
      ['youth', 18, 30],
      ['adulthood', 30, 60],
      ['later_life', 60, 100],
    ]);
    expect(
      segments(stages, 100, true)
        .slice(-2)
        .map((s) => s.key),
    ).toEqual(['later_life', 'advanced']);
    expect(clusterMilestones([{ x: 10 }, { x: 11 }, { x: 40 }]).map((g) => g.length)).toEqual([
      2, 1,
    ]);
  });

  it('child without the setting: phases and milestones, no end', () => {
    render(
      <LifeBar
        outlook={{ ...SAMPLE, name: 'Leo', age: 4, birth_year: 2022, child: true, show_end: false }}
        tone="simples"
        place={{ code: 'BR', name: 'Brasil' }}
      />,
    );
    const alt = screen.getByRole('img').getAttribute('aria-label')!;
    expect(alt).toContain('Linha da vida de Leo: 4 anos, Infância.');
    expect(alt).not.toContain('Metade');
    expect(screen.getByText(/Fase de hoje: Infância\./)).toBeTruthy();
  });

  it('motion: fills once on open; with "reduce motion", appears full and with no sweep', () => {
    const { container, unmount } = render(
      <LifeBar outlook={SAMPLE} tone="simples" place={{ code: 'BR', name: 'Brasil' }} />,
    );
    expect(container.querySelector('.life-bar')!.getAttribute('data-motion')).toBe('fill');
    unmount();
    vi.spyOn(window, 'matchMedia').mockImplementation(
      (query: string) =>
        ({
          matches: query.includes('prefers-reduced-motion'),
          media: query,
          addEventListener: () => {},
          removeEventListener: () => {},
        }) as unknown as MediaQueryList,
    );
    const again = render(
      <LifeBar outlook={SAMPLE} tone="simples" place={{ code: 'BR', name: 'Brasil' }} />,
    );
    expect(again.container.querySelector('.life-bar')!.getAttribute('data-motion')).toBe('static');

    // CSS: no endless animation; only transform and opacity; with reduced motion, nothing.
    expect(lifebarCss).not.toContain('infinite');
    const reduced = lifebarCss.slice(lifebarCss.indexOf('@media (prefers-reduced-motion: reduce)'));
    expect(reduced).toContain('.life-bar-fill');
    expect(reduced).toContain('.life-bar-sweep');
    expect(reduced).toContain('animation: none !important');
    for (const [, name, body] of lifebarCss.matchAll(/@keyframes ([\w-]+) \{([\s\S]*?)\n\}\n/g)) {
      const props = [...body!.matchAll(/([\w-]+):/g)].map((m) => m[1]);
      for (const p of props) expect(['opacity', 'transform'], `${name}: ${p}`).toContain(p);
    }
    // The Scanner sweep only shows on its theme.
    expect(lifebarCss).toMatch(/\.life-bar-sweep \{\s*display: var\(--scanner-display\);/);
  });
});
