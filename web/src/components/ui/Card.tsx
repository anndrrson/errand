import { forwardRef } from "react";

type CardProps = {
  interactive?: boolean;
  padding?: "sm" | "md";
} & React.HTMLAttributes<HTMLDivElement>;

export const Card = forwardRef<HTMLDivElement, CardProps>(function Card(
  { interactive, padding = "md", className = "", children, ...rest },
  ref
) {
  return (
    <div
      ref={ref}
      className={`rounded-xl border border-border bg-surface-raised ${
        padding === "sm" ? "p-4" : "p-6"
      } ${
        interactive
          ? "hover:border-text-muted hover:bg-surface-overlay transition-all"
          : ""
      } ${className}`}
      {...rest}
    >
      {children}
    </div>
  );
});
