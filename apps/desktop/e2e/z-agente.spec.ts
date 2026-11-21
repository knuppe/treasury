/**
 * Agente › Configurar contra o núcleo real: cadastro de um motor local (sem rede),
 * rota de chat e limite de gasto, persistidos no cofre. Nenhuma chamada a modelo é feita:
 * o teste nunca clica em "Testar" nem envia mensagem.
 *
 * O nome começa com `z-` porque o Playwright roda os arquivos em ordem alfabética e o
 * `vault.spec.ts` precisa encontrar o cofre ainda por criar; este aqui cria ou desbloqueia.
 */
import { expect, test, type Page } from '@playwright/test';

import { passGates } from './helpers';

const PASSWORD = 'senha-e2e-muito-forte-123';

async function openVault(page: Page) {
  await page.goto('/');
  await passGates(page);
  const create = page.getByLabel('Nome da família');
  const unlock = page.getByRole('button', { name: 'Desbloquear' });
  const shell = page.getByRole('navigation', { name: 'Telas' });
  // O núcleo pode já estar com o cofre aberto (sessão do teste anterior).
  await expect(create.or(unlock).or(shell)).toBeVisible();
  if (await shell.isVisible()) return;
  if (await create.isVisible()) {
    await create.fill('Família E2E');
    await page.getByLabel('Membro 1', { exact: true }).fill('Ana');
    await page.getByLabel('Senha mestra', { exact: true }).fill(PASSWORD);
    await page.getByLabel('Confirmar senha').fill(PASSWORD);
    await page
      .getByText(
        'Entendi que, se esquecer a senha e não tiver o kit de recuperação, perco os dados.',
      )
      .click();
    await page.getByRole('button', { name: 'Criar cofre cifrado' }).click();
  } else {
    await page.getByLabel('Senha mestra').fill(PASSWORD);
    await unlock.click();
  }
  await expect(page.getByText('Família E2E').first()).toBeVisible();
}

test('configura motor local, rota de chat e limite de gasto no núcleo real', async ({ page }) => {
  await openVault(page);
  await page.goto('/#/agente?aba=configurar');
  await expect(page.getByText('Nenhum motor cadastrado.', { exact: false })).toBeVisible();

  // Motor local (LM Studio): só grava o cadastro; nada é chamado.
  await page.getByRole('button', { name: 'Adicionar motor' }).click();
  const dialog = page.getByRole('dialog', { name: 'Novo motor de IA' });
  await dialog.getByRole('combobox').first().click();
  await page.getByRole('option', { name: /LM Studio/ }).click();
  await dialog.getByLabel('Nome').fill('Estúdio local');
  await dialog.getByLabel('Modelo').fill('qwen3-8b');
  await dialog.getByLabel('URL do servidor local').fill('http://127.0.0.1:1234/v1');
  await dialog.getByRole('button', { name: 'Cadastrar' }).click();
  const engine = page.getByRole('listitem', { name: 'Motor Estúdio local' });
  await expect(engine).toBeVisible();
  await expect(engine.getByText('pronto')).toBeVisible();
  await expect(engine.getByText('dados ficam neste computador').first()).toBeVisible();

  // Rota de chat com o motor novo.
  const chat = page.getByRole('listitem', { name: 'Rota: Chat com o agente' });
  await chat.getByRole('combobox', { name: /Adicionar motor à rota/ }).click();
  await page.getByRole('option', { name: /Estúdio local/ }).click();
  await expect(chat.getByText('Estúdio local')).toBeVisible();

  // Limite de gasto para um provider de API (parte "Custos" dos ajustes de IA).
  const parts = page.getByRole('tablist', { name: 'Partes dos ajustes de IA' });
  await parts.getByRole('tab', { name: 'Custos' }).click();
  await page.getByRole('combobox', { name: 'Limitar outro provider' }).click();
  await page.getByRole('option', { name: 'Anthropic (Claude)' }).click();
  const limit = page.getByRole('listitem', { name: 'Limites: Anthropic (Claude)' });
  await limit.getByLabel('Por dia (USD)').fill('2,5');
  await limit.getByLabel('Confirmar acima de (USD)').fill('0,1');
  await limit.getByRole('button', { name: 'Salvar' }).click();
  await expect(page.getByText('Limites de Anthropic (Claude) salvos').first()).toBeVisible();

  // Tudo persistiu no núcleo: recarrega e confere (a parte aberta dos ajustes fica na URL).
  await page.reload();
  await page
    .getByRole('tablist', { name: 'Partes dos ajustes de IA' })
    .getByRole('tab', { name: 'Motores' })
    .click();
  await expect(page.getByRole('listitem', { name: 'Motor Estúdio local' })).toBeVisible();
  await expect(
    page.getByRole('listitem', { name: 'Rota: Chat com o agente' }).getByText('Estúdio local'),
  ).toBeVisible();
  await page
    .getByRole('tablist', { name: 'Partes dos ajustes de IA' })
    .getByRole('tab', { name: 'Custos' })
    .click();
  const saved = page.getByRole('listitem', { name: 'Limites: Anthropic (Claude)' });
  await expect(saved.getByLabel('Por dia (USD)')).toHaveValue('2,5');
  await expect(saved.getByLabel('Confirmar acima de (USD)')).toHaveValue('0,1');

  // A conversa mostra o motor que vai receber os dados, e a fila começa vazia.
  await page.getByRole('tab', { name: 'Conversa' }).click();
  const indicator = page.getByRole('status', { name: 'Motor que recebe os dados' });
  await expect(indicator.getByText('Estúdio local')).toBeVisible();
  await expect(indicator.getByText('dados ficam neste computador')).toBeVisible();
  await page.getByRole('tab', { name: /Aprovação/ }).click();
  await expect(page.getByText('Nenhuma proposta esperando aprovação.')).toBeVisible();
  await expect(page.getByText(/Modo autônomo desligado/)).toBeVisible();
});

test('memória de procedimentos e acesso ao computador contra o núcleo real', async ({ page }) => {
  await openVault(page);
  // A conversa começa sem acesso ao computador.
  await page.goto('/#/agente?aba=conversa');
  const sw = page.getByRole('switch', { name: 'Acesso ao computador nesta conversa' });
  await expect(sw).toHaveAttribute('aria-checked', 'false');
  await sw.click();
  await expect(sw).toHaveAttribute('aria-checked', 'true');

  // Nota gravada pela interface (o agente grava sozinho; aqui só o usuário).
  await page.getByRole('tab', { name: /Memória/ }).click();
  await expect(page.getByText(/O agente ainda não gravou procedimentos/)).toBeVisible();
  await page.getByRole('button', { name: 'Adicionar' }).click();
  const add = page.getByRole('dialog', { name: 'Novo procedimento' });
  await add.getByLabel('Título').fill('Extrato OFX do banco');
  await add.getByLabel('Site').fill('https://www.banco.com.br/extrato?sessao=abc');
  await add.getByLabel('Formato').fill('ofx');
  await add.getByLabel('Passos').fill('Entrar (senha: 1234) › Extrato › Exportar › OFX');
  await add.getByRole('button', { name: 'Salvar' }).click();
  const row = page.getByRole('listitem', { name: 'Procedimento: Extrato OFX do banco' });
  await expect(row).toBeVisible();
  // O núcleo guarda só o domínio, o formato em maiúsculas e oculta a senha.
  await expect(row.getByText('www.banco.com.br')).toBeVisible();
  await expect(row.getByText('arquivo OFX')).toBeVisible();
  await expect(row.getByText(/senha: \[oculto\]/)).toBeVisible();
  await expect(row.getByText(/Gravado por você/)).toBeVisible();

  // Persistiu no cofre; editar e apagar.
  await page.reload();
  const saved = page.getByRole('listitem', { name: 'Procedimento: Extrato OFX do banco' });
  await saved.getByRole('button', { name: 'Editar' }).click();
  const edit = page.getByRole('dialog', { name: 'Editar procedimento' });
  await edit.getByLabel('Passos').fill('Menu Extratos › Exportar › OFX');
  await edit.getByRole('button', { name: 'Salvar' }).click();
  await expect(saved.getByText('Menu Extratos › Exportar › OFX')).toBeVisible();
  await saved.getByRole('button', { name: 'Apagar' }).click();
  await page
    .getByRole('dialog', { name: 'Apagar este procedimento' })
    .getByRole('button', { name: 'Apagar' })
    .click();
  await expect(page.getByText(/O agente ainda não gravou procedimentos/)).toBeVisible();
});
