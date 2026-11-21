/**
 * A foto do cupom ao lado dos campos da compra (docs/ocr-receipts.md): aproximar e afastar (botões
 * ou roda do mouse), arrastar e girar, para conferir ou digitar o que a leitura errou. A imagem só
 * existe na memória desta tela.
 */
import { useEffect, useRef, useState, type PointerEvent } from 'react';

import { Icon, IconButton } from '@/design/components';
import { t } from '@/i18n';

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));

export function PhotoPane({ url }: { url: string }) {
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [turn, setTurn] = useState(0);
  const drag = useRef<{ x: number; y: number; px: number; py: number } | null>(null);
  const zoomBy = (f: number) => setZoom((z) => clamp(z * f, 0.5, 8));
  const reset = () => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
  };
  // Roda do mouse aproxima e afasta (ouvinte não passivo, para não rolar o diálogo junto).
  const area = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = area.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      setZoom((z) => clamp(z * (e.deltaY < 0 ? 1.15 : 1 / 1.15), 0.5, 8));
    };
    el.addEventListener('wheel', onWheel, { passive: false });
    return () => el.removeEventListener('wheel', onWheel);
  }, []);
  const onDown = (e: PointerEvent) => {
    (e.target as Element).setPointerCapture?.(e.pointerId);
    drag.current = { x: e.clientX, y: e.clientY, px: pan.x, py: pan.y };
  };
  const onMove = (e: PointerEvent) => {
    const d = drag.current;
    if (d) setPan({ x: d.px + e.clientX - d.x, y: d.py + e.clientY - d.y });
  };
  return (
    <section
      aria-label={t('cupons-foto-imagem')}
      className="relative flex h-full min-h-[320px] flex-col overflow-hidden rounded-card border border-line bg-surface-2"
    >
      <div className="absolute top-2 right-2 z-10 flex gap-1 rounded-control border border-line bg-surface p-0.5">
        <IconButton
          size="sm"
          label={t('cupons-foto-aproximar')}
          icon={<Icon name="plus" />}
          onClick={() => zoomBy(1.4)}
        />
        <IconButton
          size="sm"
          label={t('cupons-foto-afastar')}
          icon={<Icon name="minus" />}
          onClick={() => zoomBy(1 / 1.4)}
        />
        <IconButton
          size="sm"
          label={t('cupons-foto-girar')}
          icon={<Icon name="refresh" />}
          onClick={() => setTurn((r) => (r + 90) % 360)}
        />
        <IconButton
          size="sm"
          label={t('cupons-foto-ajustar')}
          icon={<Icon name="maximize" />}
          onClick={reset}
        />
      </div>
      <div
        ref={area}
        className="min-h-0 flex-1 cursor-grab touch-none select-none active:cursor-grabbing"
        onPointerDown={onDown}
        onPointerMove={onMove}
        onPointerUp={() => (drag.current = null)}
        onPointerCancel={() => (drag.current = null)}
        onDoubleClick={() => zoomBy(2)}
      >
        <img
          src={url}
          alt={t('cupons-foto-imagem')}
          draggable={false}
          className="h-full w-full object-contain"
          style={{
            transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom}) rotate(${turn}deg)`,
            transformOrigin: 'center center',
          }}
        />
      </div>
      <p className="shrink-0 border-t border-line px-3 py-1.5 text-[11.5px] text-dim">
        {t('cupons-foto-imagem-dica')}
      </p>
    </section>
  );
}
