import React from 'react';
import { cn } from '@/lib/utils';

interface GlassCardProps {
  children: React.ReactNode;
  className?: string;
  intensity?: 'subtle' | 'medium' | 'strong';
  hover?: boolean;
}

const intensityClasses = {
  subtle: 'bg-white/5 backdrop-blur-sm border-white/10',
  medium: 'bg-white/10 backdrop-blur-lg border-white/20',
  strong: 'bg-white/15 backdrop-blur-xl border-white/30',
};

export function GlassCard({ 
  children, 
  className,
  intensity = 'medium',
  hover = false 
}: GlassCardProps) {
  return (
    <div
      className={cn(
        'rounded-xl p-6 shadow-xl',
        intensityClasses[intensity],
        'border',
        hover && 'transition-all duration-300 hover:bg-white/15 hover:border-white/30 hover:shadow-2xl',
        className
      )}
    >
      {children}
    </div>
  );
}
