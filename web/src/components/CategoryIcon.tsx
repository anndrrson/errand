"use client";

import {
  Search,
  PenTool,
  BarChart3,
  Coins,
  Eye,
  type LucideIcon,
} from "lucide-react";
import type { TaskCategory } from "@/lib/types";

const categoryConfig: Record<
  TaskCategory,
  { icon: LucideIcon; label: string; color: string }
> = {
  research: {
    icon: Search,
    label: "Research",
    color: "text-brand-light",
  },
  content: {
    icon: PenTool,
    label: "Content",
    color: "text-success",
  },
  data: {
    icon: BarChart3,
    label: "Data",
    color: "text-warning",
  },
  crypto: {
    icon: Coins,
    label: "Crypto",
    color: "text-error",
  },
  monitor: {
    icon: Eye,
    label: "Monitor",
    color: "text-purple",
  },
};

export function CategoryIcon({
  category,
  size = 20,
  showLabel = false,
}: {
  category: TaskCategory;
  size?: number;
  showLabel?: boolean;
}) {
  const config = categoryConfig[category];
  const Icon = config.icon;

  return (
    <span className={`inline-flex items-center gap-1.5 ${config.color}`}>
      <Icon size={size} />
      {showLabel && (
        <span className="text-sm font-medium">{config.label}</span>
      )}
    </span>
  );
}

export function CategoryBadge({ category }: { category: TaskCategory }) {
  const config = categoryConfig[category];
  const Icon = config.icon;

  return (
    <span
      className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium bg-surface-overlay border border-border ${config.color}`}
    >
      <Icon size={12} />
      {config.label}
    </span>
  );
}

export function getCategoryLabel(category: TaskCategory): string {
  return categoryConfig[category].label;
}

export { categoryConfig };
