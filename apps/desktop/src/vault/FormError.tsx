import { Icon } from '@/design/components';

/** Erro do formulário vindo do núcleo, anunciado aos leitores de tela. */
export function FormError({ error }: { error: Error | null | undefined }) {
  return (
    <div role="alert" aria-live="assertive">
      {error ? (
        <p className="flex items-start gap-2 rounded-control border border-negative/50 bg-negative/8 px-3 py-2 text-[13px] text-fg">
          <Icon name="alert" className="mt-0.5 shrink-0 text-negative" />
          <span>{error.message}</span>
        </p>
      ) : null}
    </div>
  );
}
