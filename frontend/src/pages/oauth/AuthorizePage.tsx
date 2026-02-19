import { useEffect, useState } from 'react';
import { useSearch } from 'wouter';
import { ShieldAlert, Check, Loader2, ShieldOff } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import { previewAuthorize, approveAuthorize } from '../../api/contollers/oauth';
import type { Client, AuthorizeApproveRequest } from '../../api/contollers/oauth';

function AuthorizePage() {
  const searchParams = useSearch();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [client, setClient] = useState<Client | null>(null);
  const [approving, setApproving] = useState(false);
  const [isDenied, setIsDenied] = useState(false);

  // Extract params
  const params = new URLSearchParams(searchParams);
  const client_id = params.get('client_id');
  const redirect_uri = params.get('redirect_uri');
  const scope = params.get('scope') || 'openid profile email';
  const response_type = params.get('response_type');
  const state = params.get('state') || undefined;
  const nonce = params.get('nonce') || undefined;
  const code_challenge = params.get('code_challenge') || undefined;
  const code_challenge_method = params.get('code_challenge_method') || undefined;

  useEffect(() => {
    async function loadPreview() {
      if (!client_id || !redirect_uri || !response_type) {
        setError('Missing required parameters');
        setLoading(false);
        return;
      }

      setLoading(true);
      const result = await previewAuthorize({
        client_id,
        redirect_uri,
        scope: scope || '',
        response_type,
        state,
        nonce,
        code_challenge,
        code_challenge_method,
      });

      if (result.success) {
        setClient(result.data.client);
      } else {
        setError(`${result.message} (${result.code})`);
      }
      setLoading(false);
    }

    loadPreview();
  }, [client_id, redirect_uri, scope, response_type, state, nonce, code_challenge, code_challenge_method]);

  const handleDecision = async (decision: 'allow' | 'deny') => {
    if (!client_id || !redirect_uri || !response_type || !state) return;

    if (decision === 'deny') {
      setIsDenied(true);
      return;
    }

    setApproving(true);

    const req: AuthorizeApproveRequest = {
      client_id,
      redirect_uri,
      scope: scope || '',
      response_type,
      state,
      nonce,
      code_challenge,
      code_challenge_method,
    };

    try {
      const result = await approveAuthorize(req);
      if (result.success) {
        let redirectUrl = result.data.redirect_to;

        // Safety check verification
        try {
          const url = new URL(redirectUrl);
          const ILLEGAL_PROTOCOLS = ['javascript:', 'data:', 'blob:', 'file:', 'about:'];
          if (ILLEGAL_PROTOCOLS.includes(url.protocol)) {
            throw new Error('Invalid protocol');
          }
          window.location.href = redirectUrl;
        } catch (e) {
          setError('The server returned an invalid redirect URL.');
          setApproving(false);
        }

      } else {
        setError(`${result.message} (${result.code})`);
        setApproving(false);
      }
    } catch (err) {
      setError('An unexpected error occurred. Please try again.');
      setApproving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-muted/40 p-4">
        <Loader2 className="h-10 w-10 animate-spin text-primary" />
      </div>
    );
  }

  if (isDenied) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-muted/40 p-4">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center">
            <div className="flex justify-center mb-4">
              <div className="rounded-full bg-muted p-4">
                <ShieldOff className="h-8 w-8 text-muted-foreground" />
              </div>
            </div>
            <CardTitle className="text-xl">Authorization Cancelled</CardTitle>
            <CardDescription>
              You may now close this window.
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-muted/40 p-4">
        <Card className="w-full max-w-md border-destructive/50">
          <CardHeader className="text-center">
            <div className="flex justify-center mb-4">
              <div className="rounded-full bg-destructive/10 p-4">
                <ShieldAlert className="h-8 w-8 text-destructive" />
              </div>
            </div>
            <CardTitle className="text-xl text-destructive">Authorization Error</CardTitle>
            <CardDescription>{error}</CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  if (!client) return null; // Should not happen if loading is false and error is null

  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/40 p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center space-y-4">
          <div className="flex justify-center">
            <Avatar className="h-20 w-20">
              <AvatarFallback className="text-xl bg-primary/10 text-primary">
                {client.app_name.substring(0, 2).toUpperCase()}
              </AvatarFallback>
            </Avatar>
          </div>
          <div>
            <CardTitle className="text-2xl">{client.app_name}</CardTitle>
          </div>
          <CardDescription className="text-base">
            wants access to your account
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-6">
          <div className="space-y-3">
            <h4 className="font-medium text-sm text-muted-foreground uppercase tracking-wider">Requested Permissions</h4>
            <ul className="space-y-2">
              <li className="flex items-start gap-2 text-sm">
                <Check className="h-4 w-4 text-green-500 mt-0.5 shrink-0" />
                <span>View your name, username, and email address</span>
              </li>
            </ul>
          </div>

          <div className="rounded-lg bg-muted p-3 text-xs text-muted-foreground">
            Make sure you trust <strong>{client.app_name}</strong>. You may be sharing sensitive information with this site.
          </div>
        </CardContent>

        <CardFooter className="flex gap-3">
          <Button
            variant="outline"
            className="w-full"
            onClick={() => handleDecision('deny')}
            disabled={approving}
          >
            Deny
          </Button>
          <Button
            className="w-full"
            onClick={() => handleDecision('allow')}
            disabled={approving}
          >
            {approving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Authorize
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}

export default AuthorizePage;
