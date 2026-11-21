import { lazy, Suspense } from 'react';
import { createHashRouter, Navigate, type RouteObject } from 'react-router';

import { VaultGate } from '@/vault/VaultGate';

import { AppShell } from './AppShell';
import { RouteError } from './RouteError';
import { DEFAULT_SCREEN, SCREEN_COMPONENTS, SCREENS, screenMeta } from './screens';

const NotFound = lazy(() => import('@/screens/NotFound'));
const BoasVindas = lazy(() => import('@/features/profile/Onboarding'));
const Ativo = lazy(() => import('@/screens/Ativo'));

function screenRoute(id: string, path: string): RouteObject {
  const Screen = SCREEN_COMPONENTS.get(id)!;
  return { path: path.replace(/^\//, ''), element: <Screen /> };
}

/** Rotas internas de desenvolvimento (fora do cofre; não existem no build de produção). */
function devRoutes(): RouteObject[] {
  if (!import.meta.env.DEV) return [];
  const Catalog = lazy(() => import('@/screens/dev/Componentes'));
  return [
    {
      path: 'dev/componentes',
      element: (
        <Suspense fallback={null}>
          <Catalog />
        </Suspense>
      ),
    },
  ];
}

/** Todas as rotas do app (roteamento por hash, que funciona no Tauri). */
export function appRoutes(): RouteObject[] {
  return [
    {
      path: '/',
      errorElement: <RouteError />,
      children: [
        ...devRoutes(),
        {
          element: (
            <VaultGate>
              <AppShell />
            </VaultGate>
          ),
          children: [
            { index: true, element: <Navigate to={screenMeta(DEFAULT_SCREEN).path} replace /> },
            ...SCREENS.map((s) => screenRoute(s.id, s.path)),
            // A tela 06 se chamava Alimentação até a 0.8 (docs/consumption.md).
            { path: 'alimentacao', element: <Navigate to="/consumo" replace /> },
            // Página dedicada do ativo (docs/asset-page.md): id do cadastro ou ticker.
            { path: 'ativo/:ref', element: <Ativo /> },
            // Questionário de perfil (cofre novo, ou "refazer" em Ajustes › Perfil).
            { path: 'boas-vindas', element: <BoasVindas /> },
            { path: '*', element: <NotFound /> },
          ],
        },
      ],
    },
  ];
}

export function createAppRouter() {
  return createHashRouter(appRoutes());
}
