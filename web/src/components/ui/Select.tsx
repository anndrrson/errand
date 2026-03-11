import { forwardRef } from "react";
import { ChevronDown } from "lucide-react";

type SelectProps = {
  error?: boolean;
  fullWidth?: boolean;
} & React.SelectHTMLAttributes<HTMLSelectElement>;

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  function Select({ error, fullWidth, className = "", children, ...rest }, ref) {
    return (
      <div className={`relative ${fullWidth ? "w-full" : "inline-block"}`}>
        <select
          ref={ref}
          className={`${
            fullWidth ? "w-full" : ""
          } appearance-none cursor-pointer px-4 py-2.5 pr-9 rounded-lg border bg-surface text-sm text-text-primary focus:outline-none transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${
            error
              ? "border-error focus:border-error"
              : "border-border focus:border-brand hover:border-text-muted"
          } ${className}`}
          {...rest}
        >
          {children}
        </select>
        <ChevronDown
          size={14}
          className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-text-muted"
        />
      </div>
    );
  }
);
