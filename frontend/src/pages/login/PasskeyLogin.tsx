import { useState } from 'react';
import { useLocation, useSearch } from 'wouter';
import { Key, ShieldCheck, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { setAuthToken } from '../../api/client';
import { startPasskeyLogin, finishPasskeyLogin } from '../../api/contollers/login';

function base64urlToBuffer(s: string): ArrayBuffer {
  const padding = '='.repeat((4 - (s.length % 4)) % 4);
  const base64 = (s + padding).replace(/-/g, '+').replace(/_/g, '/');
  const raw = window.atob(base64);
  const out = new Uint8Array(raw.length);
  for (let i = 0; i < raw.length; i++) out[i] = raw.charCodeAt(i);
  return out.buffer;
}

function bufferToBase64url(buf: ArrayBuffer): string {
  const bytes = new Uint8Array(buf);
  let s = '';
  for (let i = 0; i < bytes.byteLength; i++) s += String.fromCharCode(bytes[i]);
  return window.btoa(s).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

function PasskeyLogin() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [, navigate] = useLocation();
  const searchParams = useSearch();

  const handlePasskeyLogin = async () => {
    setLoading(true);
    setError(null);

    // 1. Get challenge from server
    const startResult = await startPasskeyLogin();
    if (!startResult.success) {
      setError(startResult.message);
      setLoading(false);
      return;
    }

    const { challenge_signature, challenge_response } = startResult.data;

    try {
      // 2. Convert challenge fields for browser WebAuthn API
      const pubKey = challenge_response.publicKey;
      pubKey.challenge = base64urlToBuffer(pubKey.challenge as string);
      if (pubKey.allowCredentials) {
        for (const c of pubKey.allowCredentials) {
          c.id = base64urlToBuffer(c.id as string);
        }
      }

      // 3. Ask browser to authenticate with credential
      const credential = await navigator.credentials.get({ publicKey: pubKey as PublicKeyCredentialRequestOptions }) as PublicKeyCredential | null;
      if (!credential) {
        setError('Authentication was cancelled.');
        setLoading(false);
        return;
      }

      const assertion = credential.response as AuthenticatorAssertionResponse;

      // 4. Serialize credential for server
      const pk_credential = {
        id: credential.id,
        rawId: bufferToBase64url(credential.rawId),
        response: {
          authenticatorData: bufferToBase64url(assertion.authenticatorData),
          clientDataJSON: bufferToBase64url(assertion.clientDataJSON),
          signature: bufferToBase64url(assertion.signature),
          ...(assertion.userHandle ? { userHandle: bufferToBase64url(assertion.userHandle) } : {}),
        },
        type: credential.type,
        ...(credential.authenticatorAttachment ? { authenticatorAttachment: credential.authenticatorAttachment } : {}),
      };

      // 5. Finalize login with server
      const finalizeResult = await finishPasskeyLogin({
        challenge_signature,
        pk_credential,
      });

      if (!finalizeResult.success) {
        setError(finalizeResult.message);
        setLoading(false);
        return;
      }

      // 6. Store tokens and redirect
      setAuthToken(finalizeResult.data.access_token, finalizeResult.data.refresh_token);

      // Handle secure redirect via 'next' parameter
      const params = new URLSearchParams(searchParams);
      const nextPath = params.get('next');

      if (nextPath) {
        try {
          const url = new URL(nextPath, window.location.origin);
          if (url.origin === window.location.origin) {
            navigate(url.pathname + url.search + url.hash);
            return;
          }
        } catch (e) {
          // Ignore invalid URLs
        }
      }

      navigate('/account/profile');
    } catch (err: unknown) {
      let message = err instanceof Error ? `Sorry, it looks like we encountered an error while doing that. Please try again! Technical details: ${err.message}` : 'An unknown error occurred.';
      if (err instanceof Error) {
        if (err.name === 'NotAllowedError') {
          message = 'Please select a passkey and try again.';
        }
      }
      setError(message);
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/40 p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="space-y-1 text-center">
          <div className="flex justify-center mb-4">
            <div className="rounded-full bg-primary/10 p-4">
              <Key className="h-8 w-8 text-primary" />
            </div>
          </div>
          <CardTitle className="text-2xl font-bold">Sign In</CardTitle>
          <CardDescription>
            To continue, you will need to authorize this session with your passkey.
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4">
          {error && (
            <div className="bg-destructive/15 text-destructive text-sm p-3 rounded-md">
              {error}
            </div>
          )}
          <Button
            className="w-full"
            onClick={handlePasskeyLogin}
            disabled={loading}
          >
            {loading ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <ShieldCheck className="mr-2 h-4 w-4" />
            )}
            Use passkey
          </Button>
        </CardContent>
        <CardFooter className="flex flex-col gap-2">
          <div className="text-xs text-muted-foreground text-center">
            Don't have an account? Ask your administrator for an invite link.
          </div>
        </CardFooter>
      </Card>
    </div>
  );
}

export default PasskeyLogin