import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { mockChatStarts } from '@/api/mock/ai';
import type { AttachmentInfo } from '@/api/generated/AttachmentInfo';
import type { Engine } from '@/api/generated/Engine';
import { renderApp } from '@/test/renderApp';

import { unreadable } from './Attachments';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

const csv = () =>
  new File(['data;valor\n2026-10-01;-12,30\n'], 'extrato.csv', { type: 'text/csv' });
const png = () =>
  new File(
    [
      Uint8Array.from(
        atob(
          'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==',
        ),
        (c) => c.charCodeAt(0),
      ),
    ],
    'nota.png',
    {
      type: 'image/png',
    },
  );

describe('caixa de mensagem do agente: anexos e esforço', () => {
  it('anexa pelo seletor e por colar, remove, escolhe o esforço e envia tudo junto', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/agente?aba=conversa');
    const box = await screen.findByLabelText('Mensagem para o agente');
    const input = screen.getAllByTestId('attach-input')[0] as HTMLInputElement;
    await user.upload(input, [csv(), png()]);
    const tray = await screen.findByRole('list', { name: 'Anexos da mensagem' });
    await waitFor(() => expect(within(tray).getByText(/30 caracteres/)).toBeTruthy());
    expect(within(tray).getByText('nota.png')).toBeTruthy();

    // Colar (⌘V) um arquivo também anexa; o texto colado continua indo para o campo.
    fireEvent.paste(box, {
      clipboardData: {
        items: [{ kind: 'file', getAsFile: () => csv() }],
        files: [],
      },
    });
    await waitFor(() => expect(within(tray).getAllByText('extrato.csv')).toHaveLength(2));
    await user.click(within(tray).getAllByRole('button', { name: 'Remover extrato.csv' })[1]!);
    await waitFor(() => expect(within(tray).getAllByText('extrato.csv')).toHaveLength(1));

    await user.click(screen.getAllByRole('combobox', { name: /Esforço/ })[0]!);
    await user.click(await screen.findByRole('option', { name: /Alto/ }));
    expect(screen.getAllByRole('combobox', { name: /Esforço/ })[0]!.textContent).toMatch(/Alto/);

    // Só anexos, sem texto, já pode enviar.
    await user.click(screen.getAllByRole('button', { name: 'Enviar' })[0]!);
    await waitFor(() => expect(mockChatStarts()).toHaveLength(1));
    const args = mockChatStarts()[0]!;
    expect((args.attachment_ids as string[]).length).toBe(2);
    expect(args.effort).toBe('high');
    expect(args.skip_unsupported).toBeUndefined();
    expect(screen.queryByRole('list', { name: 'Anexos da mensagem' })).toBeNull();
    const thread = screen.getByRole('region', { name: 'Conversa com o agente' });
    expect((await within(thread).findAllByText(/não guardado/)).length).toBeGreaterThan(0);
  });

  it('imagem com texto lido no computador: pergunta o que mandar e lembra na conversa', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/agente?aba=conversa');
    await screen.findByLabelText('Mensagem para o agente');
    const input = screen.getAllByTestId('attach-input')[0] as HTMLInputElement;
    const file = new File([png()], 'comprovante.png', { type: 'image/png' });
    await user.upload(input, [file]);
    const group = await screen.findByRole('radiogroup', { name: 'O que mandar das imagens' });
    // Confiança alta: a sugestão é só o texto (a imagem fica no computador).
    expect(
      within(group).getByRole('radio', { name: 'Só o texto' }).getAttribute('aria-checked'),
    ).toBe('true');
    await user.click(within(group).getByRole('radio', { name: 'Os dois' }));
    await user.click(screen.getAllByRole('button', { name: 'Enviar' })[0]!);
    await waitFor(() => expect(mockChatStarts()).toHaveLength(1));
    expect(mockChatStarts()[0]!.image_mode).toBe('both');
  });

  it('com texto do OCR, só a imagem precisa de motor com visão', () => {
    const img = { id: 'a', kind: 'image', text_chars: 120 } as AttachmentInfo;
    const textOnly = { caps: { images: false, pdf: false, effort: false } } as Engine;
    expect(unreadable([img], [textOnly], 'text')).toEqual([]);
    expect(unreadable([img], [textOnly], 'both')).toEqual([]);
    expect(unreadable([img], [textOnly], 'image').map((a) => a.id)).toEqual(['a']);
  });

  it('aponta anexos que nenhum motor candidato lê', () => {
    const img = { id: 'a', kind: 'image', text_chars: 0 } as AttachmentInfo;
    const pdf = { id: 'b', kind: 'pdf', text_chars: 0 } as AttachmentInfo;
    const txt = { id: 'c', kind: 'text', text_chars: 10 } as AttachmentInfo;
    const textOnly = { caps: { images: false, pdf: false, effort: false } } as Engine;
    const vision = { caps: { images: true, pdf: true, effort: true } } as Engine;
    expect(unreadable([img, pdf, txt], [textOnly]).map((a) => a.id)).toEqual(['a', 'b']);
    expect(unreadable([img, pdf, txt], [textOnly, vision])).toEqual([]);
    expect(unreadable([img], [])).toEqual([]);
  });
});
