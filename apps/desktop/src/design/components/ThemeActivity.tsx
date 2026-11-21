import { cx } from '../cx';

/**
 * Atividade dos temas de cinema e do Lo-fi (activity.css): o equivalente do scanner
 * do Scanner na borda de baixo da barra superior, que só se mexe enquanto há
 * tarefa do agente em andamento. Fica sempre no DOM; cada tema mostra só a sua peça
 * (`--activity-*-display` em tokens.css), e nos outros nada aparece. Decorativo
 * (`aria-hidden`).
 */

export interface ThemeActivityProps {
  /** `false` apaga a peça, cada tema do seu jeito, e pausa as animações. */
  active?: boolean;
  className?: string;
}

export function ThemeActivity({ active = true, className }: ThemeActivityProps) {
  return (
    <span
      aria-hidden
      data-active={active ? 'true' : 'false'}
      className={cx('theme-activity', className)}
    >
      {/* Órbita: a nave, um ponto de luz quente, cruzando a lente. */}
      <span className="activity-orbit">
        <span className="activity-lens" />
        <span className="activity-star" />
      </span>
      {/* Neon: o tubo de neon e as luzes rosa e ciano. */}
      <span className="activity-neon">
        <span className="activity-tube" />
        <span className="activity-spinner" />
        <span className="activity-spinner activity-spinner-cyan" />
      </span>
      {/* Lo-fi: um ginkgo e um bordo levados pelo vento ao longo da barra. */}
      <span className="activity-lofi">
        <span className="activity-leaf" />
        <span className="activity-leaf activity-leaf-maple" />
      </span>
    </span>
  );
}
