import { t } from '@/i18n';
import { MIN_MASTER_LENGTH, MIN_MASTER_SCORE, type PasswordStrength } from '@/lib/passwordStrength';

export interface NewPasswordErrors {
  password?: string;
  confirm?: string;
}

/** Regras da senha mestra no cliente (o núcleo valida de novo). */
export function validateNewPassword(
  password: string,
  confirm: string,
  strength: PasswordStrength,
): NewPasswordErrors {
  const errors: NewPasswordErrors = {};
  if (password.length === 0) errors.password = t('cofre-senha-erro-vazia');
  else if (password.length < MIN_MASTER_LENGTH) {
    errors.password = t('cofre-senha-erro-curta', { min: MIN_MASTER_LENGTH });
  } else if (strength.score < MIN_MASTER_SCORE) {
    errors.password = t('cofre-senha-erro-fraca');
  }
  if (confirm !== password) errors.confirm = t('cofre-senha-erro-confirmar');
  return errors;
}
