#!/usr/bin/env node
/**
 * Contrato das telas Contas, Lançamentos e Ajustes contra o núcleo
 * real (`treasury-dev serve`). Exercita os comandos que as telas usam, na ordem de
 * um uso real, e confere as formas das respostas contra os tipos gerados
 * (`src/api/contract/shapes.ts`) e regras de negócio (saldo, divisão, câmbio,
 * desfazer, conflito).
 *
 * Uso (Node 22.18+, que lê TypeScript sem compilar):
 *
 *   treasury-dev serve --home /tmp/tdl --port 7781 --token dev --no-mcp-socket
 *   node scripts/contract-ledger.mjs                 # usa http://127.0.0.1:7781, token "dev"
 *   TREASURY_API=http://127.0.0.1:7777 TREASURY_TOKEN=dev TREASURY_PASSWORD='…' node scripts/contract-ledger.mjs
 *
 * Opções: `--sync` chama market.sync (rede); `--no-restore` não restaura o backup no
 * fim; `--json` imprime o resultado em JSON. Sem cofre, cria um com a senha de teste.
 * Sai com código 1 se algum passo falhar (avisos não falham).
 */
import { mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { runLedgerContract } from '../src/api/contract/flow.ts';

const API = (process.env.TREASURY_API ?? 'http://127.0.0.1:7781').replace(/\/+$/, '');
const TOKEN = process.env.TREASURY_TOKEN ?? 'dev';
const PASSWORD = process.env.TREASURY_PASSWORD ?? 'senha de teste 2026';
const args = new Set(process.argv.slice(2));

async function call(cmd, params) {
  let res;
  try {
    res = await fetch(`${API}/api`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${TOKEN}` },
      body: JSON.stringify({ cmd, args: params ?? null }),
    });
  } catch (e) {
    throw { code: 'network', message: `sem conexão com ${API}: ${e.message}` };
  }
  const body = await res.json();
  if (body.ok) return body.data;
  throw body.error ?? { code: 'bad_response', message: `HTTP ${res.status}` };
}

const status = await call('vault.status');
if (status.state === 'no_vault') {
  await call('vault.create', {
    password: PASSWORD,
    household_name: 'Família Contrato',
    members: ['Ana', 'Bruno'],
  });
} else if (status.state === 'locked') {
  await call('vault.unlock', { password: PASSWORD });
}

const exportDir = mkdtempSync(join(tmpdir(), 'treasury-contrato-'));
const today = new Date();
const iso = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`;
const results = await runLedgerContract(call, {
  password: PASSWORD,
  exportDir,
  today: iso,
  restore: !args.has('--no-restore'),
  sync: args.has('--sync'),
});

if (args.has('--json')) {
  console.log(JSON.stringify(results, null, 2));
} else {
  const errors = results.filter((r) => !r.ok);
  const warnings = results.flatMap((r) =>
    r.findings.filter((f) => f.level === 'aviso').map((f) => ({ ...f, step: r.step, cmd: r.cmd })),
  );
  for (const r of results) {
    const mark = r.ok ? 'ok ' : 'ERRO';
    console.log(`${mark}  ${r.cmd.padEnd(34)} ${r.step}`);
    if (r.error) console.log(`      ${r.error}`);
    for (const f of r.findings.filter((x) => x.level === 'erro'))
      console.log(`      ${f.path}: ${f.message}`);
  }
  const unique = new Map();
  for (const w of warnings) unique.set(`${w.cmd} ${w.message}`, w);
  if (unique.size) {
    console.log('\nAvisos (divergências de forma que não quebram a tela):');
    for (const w of unique.values()) console.log(`  ${w.cmd}: ${w.message} (${w.path})`);
  }
  console.log(
    `\n${results.length} passos · ${results.length - errors.length} ok · ${errors.length} com erro · ${unique.size} aviso(s) · núcleo em ${API}`,
  );
}
process.exit(results.some((r) => !r.ok) ? 1 : 0);
