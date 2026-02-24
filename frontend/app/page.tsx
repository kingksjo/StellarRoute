import { SwapInterface } from '@/components/swap/SwapInterface';

export default function Home() {
  return (
    <main className="min-h-screen relative overflow-hidden">
      {/* Background gradient effects */}
      <div className="fixed inset-0 pointer-events-none">
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[600px] bg-[#6366f1]/[0.04] rounded-full blur-[120px]" />
        <div className="absolute bottom-0 right-0 w-[600px] h-[400px] bg-[#818cf8]/[0.03] rounded-full blur-[100px]" />
      </div>

      {/* Header */}
      <header className="relative z-10 flex items-center justify-between px-6 py-4 max-w-7xl mx-auto">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-[#6366f1] to-[#818cf8] flex items-center justify-center">
            <span className="text-sm font-bold text-white">S</span>
          </div>
          <span className="text-lg font-bold text-white">
            Stellar<span className="text-[#818cf8]">Route</span>
          </span>
        </div>
        <nav className="hidden sm:flex items-center gap-6">
          <a href="#" className="text-sm text-white/60 hover:text-white transition-colors">
            Swap
          </a>
          <a href="#" className="text-sm text-white/40 hover:text-white/70 transition-colors">
            Pools
          </a>
          <a href="#" className="text-sm text-white/40 hover:text-white/70 transition-colors">
            Portfolio
          </a>
        </nav>
        <button className="px-4 py-2 rounded-xl bg-white/[0.06] border border-white/[0.08] text-sm font-medium text-white hover:bg-white/[0.1] transition-all">
          Connect Wallet
        </button>
      </header>

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
