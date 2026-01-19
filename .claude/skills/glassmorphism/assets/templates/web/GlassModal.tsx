import React from 'react';
import { cn } from '@/lib/utils';

interface GlassModalProps {
  isOpen: boolean;
  onClose: () => void;
  children: React.ReactNode;
  title?: string;
  className?: string;
}

export function GlassModal({
  isOpen,
  onClose,
  children,
  title,
  className,
}: GlassModalProps) {
  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      onClick={onClose}
    >
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" />
      
      {/* Modal */}
      <div
        className={cn(
          'relative',
          'bg-white/10 backdrop-blur-xl',
          'border border-white/20',
          'rounded-2xl p-8',
          'shadow-2xl',
          'max-w-md w-full mx-4',
          'max-h-[90vh] overflow-y-auto',
          className
        )}
        onClick={(e) => e.stopPropagation()}
      >
        {title && (
          <h2 className="text-white text-2xl font-bold mb-4">{title}</h2>
        )}
        {children}
        <button
          onClick={onClose}
          className="absolute top-4 right-4 text-white/80 hover:text-white text-2xl leading-none"
          aria-label="Close"
        >
          ×
        </button>
      </div>
    </div>
  );
}
