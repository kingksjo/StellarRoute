import React from 'react';
import { cn } from '@/lib/utils';

interface GlassContainerProps {
  children: React.ReactNode;
  className?: string;
  gradient?: boolean;
  gradientFrom?: string;
  gradientTo?: string;
}

export function GlassContainer({
  children,
  className,
  gradient = true,
  gradientFrom = 'from-purple-500',
  gradientTo = 'to-pink-500',
}: GlassContainerProps) {
  return (
    <div className="relative min-h-screen">
      {/* Gradient Background */}
      {gradient && (
        <div
          className={cn(
            'absolute inset-0',
            `bg-gradient-to-br ${gradientFrom} ${gradientTo}`
          )}
        />
      )}
      
      {/* Glass Content */}
      <div className={cn('relative z-10', className)}>
        {children}
      </div>
    </div>
  );
}
