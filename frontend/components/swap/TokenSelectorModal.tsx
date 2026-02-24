'use client';

import React, { useMemo, useState } from 'react';
import { Search, X } from 'lucide-react';
import type { TradingPair } from '@/types';

interface TokenSelectorModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSelect: (assetIdentifier: string, code: string) => void;
    pairs: TradingPair[];
    loading?: boolean;
    /** Which side we're selecting for — used to extract the right asset */
    side: 'base' | 'counter';
}

const POPULAR_CODES = ['XLM', 'USDC', 'BTC', 'ETH', 'AQUA', 'yXLM'];

export function TokenSelectorModal({
    isOpen,
    onClose,
    onSelect,
    pairs,
    loading = false,
    side,
}: TokenSelectorModalProps) {
    const [search, setSearch] = useState('');

    // Deduplicate tokens from pairs
    const tokens = useMemo(() => {
        const seen = new Set<string>();
        const result: { code: string; identifier: string }[] = [];

        for (const pair of pairs) {
            const code = side === 'base' ? pair.base : pair.counter;
            const id = side === 'base' ? pair.base_asset : pair.counter_asset;
            if (!seen.has(id)) {
                seen.add(id);
                result.push({ code, identifier: id });
            }
            // Also add the other side for completeness
            const otherCode = side === 'base' ? pair.counter : pair.base;
            const otherId = side === 'base' ? pair.counter_asset : pair.base_asset;
            if (!seen.has(otherId)) {
                seen.add(otherId);
                result.push({ code: otherCode, identifier: otherId });
            }
        }

        return result;
    }, [pairs, side]);

    const popularTokens = useMemo(
        () => tokens.filter((t) => POPULAR_CODES.includes(t.code)),
        [tokens],
    );

    const filteredTokens = useMemo(() => {
        if (!search) return tokens;
        const q = search.toLowerCase();
        return tokens.filter(
            (t) =>
                t.code.toLowerCase().includes(q) ||
                t.identifier.toLowerCase().includes(q),
        );
    }, [tokens, search]);

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
            {/* Backdrop */}
            <div
                className="absolute inset-0 bg-black/60 backdrop-blur-sm"
                onClick={onClose}
            />

            {/* Modal */}
            <div className="relative w-full max-w-md mx-4 rounded-2xl bg-[#0f0f1a] border border-white/[0.08] shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-200">
                {/* Header */}
                <div className="flex items-center justify-between p-4 border-b border-white/[0.06]">
                    <h2 className="text-lg font-semibold text-white">Select a token</h2>
                    <button
                        onClick={onClose}
                        className="p-1.5 rounded-lg hover:bg-white/[0.06] transition-colors"
                    >
                        <X className="w-5 h-5 text-white/60" />
                    </button>
                </div>

                {/* Search */}
                <div className="p-4 pb-2">
                    <div className="relative">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-white/30" />
                        <input
                            type="text"
                            placeholder="Search by name or address..."
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
                            className="w-full h-10 pl-10 pr-4 rounded-xl bg-white/[0.04] border border-white/[0.08] text-sm text-white placeholder:text-white/30 outline-none focus:border-[#6366f1]/40 transition-colors"
                            autoFocus
                        />
                    </div>
                </div>

                {/* Popular tokens */}
                {!search && popularTokens.length > 0 && (
                    <div className="px-4 pb-3">
                        <span className="text-[10px] font-medium text-white/30 uppercase tracking-wider">
                            Popular
                        </span>
                        <div className="flex flex-wrap gap-2 mt-2">
                            {popularTokens.map((t) => (
                                <button
                                    key={t.identifier}
                                    onClick={() => onSelect(t.identifier, t.code)}
                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-white/[0.04] border border-white/[0.08] text-sm text-white hover:bg-white/[0.08] hover:border-white/[0.12] transition-all"
                                >
                                    <div className="w-5 h-5 rounded-full bg-gradient-to-br from-[#818cf8] to-[#6366f1] flex items-center justify-center text-[8px] font-bold text-white">
                                        {t.code.charAt(0)}
                                    </div>
                                    {t.code}
                                </button>
                            ))}
                        </div>
                    </div>
                )}

                {/* Token list */}
                <div className="max-h-72 overflow-y-auto border-t border-white/[0.04]">
                    {loading ? (
                        <div className="flex flex-col gap-1 p-2">
                            {[...Array(5)].map((_, i) => (
                                <div
                                    key={i}
                                    className="h-12 rounded-xl bg-white/[0.03] animate-pulse"
                                />
                            ))}
                        </div>
                    ) : filteredTokens.length === 0 ? (
                        <div className="py-12 text-center text-sm text-white/30">
                            No tokens found
                        </div>
                    ) : (
                        <div className="p-2">
                            {filteredTokens.map((t) => (
                                <button
                                    key={t.identifier}
                                    onClick={() => onSelect(t.identifier, t.code)}
                                    className="w-full flex items-center gap-3 px-3 py-3 rounded-xl hover:bg-white/[0.04] transition-colors"
                                >
                                    <div className="w-8 h-8 rounded-full bg-gradient-to-br from-[#818cf8] to-[#6366f1] flex items-center justify-center text-xs font-bold text-white shrink-0">
                                        {t.code.charAt(0)}
                                    </div>
                                    <div className="flex flex-col items-start min-w-0">
                                        <span className="text-sm font-semibold text-white">
                                            {t.code}
                                        </span>
                                        <span className="text-[11px] text-white/30 truncate max-w-[200px]">
                                            {t.identifier === 'native'
                                                ? 'Stellar Lumens'
                                                : t.identifier}
                                        </span>
                                    </div>
                                </button>
                            ))}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
