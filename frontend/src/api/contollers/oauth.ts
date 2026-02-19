import api from '../client';
import type { ApiResult } from '../client';

export interface Client {
  client_id: string;
  app_name: string;
  app_description: string;
  redirect_uris: string[];
  is_managed: boolean;
  is_disabled: boolean;
  default_allowed: boolean;
  allow_implicit_flow: boolean;
  allow_explicit_flow: boolean;
}

export interface AuthorizePreviewRequest {
  client_id: string;
  redirect_uri: string;
  scope: string;
  response_type: string;
  state?: string;
  nonce?: string;
  code_challenge?: string;
  code_challenge_method?: string;
}

export interface AuthorizePreviewResponse {
  client: Client;
}

export interface AuthorizeApproveRequest {
  client_id: string;
  redirect_uri: string;
  scope: string;
  response_type: string;
  state: string;
  nonce?: string;
  code_challenge?: string;
  code_challenge_method?: string;
}

export interface AuthorizeApproveResponse {
  redirect_to: string;
}

/**
 * Fetches information about the OAuth client and authorization request for preview.
 */
export function previewAuthorize(params: AuthorizePreviewRequest): Promise<ApiResult<AuthorizePreviewResponse>> {
  return api.post<AuthorizePreviewResponse>('/oauth/authorize/preview', params);
}

/**
 * Approves or denies the OAuth authorization request.
 */
export function approveAuthorize(params: AuthorizeApproveRequest): Promise<ApiResult<AuthorizeApproveResponse>> {
  return api.post<AuthorizeApproveResponse>('/oauth/authorize/approve', params);
}
