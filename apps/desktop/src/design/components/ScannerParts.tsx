import { cx } from '../cx';

/**
 * Peças do tema Scanner (scanner.css). Ficam sempre no DOM, mas só aparecem quando
 * o tema mais próximo é o Scanner (`--scanner-display` em tokens.css), então as
 * telas não precisam saber qual tema está ativo. São decorativas (`aria-hidden`).
 */

export interface ScannerSweepProps {
  /**
   * `bar`: faixa presa à borda de baixo do contêiner (posicionado), como o scanner
   * na frente do carro; `inline`: faixa maior no fluxo (telas de espera e cofre).
   */
  variant?: 'bar' | 'inline';
  /**
   * `false` desliga a luz como uma TV de tubo (recolhe e some). Na barra superior, só
   * fica ligado enquanto há tarefa do agente em andamento.
   */
  active?: boolean;
  className?: string;
}

/** Scanner: oito lâmpadas e uma luz vermelha que vai e volta com rastro. */
export function ScannerSweep({ variant = 'bar', active = true, className }: ScannerSweepProps) {
  return (
    <span
      aria-hidden
      data-variant={variant}
      data-active={active ? 'true' : 'false'}
      className={cx('scanner-sweep', className)}
    >
      <span className="scanner-halo" />
      <span className="scanner-lamps">
        <span className="scanner-beam" />
        <span className="scanner-beam scanner-trail" />
        <span className="scanner-beam scanner-trail" />
      </span>
    </span>
  );
}

/**
 * Caixa de voz do painel: três colunas de barras vermelhas que pulsam
 * enquanto o agente responde (`speaking`) e ficam baixas e quietas no resto do tempo.
 */
export function ScannerVoice({ speaking, className }: { speaking: boolean; className?: string }) {
  return (
    <span
      aria-hidden
      data-speaking={speaking ? 'true' : undefined}
      className={cx('scanner-voice', className)}
    >
      <span className="scanner-voice-bars">
        <span />
        <span />
        <span />
      </span>
    </span>
  );
}
