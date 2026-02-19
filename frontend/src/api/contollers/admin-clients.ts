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

export interface ListClientsResponse {
  clients: Client[];
}

export interface CreateClientRequest {
  app_name: string;
  app_description: string;
  redirect_uris: string[];
  is_disabled: boolean;
  default_allowed: boolean;
  allow_implicit_flow: boolean;
  allow_explicit_flow: boolean;
}

export interface CreateClientResponse {
  client: Client;
  client_secret: string;
}

export interface UpdateClientRequest {
  app_name: string;
  app_description: string;
  redirect_uris: string[];
  is_disabled: boolean;
  default_allowed: boolean;
  allow_implicit_flow: boolean;
  allow_explicit_flow: boolean;
}

export interface UpdateClientResponse {
  client: Client;
}

export interface RotateClientSecretResponse {
  client: Client;
  client_secret: string;
}

/**
 * Lists all OAuth clients in the system (admin only).
 */
export async function listClients(): Promise<ApiResult<ListClientsResponse>> {
  return api.get<ListClientsResponse>('/clients');
}

/**
 * Creates a new OAuth client (admin only).
 */
export async function createClient(data: CreateClientRequest): Promise<ApiResult<CreateClientResponse>> {
  return api.post<CreateClientResponse>('/clients', data);
}

/**
 * Updates an OAuth client's information (admin only).
 */
export async function updateClient(clientId: string, data: UpdateClientRequest): Promise<ApiResult<UpdateClientResponse>> {
  return api.patch<UpdateClientResponse>(`/clients/${clientId}`, data);
}

/**
 * Rotates an OAuth client's secret (admin only).
 */
export async function rotateClientSecret(clientId: string): Promise<ApiResult<RotateClientSecretResponse>> {
  return api.post<RotateClientSecretResponse>(`/clients/${clientId}/rotate-secret`, undefined);
}

// Permissions & Roles

export interface UserPermissionOverride {
  user_id: number;
  client_id: string;
  granted: boolean;
}

export interface GroupPermissionOverride {
  group_id: number;
  client_id: string;
  granted: boolean;
  override_priority: number;
}

export interface UserAppRoleOverride {
  user_id: number;
  client_id: string;
  role: string;
  granted: boolean;
}

export interface GroupAppRoleOverride {
  group_id: number;
  client_id: string;
  role: string;
  granted: boolean;
  override_priority: number;
}

export interface GetClientDetailedResponse {
  client: Client;
  user_permission_overrides: UserPermissionOverride[];
  group_permission_overrides: GroupPermissionOverride[];
  user_role_overrides: UserAppRoleOverride[];
  group_role_overrides: GroupAppRoleOverride[];
}

export interface UpdateGroupPermissionOverridesRequest {
  group_permission_overrides: GroupPermissionOverride[];
}

export interface UpdateGroupPermissionOverridesResponse {
  client: Client;
  group_permission_overrides: GroupPermissionOverride[];
}

export interface UpdateGroupRoleOverridesRequest {
  group_role_overrides: GroupAppRoleOverride[];
}

export interface UpdateGroupRoleOverridesResponse {
  client: Client;
  group_role_overrides: GroupAppRoleOverride[];
}

export interface UpdateUserPermissionOverrideRequest {
  granted: boolean;
}

export interface UpdateUserRoleOverrideRequest {
  role: string;
  granted: boolean;
}

/**
 * Gets detailed information about a client including permissions and roles.
 */
export async function getClientDetailed(clientId: string): Promise<ApiResult<GetClientDetailedResponse>> {
  return api.get<GetClientDetailedResponse>(`/clients/${clientId}`);
}

/**
 * Updates group permission overrides for a client.
 */
export async function updateGroupPermissionOverrides(clientId: string, overrides: GroupPermissionOverride[]): Promise<ApiResult<UpdateGroupPermissionOverridesResponse>> {
  return api.patch<UpdateGroupPermissionOverridesResponse>(`/clients/${clientId}/group-overrides/permissions`, {
    group_permission_overrides: overrides
  });
}

/**
 * Updates group role overrides for a client.
 */
export async function updateGroupRoleOverrides(clientId: string, overrides: GroupAppRoleOverride[]): Promise<ApiResult<UpdateGroupRoleOverridesResponse>> {
  return api.patch<UpdateGroupRoleOverridesResponse>(`/clients/${clientId}/group-overrides/roles`, {
    group_role_overrides: overrides
  });
}

/**
 * Updates or creates a user permission override.
 */
export async function updateUserPermissionOverride(clientId: string, userId: number, granted: boolean): Promise<ApiResult<void>> {
  return api.patch<void>(`/clients/${clientId}/user-overrides/${userId}/permission`, {
    granted
  });
}

/**
 * Deletes a user permission override.
 */
export async function deleteUserPermissionOverride(clientId: string, userId: number): Promise<ApiResult<void>> {
  return api.del<void>(`/clients/${clientId}/user-overrides/${userId}/permission`);
}

/**
 * Updates or creates a user role override.
 */
export async function updateUserRoleOverride(clientId: string, userId: number, role: string, granted: boolean): Promise<ApiResult<void>> {
  return api.patch<void>(`/clients/${clientId}/user-overrides/${userId}/roles/${encodeURIComponent(role)}`, {
    granted
  });
}

/**
 * Deletes a user role override.
 */
export async function deleteUserRoleOverride(clientId: string, userId: number, role: string): Promise<ApiResult<void>> {
  return api.del<void>(`/clients/${clientId}/user-overrides/${userId}/roles/${role}`);
}
