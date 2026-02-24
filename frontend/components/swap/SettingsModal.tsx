'use client';

import React, { useState } from 'react';
import { X, RotateCcw, AlertTriangle } from 'lucide-react';
import { calculateMinimumReceived, formatAmount } from '@/lib/trade-utils';

interface SettingsModalProps {
    isOpen: boolean;
    onClose: () => void;
    slippage: number;
    onSlippageChange: (value: number) => void;
    deadline: number;
    onDeadlineChange: (value: number) => void;
    onReset: () => void;
    slippageWarning: 'low' | 'high' | null;
    /** Expected output amount for "Minimum Received" preview */
    expectedOutput?: string;
    outputTokenCode?: string;
}

const SLIPPAGE_PRESETS = [0.1, 0.5, 1.0];
const DEADLINE_PRESETS = [10, 30, 60];

export function SettingsModal({
    isOpen,
    onClose,
    slippage,
    onSlippageChange,
    deadline,
    onDeadlineChange,
    onReset,
    slippageWarning,
    expectedOutput,
    outputTokenCode = '',
}: SettingsModalProps) {
    const [customSlippage, setCustomSlippage] = useState('');
    const [customDeadline, setCustomDeadline] = useState('');

    if (!isOpen) return null;

    const handleCustomSlippage = (val: string) => {
        setCustomSlippage(val);
        const num = parseFloat(val);
        if (!isNaN(num) && num >= 0.01 && num <= 50) {
            onSlippageChange(num);
        }
    };

    const handleCustomDeadline = (val: string) => {
        setCustomDeadline(val);
        const num = parseInt(val, 10);
        if (!isNaN(num) && num >= 1 && num <= 1440) {
            onDeadlineChange(num);
        }
    };

    const isPresetSlippage = SLIPPAGE_PRESETS.includes(slippage);
    const isPresetDeadline = DEADLINE_PRESETS.includes(deadline);

    const minReceived =
        expectedOutput && parseFloat(expectedOutput) > 0
            ? calculateMinimumReceived(expectedOutput, slippage)
            : null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
            <div
                className="absolute inset-0 bg-black/60 backdrop-blur-sm"
                onClick={onClose}
            />

            <div className="relative w-full max-w-sm mx-4 rounded-2xl bg-[#0f0f1a] border border-white/[0.08] shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-200">
                {/* Header */}
                <div className="flex items-center justify-between p-4 border-b border-white/[0.06]">
                    <h2 className="text-base font-semibold text-white">
                        Transaction Settings
                    </h2>
                    <div className="flex items-center gap-2">
                        <button
                            onClick={onReset}
                            className="p-1.5 rounded-lg hover:bg-white/[0.06] transition-colors"
                            title="Reset to defaults"
                        >
                            <RotateCcw className="w-4 h-4 text-white/40" />
                        </button>
                        <button
                            onClick={onClose}
                            className="p-1.5 rounded-lg hover:bg-white/[0.06] transition-colors"
                        >
                            <X className="w-4.5 h-4.5 text-white/60" />
                        </button>
                    </div>
                </div>

                <div className="p-4 space-y-5">
                    {/* Slippage Tolerance */}
                    <div>
                        <div className="flex items-center gap-1.5 mb-2.5">
                            <span className="text-sm font-medium text-white/70">
                                Slippage Tolerance
                            </span>
                            <span
                                title="Your transaction will revert if the price changes unfavorably by more than this percentage."
                                className="text-white/20 cursor-help text-xs"
                            >
                                ⓘ
                            </span>
                        </div>

                        <div className="flex items-center gap-2">
                            {SLIPPAGE_PRESETS.map((preset) => (
                                <button
                                    key={preset}
                                    onClick={() => {
                                        onSlippageChange(preset);
                                        setCustomSlippage('');
                                    }}
                                    className={`flex-1 h-9 rounded-xl text-sm font-medium transition-all ${slippage === preset && isPresetSlippage
                                            ? 'bg-[#6366f1] text-white'
                                            : 'bg-white/[0.04] text-white/50 hover:bg-white/[0.08] border border-white/[0.06]'
                                        }`}
                                >
                                    {preset}%
                                </button>
                            ))}
                            <div className="relative flex-1">
                                <input
                                    type="text"
                                    inputMode="decimal"
                                    placeholder="Custom"
                                    value={
                                        !isPresetSlippage
                                            ? customSlippage || String(slippage)
                                            : customSlippage
                                    }
                                    onChange={(e) => handleCustomSlippage(e.target.value)}
                                    className={`w-full h-9 px-3 rounded-xl text-sm text-right outline-none transition-all ${!isPresetSlippage && slippage > 0
                                            ? 'bg-[#6366f1]/20 border border-[#6366f1]/40 text-white'
                                            : 'bg-white/[0.04] border border-white/[0.06] text-white/60 placeholder:text-white/25'
                                        }`}
                                />
                                <span className="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-white/30">
                                    %
                                </span>
                            </div>
                        </div>

                        {/* Slippage warning */}
                        {slippageWarning && (
                            <div
                                className={`flex items-start gap-2 mt-2 p-2.5 rounded-xl text-xs ${slippageWarning === 'low'
                                        ? 'bg-yellow-500/10 text-yellow-400'
                                        : 'bg-red-500/10 text-red-400'
                                    }`}
                            >
                                <AlertTriangle className="w-3.5 h-3.5 mt-0.5 shrink-0" />
                                <span>
                                    {slippageWarning === 'low'
                                        ? 'Your transaction may fail due to very low slippage tolerance.'
                                        : 'High slippage increases front-running risk. Proceed with caution.'}
                                </span>
                            </div>
                        )}
                    </div>

                    {/* Transaction Deadline */}
                    <div>
                        <div className="flex items-center gap-1.5 mb-2.5">
                            <span className="text-sm font-medium text-white/70">
                                Transaction Deadline
                            </span>
                            <span
                                title="Your transaction will revert if it is pending for more than this period of time."
                                className="text-white/20 cursor-help text-xs"
                            >
                                ⓘ
                            </span>
                        </div>

                        <div className="flex items-center gap-2">
                            {DEADLINE_PRESETS.map((preset) => (
                                <button
                                    key={preset}
                                    onClick={() => {
                                        onDeadlineChange(preset);
                                        setCustomDeadline('');
                                    }}
                                    className={`flex-1 h-9 rounded-xl text-sm font-medium transition-all ${deadline === preset && isPresetDeadline
                                            ? 'bg-[#6366f1] text-white'
                                            : 'bg-white/[0.04] text-white/50 hover:bg-white/[0.08] border border-white/[0.06]'
                                        }`}
                                >
                                    {preset >= 60 ? `${preset / 60}h` : `${preset}m`}
                                </button>
                            ))}
                            <div className="relative flex-1">
                                <input
                                    type="text"
                                    inputMode="numeric"
                                    placeholder="Custom"
                                    value={
                                        !isPresetDeadline
                                            ? customDeadline || String(deadline)
                                            : customDeadline
                                    }
                                    onChange={(e) => handleCustomDeadline(e.target.value)}
                                    className={`w-full h-9 px-3 rounded-xl text-sm text-right outline-none transition-all ${!isPresetDeadline && deadline > 0
                                            ? 'bg-[#6366f1]/20 border border-[#6366f1]/40 text-white'
                                            : 'bg-white/[0.04] border border-white/[0.06] text-white/60 placeholder:text-white/25'
                                        }`}
                                />
                                <span className="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-white/30">
                                    min
                                </span>
                            </div>
                        </div>
                    </div>

                    {/* Minimum Received preview */}
                    {minReceived && (
                        <div className="flex items-center justify-between p-3 rounded-xl bg-white/[0.02] border border-white/[0.04]">
                            <span className="text-xs text-white/40">Minimum Received</span>
                            <span className="text-sm font-medium text-white/80">
                                {formatAmount(minReceived)} {outputTokenCode}
                            </span>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
