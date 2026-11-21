/**
 * Catálogo interno do design system (só em desenvolvimento: `#/dev/componentes`).
 * Todos os números desta página são fictícios.
 */
import { useMemo, useState, type ReactNode } from 'react';
import { Link } from 'react-router';

import { Brand } from '@/app/Brand';
import {
  Button,
  DateInput,
  Dialog,
  DialogClose,
  EmptyState,
  Field,
  Figure,
  Icon,
  IconButton,
  Input,
  InstrumentCard,
  Kbd,
  KeyValue,
  Label,
  LegendItem,
  LiveIndicator,
  Measure,
  Money,
  MoneyInput,
  OriginBadge,
  PasswordInput,
  ScreenHeader,
  Select,
  Sparkline,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { contrastRatio } from '@/design/contrast';
import { cx } from '@/design/cx';
import { THEMES, type ThemeId } from '@/design/themes';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, formatMonthShort, formatPercent } from '@/lib/format';
import { useUi } from '@/state/ui';

const TOKENS = [
  'bg',
  'surface',
  'surface-2',
  'line',
  'line-strong',
  'text',
  'text-dim',
  'accent',
  'positive',
  'negative',
  'warning',
] as const;

function readTokens(theme: ThemeId) {
  const style = getComputedStyle(document.documentElement);
  const value = (name: string) => style.getPropertyValue(`--${name}`).trim();
  const bg = value('bg');
  return {
    theme,
    tokens: TOKENS.map((name) => {
      const color = value(name);
      return { name, color, contrast: contrastRatio(color, bg) };
    }),
  };
}

function Section({
  index,
  title,
  children,
  note,
}: {
  index: string;
  title: string;
  children: ReactNode;
  note?: ReactNode;
}) {
  return (
    <section className="flex flex-col gap-4">
      <div className="flex items-baseline gap-3 border-b border-line pb-2">
        <span className="num text-[12px] text-accent">{index}</span>
        <h2 className="label-caps text-fg">{title}</h2>
        {note ? <span className="ml-auto text-[12px] text-dim">{note}</span> : null}
      </div>
      {children}
    </section>
  );
}

// ── Dados fictícios ──────────────────────────────────────────────

const NET_WORTH = [
  412, 418, 415, 426, 431, 429, 440, 452, 449, 461, 470, 468, 479, 488, 486, 497, 505, 503, 516,
  524, 521, 533, 541, 548,
];

const MONTHS = Array.from({ length: 24 }, (_, i) => {
  const date = new Date(Date.UTC(2024, 10 + i, 1));
  return `${date.getUTCFullYear()}-${String(date.getUTCMonth() + 1).padStart(2, '0')}`;
});

interface DemoRow {
  id: string;
  date: string;
  description: string;
  category: string;
  member: string;
  cents: number;
  estimated?: boolean;
}

const demoRows = (): DemoRow[] => [
  {
    id: '1',
    date: '2026-10-14',
    description: t('desenvolvedor-comp-ex-supermercado'),
    category: t('desenvolvedor-comp-cat-alimentacao'),
    member: 'Ana',
    cents: -48735,
  },
  {
    id: '2',
    date: '2026-10-13',
    description: t('desenvolvedor-comp-ex-salario'),
    category: t('desenvolvedor-comp-cat-receitas'),
    member: 'Bruno',
    cents: 1250000,
  },
  {
    id: '3',
    date: '2026-10-12',
    description: t('desenvolvedor-comp-ex-farmacia'),
    category: t('desenvolvedor-comp-cat-saude'),
    member: 'Ana',
    cents: -8990,
  },
  {
    id: '4',
    date: '2026-10-10',
    description: t('desenvolvedor-comp-ex-aluguel'),
    category: t('desenvolvedor-comp-cat-moradia'),
    member: 'Bruno',
    cents: -320000,
  },
  {
    id: '5',
    date: '2026-10-08',
    description: t('desenvolvedor-comp-ex-escola'),
    category: t('desenvolvedor-comp-cat-educacao'),
    member: 'Ana',
    cents: -185000,
  },
  {
    id: '6',
    date: '2026-10-05',
    description: t('desenvolvedor-comp-ex-cdb'),
    category: t('desenvolvedor-comp-cat-investimentos'),
    member: 'Bruno',
    cents: 23417,
    estimated: true,
  },
  {
    id: '7',
    date: '2026-10-03',
    description: t('desenvolvedor-comp-ex-streaming'),
    category: t('desenvolvedor-comp-cat-assinaturas'),
    member: 'Ana',
    cents: -5590,
  },
  {
    id: '8',
    date: '2026-10-01',
    description: t('desenvolvedor-comp-ex-luz'),
    category: t('desenvolvedor-comp-cat-moradia'),
    member: 'Bruno',
    cents: -21344,
  },
];

const demoColumns = (): TechColumn<DemoRow>[] => [
  {
    key: 'date',
    header: t('desenvolvedor-comp-data'),
    width: '110px',
    numeric: true,
    align: 'left',
    sortValue: (r) => r.date,
    cell: (r) => formatDate(r.date),
  },
  {
    key: 'description',
    header: t('desenvolvedor-comp-descricao-col'),
    sortValue: (r) => r.description,
  },
  {
    key: 'category',
    header: t('desenvolvedor-comp-categoria'),
    sortValue: (r) => r.category,
    cell: (r) => <span className="text-dim">{r.category}</span>,
  },
  {
    key: 'member',
    header: t('desenvolvedor-comp-membro'),
    width: '90px',
    sortValue: (r) => r.member,
  },
  {
    key: 'cents',
    header: t('desenvolvedor-comp-valor'),
    numeric: true,
    width: '160px',
    sortValue: (r) => r.cents,
    cell: (r) => (
      <span className="inline-flex items-center gap-1.5">
        <Money cents={r.cents} tone="auto" estimated={r.estimated} />
        {r.estimated ? (
          <OriginBadge
            source={t('desenvolvedor-comp-curva-cdi')}
            date={r.date}
            status="estimado"
            note={t('desenvolvedor-comp-rendimento-projetado')}
          />
        ) : null}
      </span>
    ),
  },
];

/** Gráfico SVG mínimo para demonstrar a Figura (os gráficos reais usam ECharts). */
function DemoChart() {
  const width = 1200;
  const height = 300;
  const pad = { l: 44, r: 12, t: 10, b: 24 };
  const nominal = NET_WORTH;
  const real = NET_WORTH.map((v, i) => Math.round(v / (1 + 0.0038 * i)));
  const all = [...nominal, ...real];
  const min = Math.floor(Math.min(...all) / 20) * 20;
  const max = Math.ceil(Math.max(...all) / 20) * 20;
  const x = (i: number) => pad.l + (i / (nominal.length - 1)) * (width - pad.l - pad.r);
  const y = (v: number) => pad.t + (1 - (v - min) / (max - min)) * (height - pad.t - pad.b);
  const path = (series: number[]) =>
    series.map((v, i) => `${i ? 'L' : 'M'}${x(i).toFixed(1)} ${y(v).toFixed(1)}`).join('');
  const ticks = Array.from({ length: 5 }, (_, i) => min + ((max - min) / 4) * i);
  return (
    <svg
      role="img"
      aria-label={t('desenvolvedor-comp-grafico-rotulo')}
      viewBox={`0 0 ${width} ${height}`}
      className="h-auto w-full"
    >
      {ticks.map((tick) => (
        <g key={tick}>
          <line x1={pad.l} x2={width - pad.r} y1={y(tick)} y2={y(tick)} stroke="var(--line)" />
          <text
            x={pad.l - 8}
            y={y(tick) + 3}
            textAnchor="end"
            className="num"
            fontSize="10"
            fill="var(--text-dim)"
          >
            {`${tick}k`}
          </text>
        </g>
      ))}
      {MONTHS.map((m, i) =>
        i % 4 === 0 ? (
          <text
            key={m}
            x={x(i)}
            y={height - 6}
            textAnchor="middle"
            className="num"
            fontSize="10"
            fill="var(--text-dim)"
          >
            {formatMonthShort(m)}
          </text>
        ) : null,
      )}
      <path
        d={path(real)}
        fill="none"
        stroke="var(--warning)"
        strokeWidth="1.5"
        strokeDasharray="4 3"
      />
      <path d={path(nominal)} fill="none" stroke="var(--accent)" strokeWidth="1.75" />
      <circle
        cx={x(nominal.length - 1)}
        cy={y(nominal[nominal.length - 1]!)}
        r="3"
        fill="var(--accent)"
      />
    </svg>
  );
}

export default function Componentes() {
  const theme = useUi((s) => s.theme);
  const setTheme = useUi((s) => s.setTheme);
  const privacy = useUi((s) => s.privacy);
  const togglePrivacy = useUi((s) => s.togglePrivacy);

  const palette = useMemo(() => readTokens(theme), [theme]);

  const [cents, setCents] = useState<number | null>(123456);
  const [date, setDate] = useState<string | null>('2026-10-14');
  const [currency, setCurrency] = useState('BRL');
  const [selected, setSelected] = useState<string | null>('2');
  const [dialogOpen, setDialogOpen] = useState(false);
  const rows = useMemo(() => demoRows(), []);
  const columns = useMemo(() => demoColumns(), []);

  return (
    <div className="h-screen overflow-y-auto bg-canvas">
      <header className="sticky top-0 z-10 flex flex-wrap items-center gap-4 border-b border-line bg-bg/95 px-8 py-3 backdrop-blur">
        <Brand />
        <Label tone="accent">{t('desenvolvedor-comp-selo')}</Label>
        <div
          className="ml-auto flex flex-wrap items-center gap-1"
          role="group"
          aria-label={t('desenvolvedor-comp-tema')}
        >
          {THEMES.map((th) => (
            <Button
              key={th.id}
              size="sm"
              variant={theme === th.id ? 'primary' : 'ghost'}
              aria-pressed={theme === th.id}
              onClick={() => setTheme(th.id)}
            >
              {th.label}
            </Button>
          ))}
        </div>
        <IconButton
          label={t('desenvolvedor-comp-privacidade')}
          pressed={privacy}
          shortcut={['mod', 'shift', 'H']}
          icon={<Icon name={privacy ? 'eyeOff' : 'eye'} />}
          onClick={togglePrivacy}
        />
        <Link to="/" className="label-caps hover:text-fg">
          {t('desenvolvedor-comp-voltar')}
        </Link>
      </header>

      <div className="mx-auto flex max-w-[1280px] flex-col gap-12 px-8 py-8">
        <ScreenHeader
          index="00"
          title={t('desenvolvedor-comp-titulo')}
          description={t('desenvolvedor-comp-descricao')}
          meta={t('desenvolvedor-comp-meta')}
          setDocumentTitle
        />

        <Section
          index="01"
          title={t('desenvolvedor-comp-tokens')}
          note={t('desenvolvedor-comp-contraste', { tema: palette.theme })}
        >
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
            {palette.tokens.map((tok) => (
              <div
                key={tok.name}
                className="flex flex-col overflow-hidden rounded-card border border-line"
              >
                <div className="h-14 border-b border-line" style={{ background: tok.color }} />
                <div className="flex flex-col gap-0.5 bg-surface px-2.5 py-2">
                  <span className="num text-[12px] text-fg">--{tok.name}</span>
                  <span className="num flex justify-between text-[11px] text-dim">
                    <span>{tok.color}</span>
                    <span
                      className={cx(
                        ['bg', 'surface', 'surface-2', 'line'].includes(tok.name)
                          ? undefined
                          : tok.contrast >= 4.5
                            ? 'text-positive'
                            : tok.name === 'line-strong' && tok.contrast >= 3
                              ? 'text-positive'
                              : 'text-negative',
                      )}
                    >
                      {Number.isFinite(tok.contrast) ? `${tok.contrast.toFixed(1)}:1` : '—'}
                    </span>
                  </span>
                </div>
              </div>
            ))}
          </div>
        </Section>

        <Section index="02" title={t('desenvolvedor-comp-tipografia')}>
          <div className="grid gap-6 lg:grid-cols-2">
            <div className="flex flex-col gap-3">
              <Label>{t('desenvolvedor-comp-rotulo-tipo')}</Label>
              <p className="text-[22px] font-semibold tracking-[0.05em] text-fg uppercase">
                {t('desenvolvedor-comp-titulo-tela')}
              </p>
              <p className="max-w-[60ch] text-[14px] leading-relaxed text-fg">
                {t('desenvolvedor-comp-texto-corrido')}
              </p>
              <p className="text-[13px] text-dim">{t('desenvolvedor-comp-texto-secundario')}</p>
            </div>
            <div className="flex flex-col gap-4">
              {(['md', 'lg', 'xl'] as const).map((size) => (
                <Measure
                  key={size}
                  size={size}
                  label={t('desenvolvedor-comp-medida', {
                    tamanho: size,
                    px: { md: 32, lg: 40, xl: 48 }[size],
                  })}
                  value={<Money cents={54812345} />}
                />
              ))}
            </div>
          </div>
        </Section>

        <Section index="03" title={t('desenvolvedor-comp-cards')}>
          <div className="grid gap-4 lg:grid-cols-3">
            <InstrumentCard
              index="01"
              title={t('desenvolvedor-comp-patrimonio')}
              live
              footer={[
                {
                  label: t('desenvolvedor-comp-fonte'),
                  value: t('desenvolvedor-comp-contas-carteira'),
                },
                { label: t('desenvolvedor-comp-atualizado'), value: formatDate('2026-10-14') },
              ]}
            >
              <div className="flex flex-col gap-4">
                <Measure
                  size="lg"
                  label={t('desenvolvedor-comp-total-brl')}
                  value={<Money cents={54812345} />}
                  changeKey={54812345}
                  delta={<Money cents={2310450} tone="auto" trend />}
                  hint={t('desenvolvedor-comp-12-meses')}
                  origin={{
                    source: t('desenvolvedor-comp-saldos-conciliados'),
                    date: '2026-10-14',
                  }}
                />
                <Sparkline
                  values={NET_WORTH}
                  label={t('desenvolvedor-comp-patrimonio-subindo')}
                  width={320}
                  height={44}
                  className="w-full"
                />
              </div>
            </InstrumentCard>
            <InstrumentCard
              index="02"
              title={t('desenvolvedor-comp-ipca')}
              footer={[
                { label: t('desenvolvedor-comp-fonte'), value: 'BCB/SGS 433' },
                { label: t('desenvolvedor-comp-ref'), value: t('desenvolvedor-comp-ref-valor') },
              ]}
            >
              <Measure
                size="lg"
                label={t('desenvolvedor-comp-inflacao')}
                value={<span className="italic">≈ {formatPercent(0.0447, { decimals: 2 })}</span>}
                delta={
                  <span className="num text-warning">{t('desenvolvedor-comp-delta-ipca')}</span>
                }
                hint={t('desenvolvedor-comp-projecao-focus')}
                origin={{
                  source: 'BCB/SGS 433 + Focus',
                  date: '2026-10-10',
                  status: 'estimado',
                  note: t('desenvolvedor-comp-nota-focus'),
                }}
              />
            </InstrumentCard>
            <InstrumentCard
              index="03"
              title={t('desenvolvedor-comp-poupanca')}
              footer={[
                {
                  label: t('desenvolvedor-comp-periodo'),
                  value: t('desenvolvedor-comp-periodo-valor'),
                },
                { label: t('desenvolvedor-comp-base'), value: t('desenvolvedor-comp-competencia') },
              ]}
            >
              <div className="flex flex-col gap-4">
                <Measure
                  size="lg"
                  label={t('desenvolvedor-comp-formula-poupanca')}
                  value={formatPercent(0.183)}
                  delta={
                    <span className="num text-negative">
                      {t('desenvolvedor-comp-delta-poupanca')}
                    </span>
                  }
                  hint={t('desenvolvedor-comp-vs-setembro')}
                />
                <KeyValue
                  layout="stacked"
                  items={[
                    { label: t('desenvolvedor-comp-receitas'), value: <Money cents={1273417} /> },
                    { label: t('desenvolvedor-comp-despesas'), value: <Money cents={-1040359} /> },
                    {
                      label: t('desenvolvedor-comp-saldo'),
                      value: <Money cents={233058} tone="auto" />,
                    },
                  ]}
                />
              </div>
            </InstrumentCard>
          </div>
        </Section>

        <Section index="04" title={t('desenvolvedor-comp-figura')}>
          <InstrumentCard index="04" title={t('desenvolvedor-comp-patrimonio-inflacao')}>
            <Figure
              number={4}
              title={t('desenvolvedor-comp-patrimonio-24')}
              legend={
                <>
                  <LegendItem>{t('desenvolvedor-comp-nominal')}</LegendItem>
                  <LegendItem color="var(--warning)" dashed>
                    {t('desenvolvedor-comp-real-ipca')}
                  </LegendItem>
                </>
              }
              source={t('desenvolvedor-comp-fonte-figura')}
            >
              <DemoChart />
            </Figure>
          </InstrumentCard>
        </Section>

        <Section
          index="05"
          title={t('desenvolvedor-comp-tabela')}
          note={t('desenvolvedor-comp-tabela-dica')}
        >
          <InstrumentCard
            index="05"
            title={t('desenvolvedor-comp-lancamentos-outubro')}
            padded={false}
          >
            <TechTable
              label={t('desenvolvedor-comp-lancamentos-exemplo')}
              prefsId="Lançamentos de exemplo"
              columns={columns}
              rows={rows}
              getRowId={(r) => r.id}
              selectedId={selected}
              onSelectedChange={(id) => setSelected(id)}
              onActivate={(row) =>
                toast({
                  title: t('desenvolvedor-comp-abrir', { descricao: row.description }),
                  description: t('desenvolvedor-comp-ativado'),
                })
              }
              footer={{
                description: (
                  <span className="label-caps">{t('desenvolvedor-comp-saldo-periodo')}</span>
                ),
                cents: <Money cents={rows.reduce((sum, r) => sum + r.cents, 0)} tone="auto" />,
              }}
            />
          </InstrumentCard>
        </Section>

        <Section index="06" title={t('desenvolvedor-comp-dinheiro')}>
          <div className="grid gap-6 lg:grid-cols-2">
            <KeyValue
              layout="stacked"
              items={[
                { label: t('desenvolvedor-comp-neutro'), value: <Money cents={123456} /> },
                {
                  label: t('desenvolvedor-comp-ganho'),
                  value: <Money cents={123456} tone="auto" trend />,
                },
                {
                  label: t('desenvolvedor-comp-perda'),
                  value: <Money cents={-123456} tone="auto" trend />,
                },
                {
                  label: t('desenvolvedor-comp-outra-moeda'),
                  value: <Money cents={250000} currency="USD" />,
                },
                {
                  label: t('desenvolvedor-comp-estimado'),
                  value: <Money cents={23417} estimated />,
                },
                {
                  label: t('desenvolvedor-comp-sem-moeda'),
                  value: <Money cents={-98765} hideCurrency />,
                },
                {
                  label: t('desenvolvedor-comp-sempre-visivel'),
                  value: <Money cents={4200} reveal />,
                },
              ]}
            />
            <div className="flex flex-col gap-3 text-[13px]">
              <p className="flex items-center gap-2">
                {t('desenvolvedor-comp-selo-oficial')}{' '}
                <OriginBadge source="B3 COTAHIST" date="2026-10-13" />
              </p>
              <p className="flex items-center gap-2">
                {t('desenvolvedor-comp-selo-estimado')}{' '}
                <OriginBadge
                  source={t('desenvolvedor-comp-focus-mediana')}
                  date="2026-10-10"
                  status="estimado"
                  note={t('desenvolvedor-comp-projecao-ipca')}
                />
              </p>
              <p className="text-dim">
                {withSlots(t('desenvolvedor-comp-privacidade-dica', { atalho: slot(0) }), [
                  <Kbd key="atalho" keys={['mod', 'shift', 'H']} />,
                ])}
              </p>
            </div>
          </div>
        </Section>

        <Section index="07" title={t('desenvolvedor-comp-campos')}>
          <div className="grid gap-5 md:grid-cols-2 lg:grid-cols-3">
            <Field label={t('desenvolvedor-comp-texto')} hint={t('desenvolvedor-comp-ajuda')}>
              <Input placeholder={t('desenvolvedor-comp-familia-exemplo')} />
            </Field>
            <Field
              label={t('desenvolvedor-comp-com-erro')}
              error={t('desenvolvedor-comp-informe-familia')}
            >
              <Input defaultValue="" />
            </Field>
            <Field label={t('desenvolvedor-comp-senha')}>
              <PasswordInput defaultValue={t('desenvolvedor-comp-senha-exemplo')} />
            </Field>
            <Field
              label={t('desenvolvedor-comp-valor')}
              hint={t('desenvolvedor-comp-centavos', {
                valor: cents === null ? 'null' : String(cents),
              })}
            >
              <MoneyInput
                value={cents}
                onValueChange={setCents}
                currency={currency}
                allowNegative
              />
            </Field>
            <Field label={t('desenvolvedor-comp-data')} hint={`ISO: ${date ?? 'null'}`}>
              <DateInput value={date} onValueChange={setDate} />
            </Field>
            <Field label={t('desenvolvedor-comp-moeda')}>
              <Select
                value={currency}
                onValueChange={setCurrency}
                options={[
                  { value: 'BRL', label: t('desenvolvedor-comp-real'), hint: 'BRL' },
                  { value: 'USD', label: t('desenvolvedor-comp-dolar'), hint: 'USD' },
                  { value: 'EUR', label: t('desenvolvedor-comp-euro'), hint: 'EUR' },
                  { value: 'JPY', label: t('desenvolvedor-comp-iene'), hint: 'JPY' },
                  {
                    value: 'BTC',
                    label: t('desenvolvedor-comp-bitcoin'),
                    hint: '—',
                    disabled: true,
                  },
                ]}
              />
            </Field>
          </div>
        </Section>

        <Section index="08" title={t('desenvolvedor-comp-botoes')}>
          <div className="flex flex-wrap items-center gap-3">
            <Button variant="primary">{t('desenvolvedor-comp-primario')}</Button>
            <Button variant="secondary">{t('desenvolvedor-comp-secundario')}</Button>
            <Button variant="ghost">{t('desenvolvedor-comp-fantasma')}</Button>
            <Button variant="danger">{t('desenvolvedor-comp-perigo')}</Button>
            <Button variant="demo">{t('desenvolvedor-comp-demonstracao')}</Button>
            <Button variant="primary" loading loadingText={t('desenvolvedor-comp-derivando')}>
              {t('desenvolvedor-comp-carregando')}
            </Button>
            <Button variant="secondary" disabled>
              {t('desenvolvedor-comp-desabilitado')}
            </Button>
            <Button size="sm" icon={<Icon name="plus" size={13} />}>
              {t('desenvolvedor-comp-pequeno')}
            </Button>
            <Button size="lg" variant="primary" icon={<Icon name="unlock" size={15} />}>
              {t('desenvolvedor-comp-grande')}
            </Button>
            <IconButton
              label={t('desenvolvedor-comp-bloquear')}
              icon={<Icon name="lock" />}
              shortcut={['mod', 'shift', 'L']}
            />
            <IconButton
              label={t('desenvolvedor-comp-alternancia')}
              pressed
              icon={<Icon name="eyeOff" />}
            />
            <IconButton
              label={t('desenvolvedor-comp-secundario')}
              variant="secondary"
              icon={<Icon name="refresh" />}
            />
          </div>
        </Section>

        <Section index="09" title={t('desenvolvedor-comp-dialogos')}>
          <div className="grid gap-6 lg:grid-cols-2">
            <div className="flex flex-wrap gap-2">
              <Dialog
                open={dialogOpen}
                onOpenChange={setDialogOpen}
                index="00"
                title={t('desenvolvedor-comp-confirmar-exclusao')}
                description={t('desenvolvedor-comp-confirmar-texto')}
                trigger={
                  <Button variant="secondary">{t('desenvolvedor-comp-abrir-dialogo')}</Button>
                }
                footer={
                  <>
                    <DialogClose asChild>
                      <Button variant="ghost">{t('comum-cancelar')}</Button>
                    </DialogClose>
                    <Button variant="danger" onClick={() => setDialogOpen(false)}>
                      {t('desenvolvedor-comp-excluir')}
                    </Button>
                  </>
                }
              >
                <KeyValue
                  layout="stacked"
                  items={[
                    { label: t('desenvolvedor-comp-data'), value: formatDate('2026-10-14') },
                    {
                      label: t('desenvolvedor-comp-descricao-col'),
                      value: t('desenvolvedor-comp-ex-supermercado'),
                    },
                    {
                      label: t('desenvolvedor-comp-valor'),
                      value: <Money cents={-48735} tone="auto" />,
                    },
                  ]}
                />
              </Dialog>
              {(['info', 'success', 'warning', 'error'] as const).map((tone) => (
                <Button
                  key={tone}
                  variant="ghost"
                  onClick={() =>
                    toast({
                      tone,
                      title: {
                        info: t('desenvolvedor-comp-toast-info'),
                        success: t('desenvolvedor-comp-toast-sucesso'),
                        warning: t('desenvolvedor-comp-toast-atencao'),
                        error: t('desenvolvedor-comp-toast-erro'),
                      }[tone],
                      description: t('desenvolvedor-comp-toast-texto'),
                    })
                  }
                >
                  {t('desenvolvedor-comp-notificacao', { tom: tone })}
                </Button>
              ))}
            </div>
            <Tabs defaultValue="nominal">
              <TabsList aria-label={t('desenvolvedor-comp-visao')}>
                <TabsTrigger value="nominal">{t('desenvolvedor-comp-nominal')}</TabsTrigger>
                <TabsTrigger value="real">{t('desenvolvedor-comp-real-ipca')}</TabsTrigger>
                <TabsTrigger value="m2" disabled>
                  {t('desenvolvedor-comp-m2')}
                </TabsTrigger>
              </TabsList>
              <TabsContent value="nominal">
                <Measure
                  size="md"
                  label={t('desenvolvedor-comp-rent-nominal')}
                  value={formatPercent(0.1342, { signed: true })}
                />
              </TabsContent>
              <TabsContent value="real">
                <Measure
                  size="md"
                  label={t('desenvolvedor-comp-rent-real')}
                  value={formatPercent(0.0856, { signed: true })}
                />
              </TabsContent>
            </Tabs>
          </div>
        </Section>

        <Section index="10" title={t('desenvolvedor-comp-estados')}>
          <div className="grid gap-6 lg:grid-cols-[2fr_1fr]">
            <EmptyState
              phase={t('desenvolvedor-comp-fase')}
              title={t('desenvolvedor-comp-carteira-fase')}
              description={t('desenvolvedor-comp-vazio-texto')}
              items={[
                t('desenvolvedor-comp-item-posicoes'),
                t('desenvolvedor-comp-item-proventos'),
                t('desenvolvedor-comp-item-twr'),
                t('desenvolvedor-comp-item-real'),
              ]}
              actions={<Button size="sm">{t('desenvolvedor-comp-roadmap')}</Button>}
            />
            <div className="flex flex-col gap-3">
              <LiveIndicator />
              <LiveIndicator label={t('desenvolvedor-comp-importando')} tone="negative" />
              <LiveIndicator label={t('desenvolvedor-comp-sincronizando')} tone="accent" />
              <LiveIndicator label={t('desenvolvedor-comp-pausado')} tone="dim" active={false} />
              <div className="flex gap-2">
                <Kbd keys={['mod', 'K']} />
                <Kbd keys={['mod', 'shift', 'H']} />
                <Kbd keys={['esc']} />
              </div>
              <div className="flex gap-3">
                <Label>{t('desenvolvedor-comp-rotulo-dim')}</Label>
                <Label tone="fg">{t('desenvolvedor-comp-rotulo-fg')}</Label>
                <Label tone="accent">{t('desenvolvedor-comp-rotulo-accent')}</Label>
              </div>
            </div>
          </div>
        </Section>
      </div>
    </div>
  );
}
