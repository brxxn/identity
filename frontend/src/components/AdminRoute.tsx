import { useEffect, useState } from 'react';
import type { ReactNode } from 'react';
import { Redirect, useLocation, useSearch } from 'wouter';
import { jwtDecode } from 'jwt-decode';
import LoadingSpinner from './LoadingSpinner';
import { getAuthToken } from '../api/client';
import { getCurrentUser } from '../api/contollers/user';

interface AdminRouteProps {
  children: ReactNode;
}

interface JwtClaims {
  sub: string;
  exp: number;
  is_admin?: boolean;
}

/**
 * Wraps routes that require admin privileges.
 * Redirects to login if not authenticated, or to profile if not admin.
 */
export default function AdminRoute({ children }: AdminRouteProps) {
  const [isAdmin, setIsAdmin] = useState<boolean | null>(null);
  const [location] = useLocation();
  const search = useSearch();

  useEffect(() => {
    const checkAdmin = async () => {
      const token = getAuthToken();
      if (!token) {
        setIsAdmin(false);
        return;
      }

      try {
        // Decode token to check is_admin claim
        const claims = jwtDecode<JwtClaims>(token);
        
        // Verify token is still valid with API
        const result = await getCurrentUser();
        if (!result.success) {
          setIsAdmin(false);
          return;
        }

        setIsAdmin(claims.is_admin === true);
      } catch (error) {
        setIsAdmin(false);
      }
    };

    checkAdmin();
  }, []);

  if (isAdmin === null) {
    return <LoadingSpinner />;
  }

  if (isAdmin === false) {
    const token = getAuthToken();
    if (!token) {
      // Not authenticated - redirect to login
      const nextPath = location.startsWith('/') ? location : '/admin/users';
      const nextParam = search ? `${nextPath}${search}` : nextPath;
      return <Redirect to={`/auth/login/passkey?next=${encodeURIComponent(nextParam)}`} />;
    } else {
      // Authenticated but not admin - redirect to profile
      return <Redirect to="/account/profile" />;
    }
  }

  return <>{children}</>;
}
