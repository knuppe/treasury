import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import type { EducationResult } from '@/api/generated/EducationResult';
import type { Member } from '@/api/generated/Member';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

import { emptyEducationForm, planFromForm, validateEducation } from './model';

const child: Member = {
  id: 'mem-lia',
  name: 'Lia',
  color: null,
  is_primary: false,
  archived: false,
  sort_order: 2,
  profile: {
    birth_date: '2018-06-15',
    relationship: 'filho',
    irpf_dependent: true,
    occupation: null,
    notes: null,
    sex: null,
  },
};

describe('college goal (model)', () => {
  it('converts the form into the plan and checks the fields', () => {
    const f = { ...emptyEducationForm('mem-lia'), fee: 250_000, durationYears: 5 };
    const p = planFromForm(f, '2026-11-15');
    expect(p.member_id).toBe('mem-lia');
    expect(p.duration_months).toBe(60);
    expect(p.real_return).toBeCloseTo(0.03);
    expect(p.fee_date).toBe('2026-11-15');
    expect(validateEducation(f, [child])).toEqual({});
    const noFee = { ...f, fee: null };
    expect(Object.keys(validateEducation(noFee, [child]))).toEqual(['fee']);
    // Public without tuition passes (the cost is the cost of living).
    expect(validateEducation({ ...noFee, institution: 'public' }, [child])).toEqual({});
    const noBirth: Member = { ...child, profile: { ...child.profile, birth_date: null } };
    expect(Object.keys(validateEducation(f, [noBirth]))).toEqual(['memberId']);
  });

  it("the mock's math: education inflation weighs on the contribution", async () => {
    renderApp('unlocked', {}, '#/orcamento?aba=metas');
    await screen.findByRole('button', { name: 'Meta de faculdade' });
    const r = (await mockCall('goals.education_simulate', {
      plan: { expected_birth: '2018-06-15', fee: 250_000, duration_months: 60 },
      birth_date: '2018-06-15',
    })) as EducationResult;
    expect(r.phase).toBe('saving');
    expect(r.target_at_start).toBeLessThan(r.total_payments);
    expect(r.monthly_headline_only).toBeLessThan(r.monthly_indexed);
    expect(r.monthly_flat).toBeGreaterThan(r.monthly_indexed);
  });
});

describe('05 · Budget › Goals › College', () => {
  it('creates the goal through the wizard and shows the year-by-year plan', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/orcamento?aba=metas');
    await user.click(await screen.findByRole('button', { name: 'Meta de faculdade' }));
    const dialog = await screen.findByRole('dialog');
    await user.click(within(dialog).getByRole('radio', { name: 'Ainda vai nascer' }));
    await user.type(within(dialog).getByLabelText('Data prevista do nascimento'), '15/06/2027');
    await user.click(within(dialog).getByRole('button', { name: 'Avançar' }));
    await user.type(within(dialog).getByLabelText('Mensalidade de hoje'), '2500');
    // The result already appears in the second step.
    expect(await within(dialog).findByText(/Necessário em/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Avançar' }));
    expect(within(dialog).getByText(/não promessa de rendimento/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Criar meta' }));
    expect(await screen.findByText('Meta criada')).toBeTruthy();
    // The card leads to the plan; the children's view appears.
    expect(await screen.findByText('Faculdade dos filhos')).toBeTruthy();
    await user.click(await screen.findByRole('button', { name: /^Ver o plano de/ }));
    const plan = await screen.findByRole('dialog');
    expect(await within(plan).findByRole('table', { name: 'Ano a ano' })).toBeTruthy();
    await user.click(within(plan).getByRole('tab', { name: 'Recálculos' }));
    expect(await within(plan).findByText('Plano criado')).toBeTruthy();
    await waitFor(async () => {
      const list = (await mockCall('goals.list', {})) as { kind: string; education?: unknown }[];
      expect(list.some((g) => g.kind === 'education' && g.education)).toBe(true);
    });
  });
});
