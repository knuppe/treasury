/**
 * Memória do gestor: o contexto persistente que vale para todos os motores (perfil de
 * investidor, regras da família, notas do agente). Ver, acrescentar e esquecer.
 */
import { useState } from 'react';

import type { MemoryItem } from '@/api/generated/MemoryItem';
import type { MemoryKind } from '@/api/generated/MemoryKind';
import { useForget, useMemory, useRemember } from '@/api/hooks/ai';
import {
  Button,
  Field,
  Icon,
  Input,
  InstrumentCard,
  Private,
  Select,
  toast,
} from '@/design/components';
import {
  ConfirmButton,
  ErrorBlock,
  LoadingBlock,
  errorMessage,
} from '@/features/portfolio/components/States';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';

import { MEMORY_KIND_LABEL } from './format';
import { Pill } from './ui';

const KINDS: readonly MemoryKind[] = ['profile', 'family_rule', 'note'];

function MemoryRow({ item }: { item: MemoryItem }) {
  const forget = useForget();
  return (
    <li className="flex flex-wrap items-start gap-3 px-4 py-2.5">
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <Private>
          <p className="text-[13.5px] text-fg">{item.content}</p>
        </Private>
        <p className="text-[11.5px] text-dim">
          {`${
            item.created_by === 'user' ? t('agente-gravada-voce') : t('agente-gravada-agente')
          } · ${formatDateTime(item.updated_at)}`}
        </p>
      </div>
      <ConfirmButton
        label={t('agente-esquecer')}
        title={t('agente-esquecer-titulo')}
        description={t('agente-esquecer-texto')}
        confirmLabel={t('agente-esquecer')}
        pending={forget.isPending}
        onConfirm={() =>
          forget.mutateAsync(item.id).then(
            () => toast({ tone: 'success', title: t('agente-nota-esquecida') }),
            (e: unknown) =>
              toast({
                tone: 'error',
                title: t('agente-nao-esqueceu'),
                description: errorMessage(e),
              }),
          )
        }
      />
    </li>
  );
}

function AddMemory() {
  const remember = useRemember();
  const [kind, setKind] = useState<MemoryKind>('family_rule');
  const [content, setContent] = useState('');
  const submit = () => {
    const text = content.trim();
    if (!text) return;
    remember.mutate(
      { kind, content: text },
      {
        onSuccess: () => {
          setContent('');
          toast({ tone: 'success', title: t('agente-gravado-memoria') });
        },
        onError: (e) =>
          toast({ tone: 'error', title: t('agente-nao-gravou'), description: errorMessage(e) }),
      },
    );
  };
  return (
    <form
      className="flex flex-wrap items-end gap-3 border-t border-line px-4 py-3"
      onSubmit={(e) => {
        e.preventDefault();
        submit();
      }}
    >
      <Field label={t('agente-tipo')} className="w-[200px]">
        <Select
          value={kind}
          onValueChange={(v) => setKind(v as MemoryKind)}
          options={KINDS.map((k) => ({ value: k, label: MEMORY_KIND_LABEL[k] }))}
        />
      </Field>
      <Field label={t('agente-memoria-nota')} className="min-w-[240px] flex-1">
        <Input
          value={content}
          maxLength={2000}
          placeholder={t('agente-memoria-placeholder')}
          onChange={(e) => setContent(e.target.value)}
        />
      </Field>
      <Button
        type="submit"
        variant="secondary"
        loading={remember.isPending}
        disabled={!content.trim()}
        icon={<Icon name="plus" size={12} />}
      >
        {t('agente-lembrar')}
      </Button>
    </form>
  );
}

export function MemoryPanel({ index = '01' }: { index?: string } = {}) {
  const memory = useMemory();
  const items = memory.data ?? [];
  return (
    <InstrumentCard
      index={index}
      title={t('agente-tabela-memoria')}
      padded={false}
      footer={<p className="text-[12px] text-dim">{t('agente-memoria-rodape')}</p>}
    >
      {memory.isLoading ? (
        <LoadingBlock />
      ) : memory.error ? (
        <ErrorBlock error={memory.error} className="m-3" />
      ) : items.length === 0 ? (
        <p className="px-4 py-6 text-center text-[13px] text-dim">{t('agente-memoria-vazia')}</p>
      ) : (
        KINDS.filter((k) => items.some((i) => i.kind === k)).map((k) => (
          <section
            key={k}
            aria-label={MEMORY_KIND_LABEL[k]}
            className="border-b border-line last:border-b-0"
          >
            <h3 className="flex items-center gap-2 px-4 pt-3">
              <Pill tone={k === 'note' ? 'dim' : 'accent'}>{MEMORY_KIND_LABEL[k]}</Pill>
            </h3>
            <ul className="divide-y divide-line">
              {items
                .filter((i) => i.kind === k)
                .map((i) => (
                  <MemoryRow key={i.id} item={i} />
                ))}
            </ul>
          </section>
        ))
      )}
      <AddMemory />
    </InstrumentCard>
  );
}
