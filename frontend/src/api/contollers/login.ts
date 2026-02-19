import api from '../client';
import type { ApiResult } from '../client';

export interface AuthenticationChallengePublicKey {
  challenge: string | ArrayBuffer;
  timeout?: number;
  rpId?: string;
  allowCredentials?: {
    type: string;
    id: string | ArrayBuffer;
    transports?: string[];
  }[];
  userVerification?: string;
  extensions?: Record<string, unknown>;
}

export interface LoginPublicKeyCredential {
  id: string;
  rawId: string;
  response: {
    authenticatorData: string;
    clientDataJSON: string;
    signature: string;
    userHandle?: string;
  };
  type: string;
  authenticatorAttachment?: string;
  extensions?: Record<string, unknown>;
}

export interface LoginInitiateResponse {
  challenge_signature: string;
  challenge_response: { publicKey: AuthenticationChallengePublicKey };
}

export interface LoginFinalizeRequest {
  challenge_signature: string;
  pk_credential: LoginPublicKeyCredential;
}

export interface User {
  id: number;
  email: string;
  username: string;
  name: string;
  is_suspended: boolean;
  credential_uuid: string;
}

export interface WebauthnCredential {
  id: number;
  name: string;
  credential_uuid: string;
  credential_id: string;
  serialized_passkey: string;
}

export interface LoginFinalizeResponse {
  access_token: string;
  refresh_token: string;
  credential: WebauthnCredential;
  user: User;
}

export interface RefreshTokenRequest {
  refresh_token: string;
}

export interface RefreshTokenResponse {
  refresh_token: string;
  access_token: string;
}

/**
 * Calls the backend to start the passkey login process.
 * This will generate a challenge that the browser's WebAuthn API will use.
 * @returns The challenge and a signature from the server.
 */
export async function startPasskeyLogin(): Promise<ApiResult<LoginInitiateResponse>> {
  return api.post<LoginInitiateResponse>('/auth/login/passkey/initiate');
}

/**
 * Calls the backend to finalize the passkey login process.
 * This sends the browser's response to the challenge back to the server for verification.
 * @param req The signed challenge and the public key credential.
 * @returns The access token, refresh token, credential, and user information.
 */
export async function finishPasskeyLogin(req: LoginFinalizeRequest): Promise<ApiResult<LoginFinalizeResponse>> {
  return api.post<LoginFinalizeResponse>('/auth/login/passkey/finalize', req);
}

export async function refreshAuthToken(req: RefreshTokenRequest): Promise<ApiResult<RefreshTokenResponse>> {
  return api.post<RefreshTokenResponse>('/auth/refresh', req);
}