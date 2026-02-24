'use client';

/**
 * Trade calculation utilities for price impact analysis and slippage.
 */

export type ImpactSeverity = 'low' | 'moderate' | 'high' | 'critical';

export interface ImpactInfo {
  severity: ImpactSeverity;
  color: string;
  bgColor: string;
  label: string;
}

/**
 * Returns a severity object based on the price impact percentage.
 *
 * Green  (< 1%):   Low — safe to proceed
 * Yellow (1%–3%):  Moderate — proceed with caution
 * Orange (3%–5%):  High — warning displayed
 * Red    (> 5%):   Critical — strong warning with confirmation step
 */
export function getPriceImpactSeverity(impactPct: number): ImpactInfo {
  if (impactPct < 1) {
    return {
      severity: 'low',
      color: 'text-emerald-400',
      bgColor: 'bg-emerald-400/10',
      label: 'Low',
    };
  }
  if (impactPct < 3) {
    return {
      severity: 'moderate',
      color: 'text-yellow-400',
      bgColor: 'bg-yellow-400/10',
      label: 'Moderate',
    };
  }
  if (impactPct < 5) {
    return {
      severity: 'high',
      color: 'text-orange-400',
      bgColor: 'bg-orange-400/10',
      label: 'High',
    };
  }
  return {
    severity: 'critical',
    color: 'text-red-400',
    bgColor: 'bg-red-400/10',
    label: 'Very High',
  };
}

/**
 * Calculate the minimum amount received given slippage tolerance.
 * @param amount    The expected output amount (as a string or number)
 * @param slippage  Slippage percentage, e.g. 0.5 for 0.5%
 */
export function calculateMinimumReceived(
  amount: number | string,
  slippage: number,
): string {
  const num = typeof amount === 'string' ? parseFloat(amount) : amount;
  if (isNaN(num) || num <= 0) return '0';
  const min = num * (1 - slippage / 100);
  return min.toFixed(7);
}

/**
 * Format a number for display (truncating excessive decimals).
 */
export function formatAmount(value: number | string, decimals = 7): string {
  const num = typeof value === 'string' ? parseFloat(value) : value;
  if (isNaN(num)) return '0';
  if (num === 0) return '0';
  if (Math.abs(num) < 0.0000001) return '< 0.0000001';
  return num.toFixed(decimals).replace(/\.?0+$/, '');
}

/**
 * Extract a human-readable asset code from an asset identifier.
 * e.g. "native" → "XLM", "USDC:GA5Z..." → "USDC"
 */
export function assetCode(identifier: string): string {
  if (identifier === 'native') return 'XLM';
  return identifier.split(':')[0] || identifier;
}
