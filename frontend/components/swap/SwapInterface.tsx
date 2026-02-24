'use client';

import React, { useCallback, useMemo, useState } from 'react';
import { ArrowDownUp, Settings } from 'lucide-react';
import { usePairs, useQuote } from '@/hooks/useApi';
import { SwapInputBox } from './SwapInputBox';
import { TokenSelectorModal } from './TokenSelectorModal';
import { PriceDetailsPanel } from './PriceDetailsPanel';
import { SwapSubmitButton, type SwapButtonState } from './SwapSubmitButton';

export function SwapInterface() {
    // ── Token selection state ───────────────────────────────────────────
    const [fromAsset, setFromAsset] = useState({ code: 'XLM', id: 'native' });
    const [toAsset, setToAsset] = useState({ code: 'USDC', id: 'USDC' });
    const [fromAmount, setFromAmount] = useState('');
    const [selectorOpen, setSelectorOpen] = useState<'from' | 'to' | null>(null);
    const [isConnected] = useState(false); // Wallet stub

    // ── Data fetching ───────────────────────────────────────────────────
    const { data: pairs, loading: pairsLoading } = usePairs();

    const parsedAmount = useMemo(() => {
        const n = parseFloat(fromAmount);
        return isNaN(n) || n <= 0 ? undefined : n;
    }, [fromAmount]);

    const {
        data: quote,
        loading: quoteLoading,
    } = useQuote(fromAsset.id, toAsset.id, parsedAmount);

    // ── Derived values ──────────────────────────────────────────────────
    const toAmount = useMemo(() => {
        if (!quote) return '';
        return quote.total;
    }, [quote]);

    // ── Button state machine ────────────────────────────────────────────
    const buttonState: SwapButtonState = useMemo(() => {
        if (!isConnected) return 'connect-wallet';
        if (!fromAmount || parseFloat(fromAmount) <= 0) return 'enter-amount';
        if (quoteLoading) return 'loading-quote';
        return 'ready';
    }, [isConnected, fromAmount, quoteLoading]);

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

    const handleSwapClick = useCallback(() => {
        if (buttonState === 'connect-wallet') {
            // TODO: Trigger wallet connection flow
            console.log('Wallet connect triggered');
            return;
        }
        if (buttonState === 'ready') {
            // TODO: Execute swap transaction
            console.log('Execute swap:', { fromAsset, toAsset, fromAmount, quote });
        }
    }, [buttonState, fromAsset, toAsset, fromAmount, quote]);

    return (
        <div className="w-full max-w-[480px] mx-auto">
            {/* Swap Card */}
            <div className="relative rounded-3xl bg-[#12121f]/80 backdrop-blur-xl border border-white/[0.06] shadow-2xl shadow-black/40 overflow-hidden">
                {/* Glow effect */}
                <div className="absolute -top-24 -left-24 w-48 h-48 bg-[#6366f1]/10 rounded-full blur-3xl pointer-events-none" />
                <div className="absolute -bottom-24 -right-24 w-48 h-48 bg-[#818cf8]/8 rounded-full blur-3xl pointer-events-none" />

                {/* Header */}
                <div className="relative flex items-center justify-between px-5 pt-5 pb-2">
                    <h2 className="text-lg font-semibold text-white">Swap</h2>
                    <button className="p-2 rounded-xl hover:bg-white/[0.06] transition-colors">
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
                            isConnected
                                ? () => setFromAmount('1234.56')
                                : undefined
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

                    {/* Price Details */}
                    <div className="pt-2">
                        <PriceDetailsPanel
                            quote={quote}
                            loading={quoteLoading && !!parsedAmount}
                        />
                    </div>

                    {/* Submit */}
                    <div className="pt-2">
                        <SwapSubmitButton
                            state={buttonState}
                            onClick={handleSwapClick}
                        />
                    </div>
                </div>
            </div>

            {/* Token Selector Modal */}
            <TokenSelectorModal
                isOpen={selectorOpen !== null}
                onClose={() => setSelectorOpen(null)}
                onSelect={handleTokenSelect}
                pairs={pairs ?? []}
                loading={pairsLoading}
                side={selectorOpen === 'from' ? 'base' : 'counter'}
            />
        </div>
    );
}
