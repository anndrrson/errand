"use client";

import { useState, useEffect } from "react";
import { Coins } from "lucide-react";
import { getCreditBalance } from "@/lib/api";

export function CreditDisplay() {
  const [balance, setBalance] = useState<number | null>(null);

  useEffect(() => {
    async function load() {
      try {
        const res = await getCreditBalance();
        setBalance(res.balance);
      } catch {
        // Show nothing if API is unavailable
      }
    }
    load();
  }, []);

  if (balance === null) return null;

  return (
    <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-surface-raised border border-border text-sm">
      <Coins size={14} className="text-warning" />
      <span className="font-medium text-text-primary">{balance}</span>
    </div>
  );
}
