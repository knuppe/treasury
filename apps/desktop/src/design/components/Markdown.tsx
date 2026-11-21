/**
 * Markdown desenhado com elementos React (sem HTML do texto, sem `dangerouslySetInnerHTML`):
 * respostas do agente com títulos, listas, tabelas, código e links. Links https abrem no
 * navegador do sistema (no app, pelo desvio de `app/externalLinks.ts`; fora dele, numa aba
 * nova), caminhos do app (`/agente?…`) navegam na própria janela e e-mails abrem o
 * aplicativo de e-mail; o resto vira texto.
 */
import { Fragment, useMemo, type ReactNode } from 'react';
import { Link } from 'react-router';

import { cx } from '../cx';
import { parseBlocks, type Align, type Block, type Inline, type ListItem } from '@/lib/markdown';
import { t } from '@/i18n';

export interface MarkdownProps {
  text: string;
  /** Tom secundário (raciocínio). */
  tone?: 'default' | 'dim';
  className?: string;
}

function Inlines({ nodes }: { nodes: Inline[] }) {
  return (
    <>
      {nodes.map((n, i) => {
        switch (n.t) {
          case 'text':
            return <Fragment key={i}>{n.v}</Fragment>;
          case 'br':
            return <br key={i} />;
          case 'code':
            return (
              <code
                key={i}
                className="rounded-[3px] border border-line bg-surface-2 px-1 py-px font-mono text-[0.88em] text-fg"
              >
                {n.v}
              </code>
            );
          case 'strong':
            return (
              <strong key={i} className="font-semibold text-fg">
                <Inlines nodes={n.c} />
              </strong>
            );
          case 'em':
            return (
              <em key={i}>
                <Inlines nodes={n.c} />
              </em>
            );
          case 'del':
            return (
              <del key={i} className="text-dim">
                <Inlines nodes={n.c} />
              </del>
            );
          case 'link':
            return <MdLink key={i} href={n.href} label={n.c} />;
        }
      })}
    </>
  );
}

const LINK =
  'text-accent underline decoration-accent/40 underline-offset-2 hover:decoration-accent';

function MdLink({ href, label }: { href: string; label: Inline[] }) {
  const children = <Inlines nodes={label} />;
  if (href.startsWith('/')) {
    return (
      <Link to={href} className={LINK}>
        {children}
      </Link>
    );
  }
  if (href.startsWith('mailto:')) {
    return (
      <a href={href} className={LINK}>
        {children}
      </a>
    );
  }
  // Nunca navega a janela do app: no app, o desvio dos links externos manda para o
  // navegador do sistema; fora dele, abre numa aba nova. O domínio de verdade fica à vista
  // ao lado do texto (antes do clique), e um texto que parece outro endereço vira alerta.
  const host = linkHost(href);
  const mismatch = host ? labelPointsElsewhere(inlineText(label), host) : false;
  return (
    <>
      <a href={href} title={href} target="_blank" rel="noopener noreferrer" className={LINK}>
        {children}
      </a>
      {host ? (
        <span
          data-link-host
          className={cx(
            'ml-1 inline-flex items-baseline gap-0.5 rounded-[3px] border px-1 align-baseline font-mono text-[10.5px] leading-[1.4] whitespace-nowrap',
            mismatch ? 'border-warning/60 text-warning' : 'border-line text-dim',
          )}
          title={mismatch ? t('componentes-link-enganoso', { endereco: href }) : href}
        >
          {mismatch ? `⚠ ${t('componentes-link-abre')} ` : '↗ '}
          {host}
        </span>
      ) : null}
    </>
  );
}

/** Domínio do link (sem "www."), em punycode para nomes com letras parecidas. */
export function linkHost(href: string): string | null {
  try {
    const u = new URL(href);
    if (u.protocol !== 'http:' && u.protocol !== 'https:') return null;
    return u.hostname.replace(/^www\./, '');
  } catch {
    return null;
  }
}

/** O texto do link parece um endereço de outro domínio? (`itau.com.br` levando a outro lugar) */
export function labelPointsElsewhere(label: string, host: string): boolean {
  const m = /(?:https?:\/\/)?((?:[a-z0-9-]+\.)+[a-z]{2,})(?:[/:?#]|$)/i.exec(label.trim());
  if (!m) return false;
  const shown = m[1]!.toLowerCase().replace(/^www\./, '');
  return !(shown === host || host.endsWith(`.${shown}`) || shown.endsWith(`.${host}`));
}

function inlineText(nodes: Inline[]): string {
  return nodes.map((n) => ('v' in n ? n.v : 'c' in n ? inlineText(n.c) : '')).join('');
}

const HEADING_CLASS: Record<number, string> = {
  1: 'text-[16px] font-semibold',
  2: 'text-[15px] font-semibold',
  3: 'text-[14px] font-semibold',
  4: 'text-[13.5px] font-semibold',
  5: 'text-[13px] font-semibold uppercase tracking-wide text-dim',
  6: 'text-[12.5px] font-semibold uppercase tracking-wide text-dim',
};

const ALIGN: Record<string, string> = {
  left: 'text-left',
  center: 'text-center',
  right: 'text-right',
};

function alignClass(a: Align | undefined) {
  return a ? ALIGN[a] : 'text-left';
}

function Items({ items }: { items: ListItem[] }) {
  return (
    <>
      {items.map((it, i) => (
        <li key={i} className={cx('pl-0.5', it.checked !== null && 'list-none -ml-4')}>
          {it.checked !== null ? (
            <span
              aria-label={it.checked ? 'feito' : 'pendente'}
              className={cx(
                'mr-1.5 inline-grid size-3.5 place-items-center rounded-[3px] border align-[-2px] text-[9px]',
                it.checked ? 'border-accent bg-accent text-bg' : 'border-line-strong',
              )}
            >
              {it.checked ? '✓' : ''}
            </span>
          ) : null}
          <Blocks blocks={it.blocks} tight />
        </li>
      ))}
    </>
  );
}

function Blocks({
  blocks,
  tight,
}: {
  blocks: Block[];
  /** Dentro de item de lista: parágrafo sem margem. */
  tight?: boolean;
}): ReactNode {
  return blocks.map((b, i) => {
    switch (b.t) {
      case 'heading': {
        const Tag = `h${Math.min(b.level + 2, 6)}` as 'h3';
        return (
          <Tag key={i} className={cx('mt-1 text-fg first:mt-0', HEADING_CLASS[b.level])}>
            <Inlines nodes={b.c} />
          </Tag>
        );
      }
      case 'paragraph':
        return tight ? (
          <span key={i} className="block">
            <Inlines nodes={b.c} />
          </span>
        ) : (
          <p key={i}>
            <Inlines nodes={b.c} />
          </p>
        );
      case 'code':
        return (
          <pre
            key={i}
            data-lang={b.lang || undefined}
            className="max-w-full overflow-x-auto rounded-control border border-line bg-surface-2 px-3 py-2 font-mono text-[12px] leading-relaxed text-fg"
          >
            <code>{b.v}</code>
          </pre>
        );
      case 'quote':
        return (
          <blockquote
            key={i}
            className="flex flex-col gap-2 border-l-2 border-line-strong pl-3 text-dim"
          >
            <Blocks blocks={b.blocks} />
          </blockquote>
        );
      case 'list':
        return b.ordered ? (
          <ol
            key={i}
            start={b.start}
            className="flex list-decimal flex-col gap-1 pl-5 marker:text-dim"
          >
            <Items items={b.items} />
          </ol>
        ) : (
          <ul key={i} className="flex list-disc flex-col gap-1 pl-5 marker:text-dim">
            <Items items={b.items} />
          </ul>
        );
      case 'table':
        return (
          <div key={i} className="max-w-full overflow-x-auto rounded-control border border-line">
            <table className="w-full border-collapse text-[12.5px]">
              <thead className="bg-surface-2">
                <tr>
                  {b.head.map((c, k) => (
                    <th
                      key={k}
                      className={cx(
                        'border-b border-line px-2.5 py-1.5 font-semibold whitespace-nowrap text-fg',
                        alignClass(b.align[k]),
                      )}
                    >
                      <Inlines nodes={c} />
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {b.rows.map((r, ri) => (
                  <tr key={ri} className="border-b border-line last:border-b-0">
                    {r.map((c, k) => (
                      <td
                        key={k}
                        className={cx(
                          'px-2.5 py-1.5 align-top',
                          b.align[k] === 'right' && 'num',
                          alignClass(b.align[k]),
                        )}
                      >
                        <Inlines nodes={c} />
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        );
      case 'hr':
        return <hr key={i} className="border-line" />;
    }
  });
}

export function Markdown({ text, tone = 'default', className }: MarkdownProps) {
  const blocks = useMemo(() => parseBlocks(text), [text]);
  return (
    <div
      data-selectable
      className={cx(
        'flex min-w-0 flex-col gap-2 break-words',
        tone === 'dim'
          ? 'text-[12px] leading-relaxed text-dim'
          : 'text-[13.5px] leading-relaxed text-fg',
        className,
      )}
    >
      <Blocks blocks={blocks} />
    </div>
  );
}
