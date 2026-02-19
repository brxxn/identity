import { Loader2 } from 'lucide-react';

interface LoadingSpinnerProps {
  className?: string;
  size?: number;
}

export default function LoadingSpinner({ className, size = 24 }: LoadingSpinnerProps) {
  return (
    <div className="flex h-full w-full items-center justify-center p-4">
      <Loader2 className={`animate-spin ${className}`} size={size} />
    </div>
  );
}
