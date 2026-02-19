import { useEffect, useState } from 'react';
import type { ReactNode } from 'react';
import { Redirect, useLocation, useSearch } from 'wouter';
import LoadingSpinner from './LoadingSpinner';
import { getAuthToken } from '../api/client';
import { getCurrentUser } from '../api/contollers/user';

interface ProtectedRouteProps {
  children: ReactNode;
}

/**
 * Wraps routes that require authentication.
 * Redirects to login with a secure 'next' parameter if user is not authenticated.
 */
export default function ProtectedRoute({ children }: ProtectedRouteProps) {
  const [isAuthenticated, setIsAuthenticated] = useState<boolean | null>(null);
  const [location] = useLocation();
  const search = useSearch();

  useEffect(() => {
    const checkAuth = async () => {
      const token = getAuthToken();
      if (!token) {
        setIsAuthenticated(false);
        return;
      }

      // Verify token is valid by checking with the API
      const result = await getCurrentUser();
      setIsAuthenticated(result.success);
    };

    checkAuth();
  }, []);

  if (isAuthenticated === null) {
    return <LoadingSpinner />;
  }

  if (!isAuthenticated) {
    const nextParam = search ? `${location}?${search}` : location;

    return <Redirect to={`/auth/login/passkey?next=${encodeURIComponent(nextParam)}`} />;
  }

  return <>{children}</>;
}
