'use client';

import React from 'react';
import { cn } from '@/lib/utils';
import { ChevronDown } from 'lucide-react';

interface SwapInputBoxProps {
    label: string;
    amount: string;
    onAmountChange?: (value: string) => void;
    tokenCode: string;
    onTokenClick: () => void;
    estimatedValue?: string;
    balance?: string;
    onMaxClick?: () => void;
    readOnly?: boolean;
    loading?: boolean;
}

export function SwapInputBox({
    label,
    amount,
    onAmountChange,
    tokenCode,
    onTokenClick,
    estimatedValue,
    balance,
    onMaxClick,
    readOnly = false,
    loading = false,
}: SwapInputBoxProps) {
    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const val = e.target.value;
        // Only allow valid numeric input (digits, one decimal point)
        if (val === '' || /^\d*\.?\d*$/.test(val)) {
            onAmountChange?.(val);
        }
    };

    return (
        <div className="rounded-2xl bg-white/[0.03] border border-white/[0.06] p-4 transition-all hover:border-white/[0.12] focus-within:border-[#6366f1]/40 focus-within:ring-1 focus-within:ring-[#6366f1]/20">
            {/* Header row */}
            <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-medium text-white/40 uppercase tracking-wider">
                    {label}
                </span>
                {balance && (
                    <div className="flex items-center gap-1.5">
                        <span className="text-xs text-white/40">
                            Balance: {balance}
                        </span>
                        {onMaxClick && (
                            <button
                                onClick={onMaxClick}
                                className="text-[10px] font-bold text-[#818cf8] hover:text-[#a5b4fc] uppercase tracking-wider transition-colors"
                            >
                                MAX
                            </button>
                        )}
                    </div>
                )}
            </div>

            {/* Input + token selector row */}
            <div className="flex items-center gap-3">
                <div className="flex-1 min-w-0">
                    {loading ? (
                        <div className="h-9 flex items-center">
                            <div className="w-24 h-6 rounded-md bg-white/[0.06] animate-pulse" />
                        </div>
                    ) : (
                        <input
                            type="text"
                            inputMode="decimal"
                            placeholder="0"
                            value={amount}
                            onChange={handleChange}
                            readOnly={readOnly}
                            className={cn(
                                'w-full bg-transparent text-2xl font-semibold text-white placeholder:text-white/20 outline-none',
                                readOnly && 'cursor-default',
                            )}
                        />
                    )}
                </div>

                {/* Token selector button */}
                <button
                    onClick={onTokenClick}
                    className="flex items-center gap-2 rounded-full bg-white/[0.06] hover:bg-white/[0.1] border border-white/[0.08] px-3 py-2 transition-all shrink-0"
                >
                    {/* Token icon placeholder */}
                    <div className="w-6 h-6 rounded-full bg-gradient-to-br from-[#818cf8] to-[#6366f1] flex items-center justify-center text-[10px] font-bold text-white">
                        {tokenCode.charAt(0)}
                    </div>
                    <span className="text-sm font-semibold text-white">{tokenCode}</span>
                    <ChevronDown className="w-3.5 h-3.5 text-white/40" />
                </button>
            </div>

            {/* Estimated value row */}
            {estimatedValue && (
                <div className="mt-2">
                    <span className="text-xs text-white/30">≈ ${estimatedValue}</span>
                </div>
            )}
        </div>
    );
}
