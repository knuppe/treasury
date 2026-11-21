import { useMemo } from 'react';

import { useCountryFeatures } from '@/api/hooks/country';
import { useAppSettings } from '@/api/hooks/settings';

import { SCREENS, screenVisible } from './screens';

/** Modo desenvolvedor (Ajustes › Desenvolvedor, ajuste `developer_tools`). */
export function useDeveloperMode(): boolean {
  return useAppSettings().data?.developer_tools ?? false;
}

/** Telas do menu e da barra de comando (a Desenvolvedor só no modo desenvolvedor; a de
 * impostos só onde há dados do país). */
export function useVisibleScreens() {
  const dev = useDeveloperMode();
  const features = useCountryFeatures();
  return useMemo(() => SCREENS.filter((s) => screenVisible(s, dev, features)), [dev, features]);
}
