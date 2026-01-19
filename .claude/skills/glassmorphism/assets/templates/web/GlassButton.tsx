import React from 'react';
import { cn } from '@/lib/utils';

interface GlassButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost';
  intensity?: 'subtle' | 'medium' | 'strong';
}

const variantClasses = {
  primary: 'bg-white/10 backdrop-blur-md border-white/20 hover:bg-white/15',
  secondary: 'bg-transparent border-2 border-white/30 hover:bg-white/10',
  ghost: 'bg-transparent border-0 hover:bg-white/10',
};

const intensityClasses = {
  subtle: 'backdrop-blur-sm',
  medium: 'backdrop-blur-md',
  strong: 'backdrop-blur-lg',
};

export function GlassButton({
  children,
  className,
  variant = 'primary',
  intensity = 'medium',
  ...props
}: GlassButtonProps) {
  return (
    <button
      className={cn(
        'rounded-lg px-6 py-3',
        'text-white font-medium',
        'border shadow-lg',
        'transition-all duration-200',
        'hover:border-white/30 active:scale-95',
        'focus:outline-none focus:ring-2 focus:ring-white/50',
        variantClasses[variant],
        intensityClasses[intensity],
        className
      )}
      {...props}
    >
      {children}
    </button>
  );
}
