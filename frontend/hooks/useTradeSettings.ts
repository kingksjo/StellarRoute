'use client';

import { useCallback, useEffect, useState } from 'react';

const SLIPPAGE_KEY = 'stellarroute_slippage';
const DEADLINE_KEY = 'stellarroute_deadline';

const DEFAULT_SLIPPAGE = 0.5;
const DEFAULT_DEADLINE = 30; // minutes

export interface TradeSettings {
    slippage: number;
    deadline: number;
}

export function useTradeSettings() {
    const [slippage, setSlippageState] = useState<number>(DEFAULT_SLIPPAGE);
    const [deadline, setDeadlineState] = useState<number>(DEFAULT_DEADLINE);

    // Load from localStorage on mount
    useEffect(() => {
        try {
            const savedSlippage = localStorage.getItem(SLIPPAGE_KEY);
            const savedDeadline = localStorage.getItem(DEADLINE_KEY);
            if (savedSlippage) setSlippageState(parseFloat(savedSlippage));
            if (savedDeadline) setDeadlineState(parseInt(savedDeadline, 10));
        } catch {
            // localStorage unavailable
        }
    }, []);

    const setSlippage = useCallback((value: number) => {
        setSlippageState(value);
        try {
            localStorage.setItem(SLIPPAGE_KEY, String(value));
        } catch {
            // ignore
        }
    }, []);

    const setDeadline = useCallback((value: number) => {
        setDeadlineState(value);
        try {
            localStorage.setItem(DEADLINE_KEY, String(value));
        } catch {
            // ignore
        }
    }, []);

    const reset = useCallback(() => {
        setSlippage(DEFAULT_SLIPPAGE);
        setDeadline(DEFAULT_DEADLINE);
    }, [setSlippage, setDeadline]);

    /** Warning level for the current slippage */
    const slippageWarning: 'low' | 'high' | null =
        slippage < 0.1 ? 'low' : slippage > 5 ? 'high' : null;

    return {
        slippage,
        setSlippage,
        deadline,
        setDeadline,
        reset,
        slippageWarning,
    };
}
