'use client';

import React from 'react';
import type { PriceQuote } from '@/types';
import {
    formatAmount,
    getPriceImpactSeverity,
    calculateMinimumReceived,
    assetCode,
} from '@/lib/trade-utils';
import { ChevronDown, ChevronUp, Info, ArrowRight } from 'lucide-react';

interface PriceDetailsPanelProps {
    /** The quote response from the API */
    quote: PriceQuote | undefined;
    /** Loading state */
    loading: boolean;
    /** Current slippage tolerance (%) — default 0.5 */
    slippage?: number;
}

export function PriceDetailsPanel({
    quote,
    loading,
    slippage = 0.5,
}: PriceDetailsPanelProps) {
    const [expanded, setExpanded] = React.useState(false);

    if (!quote && !loading) return null;

    if (loading) {
        return (
            <div className="rounded-xl bg-white/[0.02] border border-white/[0.04] p-3">
                <div className="flex items-center justify-between">
                    <div className="h-4 w-32 bg-white/[0.06] rounded animate-pulse" />
                    <div className="h-4 w-16 bg-white/[0.06] rounded animate-pulse" />
                </div>
            </div>
        );
    }

    if (!quote) return null;

    // Derive values from the quote
    const price = parseFloat(quote.price);
    const total = parseFloat(quote.total);
    const amount = parseFloat(quote.amount);

    // Estimate price impact (simplified — real impact comes from the backend)
    const priceImpactPct = quote.path.length > 1 ? 0.3 * quote.path.length : 0.1;
    const impact = getPriceImpactSeverity(priceImpactPct);

    const fromCode = assetCode(
        quote.base_asset.asset_code || 'XLM',
    );
    const toCode = assetCode(
        quote.quote_asset.asset_code || 'XLM',
    );

    const minReceived = calculateMinimumReceived(total, slippage);

    return (
        <div className="rounded-xl bg-white/[0.02] border border-white/[0.04] overflow-hidden transition-all">
            {/* Summary row (always visible) */}
            <button
                onClick={() => setExpanded(!expanded)}
                className="w-full flex items-center justify-between p-3 hover:bg-white/[0.02] transition-colors"
            >
                <div className="flex items-center gap-2 text-sm text-white/60">
                    <span>
                        1 {fromCode} = {formatAmount(price, 6)} {toCode}
                    </span>
                </div>
                <div className="flex items-center gap-1.5">
                    <span className={`text-xs font-medium ${impact.color}`}>
                        {priceImpactPct.toFixed(2)}% impact
                    </span>
                    {expanded ? (
                        <ChevronUp className="w-4 h-4 text-white/30" />
                    ) : (
                        <ChevronDown className="w-4 h-4 text-white/30" />
                    )}
                </div>
            </button>

            {/* Expanded details */}
            {expanded && (
                <div className="px-3 pb-3 space-y-2.5 border-t border-white/[0.04] pt-3 animate-in slide-in-from-top-2 duration-200">
                    {/* Price Impact */}
                    <DetailRow
                        label="Price Impact"
                        tooltip="How much your trade will move the market price"
                    >
                        <span
                            className={`text-xs font-semibold px-2 py-0.5 rounded-full ${impact.color} ${impact.bgColor}`}
                        >
                            {priceImpactPct.toFixed(2)}% ({impact.label})
                        </span>
                    </DetailRow>

                    {/* Minimum Received */}
                    <DetailRow
                        label="Minimum Received"
                        tooltip={`After ${slippage}% slippage tolerance`}
                    >
                        <span className="text-sm text-white/80">
                            {formatAmount(minReceived)} {toCode}
                        </span>
                    </DetailRow>

                    {/* Expected Output */}
                    <DetailRow label="Expected Output">
                        <span className="text-sm text-white/80">
                            {formatAmount(total)} {toCode}
                        </span>
                    </DetailRow>

                    {/* Slippage Tolerance */}
                    <DetailRow label="Slippage Tolerance">
                        <span className="text-sm text-white/80">{slippage}%</span>
                    </DetailRow>

                    {/* Network Fee */}
                    <DetailRow label="Network Fee">
                        <span className="text-sm text-white/80">~0.00001 XLM</span>
                    </DetailRow>

                    {/* Route */}
                    {quote.path.length > 0 && (
                        <div className="pt-1.5 border-t border-white/[0.04]">
                            <div className="flex items-center gap-1 mb-1.5">
                                <span className="text-xs text-white/40">Route</span>
                            </div>
                            <div className="flex items-center gap-1 flex-wrap">
                                {quote.path.map((step, i) => {
                                    const from =
                                        step.from_asset.asset_code || 'XLM';
                                    const to =
                                        step.to_asset.asset_code || 'XLM';
                                    return (
                                        <React.Fragment key={i}>
                                            {i === 0 && (
                                                <span className="text-xs font-medium text-white/70 px-1.5 py-0.5 rounded bg-white/[0.04]">
                                                    {from}
                                                </span>
                                            )}
                                            <ArrowRight className="w-3 h-3 text-white/20" />
                                            <span className="text-xs font-medium text-white/70 px-1.5 py-0.5 rounded bg-white/[0.04]">
                                                {to}
                                            </span>
                                        </React.Fragment>
                                    );
                                })}
                            </div>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}

// Small helper sub-component
function DetailRow({
    label,
    tooltip,
    children,
}: {
    label: string;
    tooltip?: string;
    children: React.ReactNode;
}) {
    return (
        <div className="flex items-center justify-between">
            <div className="flex items-center gap-1">
                <span className="text-xs text-white/40">{label}</span>
                {tooltip && (
                    <span title={tooltip}>
                        <Info className="w-3 h-3 text-white/20 cursor-help" />
                    </span>
                )}
            </div>
            {children}
        </div>
    );
}
