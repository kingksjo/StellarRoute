'use client';

import React, { useState } from 'react';
import { AlertTriangle, X } from 'lucide-react';
import { getPriceImpactSeverity, formatAmount } from '@/lib/trade-utils';

interface HighImpactModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: () => void;
    priceImpactPct: number;
    fromAmount: string;
    fromToken: string;
    toAmount: string;
    toToken: string;
}

export function HighImpactModal({
    isOpen,
    onClose,
    onConfirm,
    priceImpactPct,
    fromAmount,
    fromToken,
    toAmount,
    toToken,
}: HighImpactModalProps) {
    const [accepted, setAccepted] = useState(false);
    const impact = getPriceImpactSeverity(priceImpactPct);
    const isExtreme = priceImpactPct > 10;

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
            <div
                className="absolute inset-0 bg-black/70 backdrop-blur-sm"
                onClick={onClose}
            />

            <div className="relative w-full max-w-sm mx-4 rounded-2xl bg-[#0f0f1a] border border-red-500/20 shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-200">
                {/* Header */}
                <div className="flex items-center justify-between p-4 border-b border-white/[0.06]">
                    <div className="flex items-center gap-2">
                        <div className="p-1.5 rounded-lg bg-red-500/10">
                            <AlertTriangle className="w-5 h-5 text-red-400" />
                        </div>
                        <h2 className="text-base font-semibold text-white">
                            High Price Impact
                        </h2>
                    </div>
                    <button
                        onClick={onClose}
                        className="p-1.5 rounded-lg hover:bg-white/[0.06] transition-colors"
                    >
                        <X className="w-4.5 h-4.5 text-white/60" />
                    </button>
                </div>

                <div className="p-4 space-y-4">
                    {/* Trade details */}
                    <div className="rounded-xl bg-white/[0.02] border border-white/[0.04] p-3 space-y-2">
                        <div className="flex items-center justify-between">
                            <span className="text-xs text-white/40">You Pay</span>
                            <span className="text-sm font-medium text-white">
                                {formatAmount(fromAmount)} {fromToken}
                            </span>
                        </div>
                        <div className="flex items-center justify-between">
                            <span className="text-xs text-white/40">You Receive</span>
                            <span className="text-sm font-medium text-white">
                                {formatAmount(toAmount)} {toToken}
                            </span>
                        </div>
                        <div className="flex items-center justify-between pt-1 border-t border-white/[0.04]">
                            <span className="text-xs text-white/40">Price Impact</span>
                            <span
                                className={`text-sm font-bold px-2 py-0.5 rounded-full ${impact.color} ${impact.bgColor}`}
                            >
                                -{priceImpactPct.toFixed(2)}%
                            </span>
                        </div>
                    </div>

                    {/* Warning message */}
                    <div className="p-3 rounded-xl bg-red-500/10 border border-red-500/15">
                        <p className="text-xs text-red-300/80 leading-relaxed">
                            This swap has a price impact of{' '}
                            <strong>{priceImpactPct.toFixed(2)}%</strong>. You may receive
                            significantly less than expected. This often happens with large
                            trades or low-liquidity pairs.
                        </p>
                    </div>

                    {/* Checkbox */}
                    <label className="flex items-start gap-3 cursor-pointer group">
                        <div className="pt-0.5">
                            <input
                                type="checkbox"
                                checked={accepted}
                                onChange={(e) => setAccepted(e.target.checked)}
                                className="w-4 h-4 rounded border-white/20 bg-white/[0.04] accent-[#6366f1] cursor-pointer"
                            />
                        </div>
                        <span className="text-xs text-white/50 group-hover:text-white/70 transition-colors leading-relaxed">
                            I understand the risks and want to proceed with this trade despite
                            the high price impact.
                        </span>
                    </label>

                    {/* Buttons */}
                    <div className="flex gap-3">
                        <button
                            onClick={onClose}
                            className="flex-1 h-11 rounded-xl bg-white/[0.04] border border-white/[0.06] text-sm font-medium text-white/60 hover:bg-white/[0.08] transition-all"
                        >
                            Cancel
                        </button>
                        <button
                            onClick={() => {
                                onConfirm();
                                setAccepted(false);
                            }}
                            disabled={!accepted}
                            className={`flex-1 h-11 rounded-xl text-sm font-semibold transition-all ${accepted
                                    ? isExtreme
                                        ? 'bg-red-500/30 hover:bg-red-500/40 text-red-300 border border-red-500/30'
                                        : 'bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/20'
                                    : 'bg-white/[0.04] text-white/20 cursor-not-allowed'
                                }`}
                        >
                            {isExtreme ? 'Swap Anyway' : 'Confirm Swap'}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
