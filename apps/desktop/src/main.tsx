import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { createQueryClient } from '@/api/queryClient';
import { App } from '@/app/App';
import { installNativeFeel } from '@/app/nativeFeel';
import { installExternalLinks } from '@/app/externalLinks';
import { createAppRouter } from '@/app/router';
import { syncThemeWithDocument } from '@/state/ui';

import './styles.css';

syncThemeWithDocument();
installNativeFeel();
installExternalLinks();

const queryClient = createQueryClient();
const router = createAppRouter();

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App queryClient={queryClient} router={router} />
  </StrictMode>,
);
