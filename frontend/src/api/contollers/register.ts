import api from '../client';
import type { ApiResult } from '../client';

export interface RegisterPublicKeyCredential {
  id: string;                    // Base64url-encoded credential ID
  rawId: string;                 // Base64url-encoded raw credential ID
  response: {
    attestationObject: string;   // Base64url-encoded attestation object
    clientDataJSON: string;      // Base64url-encoded client data JSON
    transports?: string[];       // Optional transport hints
  };
  type: string;                  // Always "public-key"
  extensions?: Record<string, unknown>;
}

export interface PublicKeyUser {
  id: string | ArrayBuffer;
  name: string;
  displayName: string;
}

export interface PublicKeyCredentialDescriptor {
  type: string;
  id: string | ArrayBuffer;
  transports?: string[];
}

export interface CreationChallengePublicKey {
  rp: { name: string; id?: string };
  user: PublicKeyUser;
  challenge: string | ArrayBuffer;
  pubKeyCredParams: { type: string; alg: number }[];
  timeout?: number;
  excludeCredentials?: PublicKeyCredentialDescriptor[];
  authenticatorSelection?: {
    authenticatorAttachment?: string;
    residentKey?: string;
    requireResidentKey?: boolean;
    userVerification?: string;
  };
  attestation?: string;
  extensions?: Record<string, unknown>;
}

/**
 * Corresponds to the `RegistrationInitiateRequest` struct in Rust.
 * Sent to start the passkey registration process.
 */
export interface RegistrationInitiateRequest {
  registration_token: string;
}

/**
 * Corresponds to the `RegistrationInitiateResponse` struct in Rust.
 * Received after successfully initiating passkey registration.
 */
export interface RegistrationInitiateResponse {
  challenge_signature: string;
  challenge_response: { publicKey: CreationChallengePublicKey };
}

/**
 * Corresponds to the `RegistrationFinalizeRequest` struct in Rust.
 * Sent to complete the passkey registration process.
 */
export interface RegistrationFinalizeRequest {
  challenge_signature: string;
  registration_token: string;
  pk_credential: RegisterPublicKeyCredential;
}

/**
 * Calls the backend to start the passkey registration process.
 * This will generate a challenge that the browser's WebAuthn API will use.
 * @param req The registration token needed to initiate the process.
 * @returns The challenge and a signature from the server.
 */
export async function startPasskeyRegistration(req: RegistrationInitiateRequest): Promise<ApiResult<RegistrationInitiateResponse>> {
  return api.post<RegistrationInitiateResponse>('/auth/register/passkey/initiate', req);
}

/**
 * Calls the backend to finalize the passkey registration process.
 * This sends the browser's response to the challenge back to the server for verification.
 * @param req The signed challenge, original token, and the public key credential.
 */
export async function finishPasskeyRegistration(req: RegistrationFinalizeRequest): Promise<ApiResult<void>> {
  return api.post<void>('/auth/register/passkey/finalize', req);
}
