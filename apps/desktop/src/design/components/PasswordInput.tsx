import { useEffect, useRef, useState } from 'react';

import { Icon } from './Icon';
import { IconButton } from './IconButton';
import { Input, type InputProps } from './Input';
import { setSecureInput } from '@/lib/osGuard';
import { t } from '@/i18n';

export type PasswordInputProps = Omit<InputProps, 'type' | 'suffix'>;

/**
 * Campo de senha com botão de mostrar/ocultar. Nunca guarda o valor sozinho. Com o foco,
 * liga o Secure Event Input do macOS, também com a senha à mostra (docs/local-security.md,
 * item 12).
 */
export function PasswordInput({ className, onFocus, onBlur, ...rest }: PasswordInputProps) {
  const [visible, setVisible] = useState(false);
  const focused = useRef(false);
  const secure = (on: boolean) => {
    if (focused.current === on) return;
    focused.current = on;
    setSecureInput(on);
  };
  useEffect(() => () => secure(false), []);
  return (
    <Input
      onFocus={(e) => {
        secure(true);
        onFocus?.(e);
      }}
      onBlur={(e) => {
        secure(false);
        onBlur?.(e);
      }}
      type={visible ? 'text' : 'password'}
      autoCapitalize="off"
      autoCorrect="off"
      spellCheck={false}
      className={className}
      suffix={
        <IconButton
          size="sm"
          label={visible ? t('componentes-ocultar-senha') : t('componentes-mostrar-senha')}
          pressed={visible}
          icon={<Icon name={visible ? 'eyeOff' : 'eye'} />}
          onClick={() => setVisible((v) => !v)}
          tooltip={false}
        />
      }
      {...rest}
    />
  );
}
