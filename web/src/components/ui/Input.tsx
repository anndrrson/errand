import { forwardRef } from "react";

type InputProps = {
  error?: boolean;
} & React.InputHTMLAttributes<HTMLInputElement>;

export const Input = forwardRef<HTMLInputElement, InputProps>(function Input(
  { error, className = "", ...rest },
  ref
) {
  return (
    <input
      ref={ref}
      className={`w-full px-4 py-2.5 rounded-lg border bg-surface text-sm text-text-primary placeholder:text-text-muted focus:outline-none transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${
        error
          ? "border-error focus:border-error"
          : "border-border focus:border-brand"
      } ${className}`}
      {...rest}
    />
  );
});
