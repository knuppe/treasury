/** Junta classes condicionais (`cx('a', cond && 'b')`). */
export function cx(...parts: Array<string | false | null | undefined | 0>): string {
  return parts.filter(Boolean).join(' ');
}
