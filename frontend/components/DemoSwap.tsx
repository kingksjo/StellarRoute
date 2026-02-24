"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { TransactionConfirmationModal } from "@/components/shared/TransactionConfirmationModal";
import { useTransactionHistory } from "@/hooks/useTransactionHistory";
import { TransactionStatus } from "@/types/transaction";
import { toast } from "sonner";
import { PathStep } from "@/types";

// Mock wallet address for demo purposes
const MOCK_WALLET = "GBSU...XYZ9";

export function DemoSwap() {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [txStatus, setTxStatus] = useState<TransactionStatus | "review">("review");
  const [errorMessage, setErrorMessage] = useState<string>();
  const [txHash, setTxHash] = useState<string>();

  const { addTransaction } = useTransactionHistory(MOCK_WALLET);

  // Mock Route Data
  const mockRoute: PathStep[] = [
    {
      from_asset: { asset_type: "native" },
      to_asset: { asset_type: "credit_alphanum4", asset_code: "USDC", asset_issuer: "GA5Z..." },
      price: "0.105",
      source: "sdex"
    }
  ];

  const handleSwapClick = () => {
    setTxStatus("review");
    setErrorMessage(undefined);
    setTxHash(undefined);
    setIsModalOpen(true);
  };

  const handleConfirm = () => {
    // 1. Awaiting Signature
    setTxStatus("pending");

    // Simulate user signing in wallet (2s)
    setTimeout(() => {
      // 2. Submitting to network
      setTxStatus("submitting");

      // Simulate submission (1s)
      setTimeout(() => {
        // 3. Processing on network
        setTxStatus("processing");

        // Simulate network processing (2s) and randomly succeed or fail
        setTimeout(() => {
          const isSuccess = Math.random() > 0.2; // 80% success rate for demo

          if (isSuccess) {
            const mockHash = "mock_tx_" + Math.random().toString(36).substring(7);
            setTxHash(mockHash);
            setTxStatus("success");
            toast.success("Transaction Successful!", {
              description: "You have swapped 100 XLM for 10.5 USDC",
            });
            
            // Add to history
            addTransaction({
              id: mockHash,
              timestamp: Date.now(),
              fromAsset: "XLM",
              fromAmount: "100",
              toAsset: "USDC",
              toAmount: "10.5",
              exchangeRate: "0.105",
              priceImpact: "0.1%",
              minReceived: "10.45",
              networkFee: "0.00001",
              routePath: mockRoute,
              status: "success",
              hash: mockHash,
              walletAddress: MOCK_WALLET
            });
          } else {
            setTxStatus("failed");
            setErrorMessage("Insufficient balance or network congestion. Please try again.");
            toast.error("Transaction Failed", {
              description: "Insufficient balance or network congestion.",
            });

            // Add failed tx to history
            addTransaction({
              id: "failed_" + Date.now(),
              timestamp: Date.now(),
              fromAsset: "XLM",
              fromAmount: "100",
              toAsset: "USDC",
              toAmount: "10.5",
              exchangeRate: "0.105",
              priceImpact: "0.1%",
              minReceived: "10.45",
              networkFee: "0.00001",
              routePath: mockRoute,
              status: "failed",
              errorMessage: "Insufficient balance.",
              walletAddress: MOCK_WALLET
            });
          }
        }, 2000);
      }, 1000);
    }, 2000);
  };

  const handleCancel = () => {
    setTxStatus("review");
    console.log("Transaction cancelled");
  };

  return (
    <Card className="p-6 max-w-sm mx-auto shadow-lg mt-8 border-primary/20 bg-background/50 backdrop-blur-sm">
      <div className="space-y-4">
        <div>
          <h2 className="text-xl font-bold mb-1">Swap Tokens</h2>
          <p className="text-sm text-muted-foreground">Demo swap interface</p>
        </div>
        
        <div className="space-y-4 bg-muted/20 p-4 rounded-lg border">
          <div>
            <span className="text-sm font-medium">Pay</span>
            <div className="text-2xl font-bold mt-1">100 XLM</div>
          </div>
          <div>
            <span className="text-sm font-medium">Receive</span>
            <div className="text-2xl font-bold mt-1 text-success">~10.5 USDC</div>
          </div>
        </div>

        <Button className="w-full text-lg h-12" onClick={handleSwapClick}>
          Review Swap
        </Button>
      </div>

      <TransactionConfirmationModal
        isOpen={isModalOpen}
        onOpenChange={setIsModalOpen}
        fromAsset="XLM"
        fromAmount="100"
        toAsset="USDC"
        toAmount="10.5"
        exchangeRate="0.105"
        priceImpact="0.1%"
        minReceived="10.45"
        networkFee="0.00001"
        routePath={mockRoute}
        onConfirm={handleConfirm}
        onCancel={handleCancel}
        status={txStatus}
        errorMessage={errorMessage}
        txHash={txHash}
      />
    </Card>
  );
}
