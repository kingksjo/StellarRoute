'use client';

import { ReactNode } from 'react';
import { ThemeProvider as NextThemesProvider } from 'next-themes';

interface ProvidersProps {
  children: ReactNode;
  defaultTheme?: string;
}

export function Providers({ children, defaultTheme = 'dark' }: ProvidersProps) {
  return (
    <NextThemesProvider
      attribute="class"
      defaultTheme={defaultTheme}
      enableSystem
      disableTransitionOnChange
    >
      {children}
    </NextThemesProvider>
  );
}
