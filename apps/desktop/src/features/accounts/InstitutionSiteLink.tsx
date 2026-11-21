/**
 * "Site ↗" de uma instituição (quando o cadastro dela tem o endereço): abre no navegador do
 * sistema (no app, pelo desvio de `app/externalLinks.ts`; fora dele, numa aba nova).
 */
import { useInstitutions } from '@/api/hooks/household';
import { t } from '@/i18n';

/** Site de uma instituição (`website` livre) como endereço http(s), se der. */
export function institutionUrl(website: string | null | undefined): string | null {
  const raw = website?.trim();
  if (!raw || /\s/.test(raw)) return null;
  const withScheme =
    /^[a-z][a-z0-9+.-]*:/i.test(raw) && !/^[^:/]+:\d/.test(raw)
      ? raw
      : `https://${raw.replace(/^\/\//, '')}`;
  let url: URL;
  try {
    url = new URL(withScheme);
  } catch {
    return null;
  }
  if (url.protocol !== 'https:' && url.protocol !== 'http:') return null;
  if (url.username || url.password || !url.hostname.includes('.')) return null;
  return url.href;
}

export function InstitutionSiteLink({
  institutionId,
  className,
}: {
  institutionId: string | null;
  className?: string;
}) {
  const institutions = useInstitutions();
  const inst = institutionId ? institutions.data?.find((i) => i.id === institutionId) : null;
  const url = institutionUrl(inst?.website);
  if (!inst || !url) return null;
  return (
    <a
      href={url}
      target="_blank"
      rel="noopener noreferrer"
      aria-label={t('contas-abrir-site', { nome: inst.name })}
      title={url}
      className={className ?? 'label-caps rounded-control px-1.5 py-0.5 text-dim hover:text-accent'}
    >
      {t('contas-site')}
    </a>
  );
}
