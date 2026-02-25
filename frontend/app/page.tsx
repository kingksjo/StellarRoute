import { SwapInterface } from '@/components/swap/SwapInterface';

export default function Home() {
  return (
    <main className="min-h-screen relative overflow-hidden">
      {/* Background gradient effects */}
      <div className="fixed inset-0 pointer-events-none">
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[600px] bg-[#6366f1]/[0.04] rounded-full blur-[120px]" />
        <div className="absolute bottom-0 right-0 w-[600px] h-[400px] bg-[#818cf8]/[0.03] rounded-full blur-[100px]" />
      </div>

      {/* Main content */}
      <div className="relative z-10 flex flex-col items-center justify-center px-4 pt-8 sm:pt-16 pb-16">
        {/* Hero text */}
        <div className="text-center mb-8">
          <h1 className="text-3xl sm:text-4xl font-bold text-white mb-2">
            Trade tokens at the{' '}
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-[#818cf8] to-[#a5b4fc]">
              best price
            </span>
          </h1>
          <p className="text-sm text-white/40 max-w-md mx-auto">
            StellarRoute aggregates liquidity from SDEX orderbooks and Soroban AMM
            pools to find the optimal trading route.
          </p>
        </div>

        {/* Swap card */}
        <SwapInterface />
      </div>
    </main>
  );
}
