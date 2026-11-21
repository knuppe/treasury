/** O que o Treasury percebe nos dados sem IA (`reports.insights`). */
import type { Insights } from '../generated/Insights';
import { useCore } from './coreQuery';

export const useInsights = (enabled = true) =>
  useCore<Insights>('reports', 'insights', {}, { enabled });
