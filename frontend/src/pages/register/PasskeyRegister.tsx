import { useState } from 'react';
import { Redirect, useLocation, useSearchParams } from 'wouter';
import { Key, ShieldCheck, Loader2 } from 'lucide-react';
import type { RegistrationTokenClaims } from '../../types/token/RegistrationToken';
import { jwtDecode } from 'jwt-decode';
import { startPasskeyRegistration, finishPasskeyRegistration } from '../../api/contollers/register';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

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

function PasskeyRegister() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [queryParams, _] = useSearchParams();
  const [, navigate] = useLocation();

  const registrationToken = queryParams.get("t");
  if (!registrationToken) {
    return (
      <Redirect to="/auth/login/passkey?next=/account"></Redirect>
    );
  }
  let claims: RegistrationTokenClaims = {
    email: '',
    exp: 0,
    username: '',
    user_id: 0
  };
  try {
    claims = jwtDecode<RegistrationTokenClaims>(registrationToken);
  } catch (ex) {
    console.error(ex);
    return (
      <Redirect to="/auth/login/passkey?next=/account"></Redirect>
    );
  }

  const handleRegister = async () => {
    setLoading(true);
    setError(null);

    // 1. Get challenge from server
    const startResult = await startPasskeyRegistration({ registration_token: registrationToken });
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
      pubKey.user.id = base64urlToBuffer(pubKey.user.id as string);
      if (pubKey.excludeCredentials) {
        for (const c of pubKey.excludeCredentials) {
          c.id = base64urlToBuffer(c.id as string);
        }
      }

      // 3. Ask browser to create credential
      const credential = await navigator.credentials.create({ publicKey: pubKey as PublicKeyCredentialCreationOptions }) as PublicKeyCredential | null;
      if (!credential) {
        setError('Credential creation was cancelled.');
        setLoading(false);
        return;
      }

      const attestation = credential.response as AuthenticatorAttestationResponse;

      // 4. Serialize credential for server
      const pk_credential = {
        id: credential.id,
        rawId: bufferToBase64url(credential.rawId),
        response: {
          attestationObject: bufferToBase64url(attestation.attestationObject),
          clientDataJSON: bufferToBase64url(attestation.clientDataJSON),
          ...(attestation.getTransports ? { transports: attestation.getTransports() } : {}),
        },
        type: credential.type,
      };

      // 5. Finalize registration with server
      const finalizeResult = await finishPasskeyRegistration({
        registration_token: registrationToken,
        challenge_signature,
        pk_credential,
      });

      if (!finalizeResult.success) {
        setError(finalizeResult.message);
        setLoading(false);
        return;
      }

      // 6. Success
      navigate('/auth/login/passkey?registered=true');
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'An unknown error occurred.';
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
          <CardTitle className="text-2xl font-bold">Register your account</CardTitle>
          <CardDescription>
            To finish setting up your account with email <strong>{claims.email}</strong>, you will need to add a passkey.
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
            onClick={handleRegister}
            disabled={loading}
          >
            {loading ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <ShieldCheck className="mr-2 h-4 w-4" />
            )}
            Create passkey
          </Button>
        </CardContent>
      </Card>
    </div>
  )
}

export default PasskeyRegister