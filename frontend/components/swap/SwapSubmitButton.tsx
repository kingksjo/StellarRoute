'use client';

import React from 'react';
import { Loader2 } from 'lucide-react';
import { cn } from '@/lib/utils';

export type SwapButtonState =
    | 'connect-wallet'
    | 'enter-amount'
    | 'loading-quote'
    | 'insufficient-balance'
    | 'price-impact-high'
    | 'ready'
    | 'swapping';

interface SwapSubmitButtonProps {
    state: SwapButtonState;
    onClick: () => void;
    priceImpactPct?: number;
}

const BUTTON_CONFIG: Record<
    SwapButtonState,
    { label: string; disabled: boolean; variant: string }
> = {
    'connect-wallet': {
        label: 'Connect Wallet',
        disabled: false,
        variant: 'bg-[#6366f1] hover:bg-[#5558e6] text-white',
    },
    'enter-amount': {
        label: 'Enter an Amount',
        disabled: true,
        variant: 'bg-white/[0.06] text-white/30 cursor-not-allowed',
    },
    'loading-quote': {
        label: 'Fetching Quote...',
        disabled: true,
        variant: 'bg-white/[0.06] text-white/40 cursor-not-allowed',
    },
    'insufficient-balance': {
        label: 'Insufficient Balance',
        disabled: true,
        variant: 'bg-red-500/10 text-red-400 cursor-not-allowed border border-red-500/20',
    },
    'price-impact-high': {
        label: 'Swap Anyway',
        disabled: false,
        variant:
            'bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/30',
    },
    ready: {
        label: 'Swap',
        disabled: false,
        variant:
            'bg-gradient-to-r from-[#6366f1] to-[#818cf8] hover:from-[#5558e6] hover:to-[#7375f0] text-white shadow-lg shadow-[#6366f1]/20',
    },
    swapping: {
        label: 'Swapping...',
        disabled: true,
        variant:
            'bg-gradient-to-r from-[#6366f1] to-[#818cf8] text-white/80 cursor-not-allowed',
    },
};

export function SwapSubmitButton({
    state,
    onClick,
    priceImpactPct,
}: SwapSubmitButtonProps) {
    const config = BUTTON_CONFIG[state];

    // Override label for extreme price impact
    const label =
        state === 'price-impact-high' && priceImpactPct && priceImpactPct > 10
            ? 'Swap Anyway — High Risk'
            : config.label;

    return (
        <button
            onClick={onClick}
            disabled={config.disabled}
            className={cn(
                'w-full h-14 rounded-2xl text-base font-semibold transition-all duration-200',
                config.variant,
            )}
        >
            <span className="flex items-center justify-center gap-2">
                {(state === 'loading-quote' || state === 'swapping') && (
                    <Loader2 className="w-4 h-4 animate-spin" />
                )}
                {label}
            </span>
        </button>
    );
}
