/**
 * Mock de `backup.*`: backups gerenciados (lista, saúde com alerta de 48 h,
 * criar, verificar, ajustes, senha própria) e restauração pedindo a senha mestra
 * e, quando o backup foi feito com outra senha, a senha do backup.
 */
import type { BackupHealth } from '@/api/generated/BackupHealth';
import type { BackupKind } from '@/api/generated/BackupKind';
import type { BackupRecord } from '@/api/generated/BackupRecord';
import type { BackupSettings } from '@/api/generated/BackupSettings';

import { onMockReset, registerMockHandlers, requireUnlocked } from '../mockRegistry';
import { ready, reseed } from './ledger/boot';
import { invalid, notFound, str, wrongPassword } from './ledger/errors';
import { hashHex } from './ledger/text';
import { verifyMasterPassword, vaultStatus } from './ledger/vaultBridge';

interface MockBackup extends BackupRecord {
  /** Senha diferente da mestra atual (backup antigo ou senha própria); `null` = a mestra. */
  password: string | null;
}

const DIR = '~/.treasury/backups';
const HOUR = 3_600_000;

let records: MockBackup[] = [];
let settings: BackupSettings = defaults();
let separatePassword: string | null = null;
let counter = 0;

function defaults(): BackupSettings {
  return {
    daily_hour: 3,
    keep_daily: 7,
    keep_weekly: 4,
    keep_monthly: 12,
    extra_destinations: [],
    on_close: true,
    key_mode: 'same',
  };
}

function record(kind: BackupKind, at: Date, extra: Partial<MockBackup> = {}): MockBackup {
  counter += 1;
  const stamp = at.toISOString().replace(/:/g, '-');
  return {
    id: `01mockbk-${String(counter).padStart(4, '0')}`,
    created_at: at.toISOString(),
    kind,
    path: `${DIR}/treasury-${stamp.replace(/\.\d{3}Z$/, (m) => m)}-${kind}.db.enc`,
    destination: 'primary',
    key_mode: 'same',
    size_bytes: 2_400_000 + counter * 13_312,
    sha256: hashHex(`${kind}${counter}`),
    verified: true,
    verify_error: null,
    exists: true,
    needs_password: false,
    password: null,
    ...extra,
  };
}

function seed(): void {
  counter = 0;
  settings = defaults();
  separatePassword = null;
  const now = Date.now();
  const at3 = (daysAgo: number) => {
    const d = new Date(now - daysAgo * 24 * HOUR);
    d.setHours(3, 0, 12, 0);
    return d;
  };
  records = [
    ...[0, 1, 2, 3, 4, 5, 6].map((d) => record('auto_daily', at3(d))),
    record('on_close', new Date(now - 30 * HOUR)),
    record('pre_import', new Date(now - 4 * 24 * HOUR - 2 * HOUR)),
    record('manual', new Date(now - 12 * 24 * HOUR), { destination: '/Volumes/Backup/Treasury' }),
    record('auto_daily', at3(21), {
      exists: false,
      verified: false,
      verify_error: 'não encontrado: arquivo do backup',
    }),
    record('pre_rekey', new Date(now - 40 * 24 * HOUR), { password: 'senha antiga do cofre' }),
    record('auto_daily', at3(60), {
      verified: false,
      verify_error: 'o hash SHA-256 não confere: arquivo alterado ou corrompido',
    }),
  ].sort((a, b) => b.created_at.localeCompare(a.created_at));
}

seed();
onMockReset(seed);

/** Backups trazidos de fora (`importados/`), fora da rotação. */
let imported: MockBackup[] = [];
onMockReset(() => {
  imported = [];
});

function asFile(r: MockBackup) {
  return {
    path: r.path,
    file_name: r.path.split('/').pop() ?? r.path,
    created_at: r.created_at,
    kind: r.kind,
    size_bytes: r.size_bytes,
    imported: imported.includes(r),
  };
}

function fileByPath(path: string): MockBackup {
  const rec = [...records, ...imported].find((r) => r.path === path && r.exists);
  if (!rec) invalid('Esse arquivo não está na pasta de backups deste cofre.');
  return rec;
}

const visible = (r: MockBackup): BackupRecord => {
  const { password: _password, ...rest } = r;
  return rest;
};

function health(): BackupHealth {
  const valid = records
    .filter((r) => r.verified && r.exists)
    .sort((a, b) => b.created_at.localeCompare(a.created_at));
  const last = valid[0]?.created_at ?? null;
  const hours = last ? Math.floor((Date.now() - Date.parse(last)) / HOUR) : null;
  const today = new Date();
  const dailyToday = records.some(
    (r) =>
      r.kind === 'auto_daily' && new Date(r.created_at).toDateString() === today.toDateString(),
  );
  return {
    last_valid_at: last,
    hours_since_last_valid: hours,
    warning: hours == null || hours > 48,
    daily_due: !dailyToday && today.getHours() >= settings.daily_hour,
    count: records.filter((r) => r.exists).length,
    total_bytes: records.filter((r) => r.exists).reduce((s, r) => s + r.size_bytes, 0),
  };
}

function get(id: string): MockBackup {
  return records.find((r) => r.id === id) ?? notFound(`backup ${id}`);
}

function create(kind: BackupKind): MockBackup {
  const rec = record(kind, new Date(), {
    key_mode: settings.key_mode,
    password: settings.key_mode === 'separate' ? separatePassword : null,
  });
  records.unshift(rec);
  for (const dest of settings.extra_destinations) {
    records.unshift({
      ...record(kind, new Date(), { key_mode: settings.key_mode }),
      destination: dest,
      path: `${dest}/treasury-${kind}.db.enc`,
    });
  }
  return rec;
}

/** Senha que abre o backup: a informada ou, sem ela, a mestra/própria (como `backup_key`). */
function checkBackupPassword(rec: MockBackup, given: string | null | undefined): void {
  if (given != null && given !== '') {
    if (rec.password ? given !== rec.password : false) wrongPassword();
    return;
  }
  if (rec.password) {
    invalid(
      rec.key_mode === 'separate'
        ? 'informe a senha de backup'
        : 'informe a senha usada quando este backup foi feito',
    );
  }
}

registerMockHandlers({
  'backup.list': () => {
    ready();
    return records.map(visible);
  },
  'backup.health': () => {
    ready();
    return health();
  },
  'backup.create': () => {
    ready();
    return visible(create('manual'));
  },
  'backup.verify': (args) => {
    ready();
    const rec = get(str(args, 'id'));
    let error: string | null = null;
    let needsPassword = false;
    if (!rec.exists) error = 'não encontrado: arquivo do backup';
    else if (rec.verify_error?.includes('SHA-256')) error = rec.verify_error;
    else {
      try {
        checkBackupPassword(rec, args.backup_password as string | undefined);
      } catch (e) {
        error = (e as { message?: string }).message ?? 'falha ao verificar';
        needsPassword = true;
      }
    }
    rec.verified = error == null;
    rec.verify_error = error;
    return { ...visible(rec), needs_password: needsPassword };
  },
  'backup.get_settings': () => {
    ready();
    return settings;
  },
  'backup.set_settings': (args) => {
    ready();
    const next = { ...defaults(), ...(args as Partial<BackupSettings>) };
    if (!(next.daily_hour >= 0 && next.daily_hour <= 23))
      invalid('hora do backup deve estar entre 0 e 23');
    if (next.key_mode === 'separate' && !separatePassword)
      invalid('defina a senha de backup antes de ativá-la');
    settings = { ...next, extra_destinations: [...next.extra_destinations] };
    return settings;
  },
  'backup.set_password': (args) => {
    ready();
    const pw = str(args, 'password');
    if ([...pw].length < 8) invalid('a senha de backup precisa de pelo menos 8 caracteres');
    separatePassword = pw;
    return null;
  },
  // Recuperação com o cofre fechado (docs/backup-recovery.md): a lista vem da pasta.
  'backup.scan': () => [...records.filter((r) => r.exists), ...imported].map(asFile),
  'backup.export_file': (args) => {
    fileByPath(str(args, 'path'));
    str(args, 'dest');
    return null;
  },
  'backup.import_file': (args) => {
    const source = str(args, 'source');
    const name = source.split('/').pop() ?? source;
    if (!name.endsWith('.enc') && !name.endsWith('.db'))
      invalid(
        'Esse arquivo não parece um backup do Treasury (os backups são bancos cifrados, terminados em .db.enc).',
      );
    counter += 1;
    const rec = record('manual', new Date(), { path: `${DIR}/importados/${name}` });
    imported.unshift(rec);
    return { ...asFile(rec), imported: true };
  },
  'backup.restore_file': async (args) => {
    const rec = fileByPath(str(args, 'path'));
    const master = str(args, 'master_password');
    checkBackupPassword(rec, args.backup_password as string | undefined);
    const { isMockUnlocked, mockRestoreLockedVault } = await import('../mock');
    if (isMockUnlocked()) {
      await verifyMasterPassword(master);
      create('pre_restore');
      reseed();
      return { status: await vaultStatus(), moved_to: null };
    }
    const movedTo = mockRestoreLockedVault(master);
    reseed();
    return { status: await vaultStatus(), moved_to: movedTo };
  },
  'backup.restore': async (args) => {
    requireUnlocked();
    ready();
    const rec = get(str(args, 'id'));
    await verifyMasterPassword(str(args, 'master_password'));
    if (!rec.exists) notFound('arquivo do backup');
    checkBackupPassword(rec, args.backup_password as string | undefined);
    create('pre_restore');
    // O mock "volta" para a família de exemplo (o estado dos backups semeados).
    reseed();
    return vaultStatus();
  },
});
