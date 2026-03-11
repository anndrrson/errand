import { forwardRef } from "react";
import Link from "next/link";
import { Loader2 } from "lucide-react";

const variants = {
  primary:
    "bg-brand text-white hover:bg-brand-dark focus-visible:outline-brand",
  secondary:
    "border border-border text-text-secondary hover:border-text-muted hover:text-text-primary focus-visible:outline-brand",
  danger:
    "border border-error/30 text-error hover:bg-error/10 focus-visible:outline-error",
  success:
    "bg-success text-white hover:bg-success/90 focus-visible:outline-success",
  ghost:
    "text-text-muted hover:text-text-secondary focus-visible:outline-brand",
} as const;

const sizes = {
  sm: "px-3 py-1.5 text-xs gap-1.5",
  md: "px-4 py-2 text-sm gap-2",
  lg: "px-5 py-2.5 text-sm gap-2",
} as const;

type ButtonProps = {
  variant?: keyof typeof variants;
  size?: keyof typeof sizes;
  loading?: boolean;
  href?: string;
} & React.ButtonHTMLAttributes<HTMLButtonElement>;

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  function Button(
    {
      variant = "primary",
      size = "md",
      loading = false,
      href,
      disabled,
      children,
      className = "",
      ...rest
    },
    ref
  ) {
    const base =
      "inline-flex items-center justify-center font-medium rounded-lg transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 disabled:opacity-50 disabled:cursor-not-allowed";
    const cls = `${base} ${variants[variant]} ${sizes[size]} ${className}`;

    if (href && !disabled) {
      return (
        <Link href={href} className={cls}>
          {children}
        </Link>
      );
    }

    return (
      <button
        ref={ref}
        disabled={disabled || loading}
        className={cls}
        {...rest}
      >
        {loading ? <Loader2 size={16} className="animate-spin" /> : children}
      </button>
    );
  }
);
