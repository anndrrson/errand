"use client";

import { Zap, Calendar, Eye, GitBranch } from "lucide-react";
import type { TaskKind } from "@/lib/types";

const typeConfig: Record<
  string,
  { icon: typeof Zap; color: string }
> = {
  one_shot: { icon: Zap, color: "text-brand-light" },
  recurring: { icon: Calendar, color: "text-success" },
  monitor: { icon: Eye, color: "text-purple" },
  pipeline: { icon: GitBranch, color: "text-warning" },
};

export function TaskTypeIcon({
  kind,
  size = 16,
}: {
  kind: TaskKind;
  size?: number;
}) {
  const config = typeConfig[kind.type];
  const Icon = config.icon;

  return <Icon size={size} className={config.color} />;
}
