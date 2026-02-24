'use client';

import React, { useCallback, useMemo, useState } from 'react';
import { ArrowDownUp, Settings } from 'lucide-react';
import { usePairs, useQuote } from '@/hooks/useApi';
import { useTradeSettings } from '@/hooks/useTradeSettings';
import { getPriceImpactSeverity } from '@/lib/trade-utils';
import { SwapInputBox } from './SwapInputBox';
import { TokenSelectorModal } from './TokenSelectorModal';
import { PriceDetailsPanel } from './PriceDetailsPanel';
import { SwapSubmitButton, type SwapButtonState } from './SwapSubmitButton';
import { SettingsModal } from './SettingsModal';
import { HighImpactModal } from './HighImpactModal';

export function SwapInterface() {
    // ── Token selection state ───────────────────────────────────────────
    const [fromAsset, setFromAsset] = useState({ code: 'XLM', id: 'native' });
    const [toAsset, setToAsset] = useState({ code: 'USDC', id: 'USDC' });
    const [fromAmount, setFromAmount] = useState('');
    const [selectorOpen, setSelectorOpen] = useState<'from' | 'to' | null>(null);
    const [settingsOpen, setSettingsOpen] = useState(false);
    const [highImpactOpen, setHighImpactOpen] = useState(false);
    const [isConnected] = useState(false); // Wallet stub

    // ── Trade settings (persisted) ──────────────────────────────────────
    const {
        slippage,
        setSlippage,
        deadline,
        setDeadline,
        reset: resetSettings,
        slippageWarning,
    } = useTradeSettings();

    // ── Data fetching ───────────────────────────────────────────────────
    const { data: pairs, loading: pairsLoading } = usePairs();

    const parsedAmount = useMemo(() => {
        const n = parseFloat(fromAmount);
        return isNaN(n) || n <= 0 ? undefined : n;
    }, [fromAmount]);

    const { data: quote, loading: quoteLoading } = useQuote(
        fromAsset.id,
        toAsset.id,
        parsedAmount,
    );

    // ── Derived values ──────────────────────────────────────────────────
    const toAmount = useMemo(() => {
        if (!quote) return '';
        return quote.total;
    }, [quote]);

    const priceImpactPct = useMemo(() => {
        if (!quote) return 0;
        return quote.path.length > 1 ? 0.3 * quote.path.length : 0.1;
    }, [quote]);

    const impact = useMemo(
        () => getPriceImpactSeverity(priceImpactPct),
        [priceImpactPct],
    );

    // ── Button state machine ────────────────────────────────────────────
    const buttonState: SwapButtonState = useMemo(() => {
        if (!isConnected) return 'connect-wallet';
        if (!fromAmount || parseFloat(fromAmount) <= 0) return 'enter-amount';
        if (quoteLoading) return 'loading-quote';
        if (priceImpactPct > 5) return 'price-impact-high';
        return 'ready';
    }, [isConnected, fromAmount, quoteLoading, priceImpactPct]);

    // ── Handlers ────────────────────────────────────────────────────────
    const handleFlip = useCallback(() => {
        setFromAsset(toAsset);
        setToAsset(fromAsset);
        setFromAmount(toAmount || '');
    }, [fromAsset, toAsset, toAmount]);

    const handleTokenSelect = useCallback(
        (identifier: string, code: string) => {
            if (selectorOpen === 'from') {
                setFromAsset({ code, id: identifier });
            } else {
                setToAsset({ code, id: identifier });
            }
            setSelectorOpen(null);
        },
        [selectorOpen],
    );

    const executeSwap = useCallback(() => {
        console.log('Execute swap:', {
            fromAsset,
            toAsset,
            fromAmount,
            quote,
            slippage,
            deadline,
        });
        setHighImpactOpen(false);
    }, [fromAsset, toAsset, fromAmount, quote, slippage, deadline]);

    const handleSwapClick = useCallback(() => {
        if (buttonState === 'connect-wallet') {
            console.log('Wallet connect triggered');
            return;
        }
        if (buttonState === 'price-impact-high') {
            setHighImpactOpen(true);
            return;
        }
        if (buttonState === 'ready') {
            executeSwap();
        }
    }, [buttonState, executeSwap]);

    return (
        <div className="w-full max-w-[480px] mx-auto">
            {/* Swap Card */}
            <div className="relative rounded-3xl bg-[#12121f]/80 backdrop-blur-xl border border-white/[0.06] shadow-2xl shadow-black/40 overflow-hidden">
                {/* Glow effect */}
                <div className="absolute -top-24 -left-24 w-48 h-48 bg-[#6366f1]/10 rounded-full blur-3xl pointer-events-none" />
                <div className="absolute -bottom-24 -right-24 w-48 h-48 bg-[#818cf8]/8 rounded-full blur-3xl pointer-events-none" />

                {/* Header */}
                <div className="relative flex items-center justify-between px-5 pt-5 pb-2">
                    <div className="flex items-center gap-3">
                        <h2 className="text-lg font-semibold text-white">Swap</h2>
                        {/* Slippage indicator */}
                        <span
                            className={`text-[11px] px-2 py-0.5 rounded-full ${slippageWarning === 'high'
                                    ? 'bg-red-500/10 text-red-400'
                                    : slippageWarning === 'low'
                                        ? 'bg-yellow-500/10 text-yellow-400'
                                        : 'bg-white/[0.04] text-white/30'
                                }`}
                        >
                            {slippage}% slippage
                        </span>
                    </div>
                    <button
                        onClick={() => setSettingsOpen(true)}
                        className="p-2 rounded-xl hover:bg-white/[0.06] transition-colors"
                    >
                        <Settings className="w-5 h-5 text-white/40" />
                    </button>
                </div>

                {/* Body */}
                <div className="relative px-4 pb-5 space-y-1">
                    {/* From input */}
                    <SwapInputBox
                        label="You Pay"
                        amount={fromAmount}
                        onAmountChange={setFromAmount}
                        tokenCode={fromAsset.code}
                        onTokenClick={() => setSelectorOpen('from')}
                        balance={isConnected ? '1,234.56' : undefined}
                        onMaxClick={
                            isConnected ? () => setFromAmount('1234.56') : undefined
                        }
                    />

                    {/* Flip button */}
                    <div className="flex justify-center -my-3 relative z-10">
                        <button
                            onClick={handleFlip}
                            className="p-2.5 rounded-xl bg-[#1a1a2e] border border-white/[0.08] hover:border-[#6366f1]/40 hover:bg-[#1e1e35] transition-all group"
                        >
                            <ArrowDownUp className="w-4 h-4 text-white/40 group-hover:text-[#818cf8] transition-colors" />
                        </button>
                    </div>

                    {/* To input */}
                    <SwapInputBox
                        label="You Receive"
                        amount={toAmount}
                        tokenCode={toAsset.code}
                        onTokenClick={() => setSelectorOpen('to')}
                        readOnly
                        loading={quoteLoading && !!parsedAmount}
                    />

                    {/* Price impact badge (always visible when quote is available) */}
                    {quote && parsedAmount && (
                        <div className="flex items-center justify-between px-1 pt-1">
                            <span className="text-xs text-white/30">Price Impact</span>
                            <span
                                className={`text-xs font-semibold px-2 py-0.5 rounded-full ${impact.color} ${impact.bgColor}`}
                                title={`Price impact: ${priceImpactPct.toFixed(2)}% (${impact.label})`}
                            >
                                {priceImpactPct.toFixed(2)}%
                            </span>
                        </div>
                    )}

                    {/* Price Details */}
                    <div className="pt-1">
                        <PriceDetailsPanel
                            quote={quote}
                            loading={quoteLoading && !!parsedAmount}
                            slippage={slippage}
                        />
                    </div>

                    {/* Submit */}
                    <div className="pt-2">
                        <SwapSubmitButton
                            state={buttonState}
                            onClick={handleSwapClick}
                            priceImpactPct={priceImpactPct}
                        />
                    </div>
                </div>
            </div>

            {/* Modals */}
            <TokenSelectorModal
                isOpen={selectorOpen !== null}
                onClose={() => setSelectorOpen(null)}
                onSelect={handleTokenSelect}
                pairs={pairs ?? []}
                loading={pairsLoading}
                side={selectorOpen === 'from' ? 'base' : 'counter'}
            />

            <SettingsModal
                isOpen={settingsOpen}
                onClose={() => setSettingsOpen(false)}
                slippage={slippage}
                onSlippageChange={setSlippage}
                deadline={deadline}
                onDeadlineChange={setDeadline}
                onReset={resetSettings}
                slippageWarning={slippageWarning}
                expectedOutput={toAmount}
                outputTokenCode={toAsset.code}
            />

            <HighImpactModal
                isOpen={highImpactOpen}
                onClose={() => setHighImpactOpen(false)}
                onConfirm={executeSwap}
                priceImpactPct={priceImpactPct}
                fromAmount={fromAmount}
                fromToken={fromAsset.code}
                toAmount={toAmount}
                toToken={toAsset.code}
            />
        </div>
    );
}
