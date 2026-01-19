import React from 'react';
import { cn } from '@/lib/utils';

interface GlassNavbarProps {
  children: React.ReactNode;
  className?: string;
  sticky?: boolean;
  intensity?: 'subtle' | 'medium' | 'strong';
}

const intensityClasses = {
  subtle: 'bg-white/5 backdrop-blur-sm border-white/10',
  medium: 'bg-white/5 backdrop-blur-md border-white/10',
  strong: 'bg-white/10 backdrop-blur-lg border-white/20',
};

export function GlassNavbar({
  children,
  className,
  sticky = true,
  intensity = 'medium',
}: GlassNavbarProps) {
  return (
    <nav
      className={cn(
        'border-b',
        intensityClasses[intensity],
        sticky && 'sticky top-0 z-50',
        'px-6 py-4',
        className
      )}
    >
      <div className="max-w-7xl mx-auto flex items-center justify-between">
        {children}
      </div>
    </nav>
  );
}
