import { forwardRef } from "react";

type TextareaProps = {
  error?: boolean;
} & React.TextareaHTMLAttributes<HTMLTextAreaElement>;

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  function Textarea({ error, className = "", ...rest }, ref) {
    return (
      <textarea
        ref={ref}
        className={`w-full px-4 py-3 rounded-lg border bg-surface text-sm text-text-primary placeholder:text-text-muted focus:outline-none transition-colors resize-y leading-relaxed disabled:opacity-50 disabled:cursor-not-allowed ${
          error
            ? "border-error focus:border-error"
            : "border-border focus:border-brand"
        } ${className}`}
        {...rest}
      />
    );
  }
);
