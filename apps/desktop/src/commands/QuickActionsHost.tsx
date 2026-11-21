import { DemoQuickDialog } from '@/features/demo/DemoQuick';

import { useQuickActions } from './quickActions';

/** Janelas das ações rápidas da barra ⌘K, montadas no casco do app. */
export function QuickActionsHost() {
  const demo = useQuickActions((s) => s.demo);
  const setDemo = useQuickActions((s) => s.setDemo);
  return <DemoQuickDialog open={demo} onOpenChange={setDemo} />;
}
