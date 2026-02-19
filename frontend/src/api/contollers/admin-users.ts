import api from '../client';
import type { ApiResult } from '../client';

export interface User {
  id: number;
  email: string;
  username: string;
  name: string;
  is_suspended: boolean;
  is_admin: boolean;
  credential_uuid: string;
}

export interface Group {
  id: number;
  slug: string;
  name: string;
  description: string;
  is_managed: boolean;
}

export interface ListUsersResponse {
  users: User[];
}

export interface GetUserResponse {
  user: User;
  groups: Group[];
}

export interface UpdateUserRequest {
  email: string;
  username: string;
  name: string;
  is_suspended: boolean;
  is_admin: boolean;
}

export interface UpdateUserResponse {
  user: User;
}

export type CreateUserRequest = UpdateUserRequest;
export type CreateUserResponse = UpdateUserResponse;

/**
 * Lists all users in the system (admin only).
 */
export async function listUsers(): Promise<ApiResult<ListUsersResponse>> {
  return api.get<ListUsersResponse>('/users');
}

/**
 * Gets a specific user by ID with their groups (admin only).
 */
export async function getUserById(userId: number): Promise<ApiResult<GetUserResponse>> {
  return api.get<GetUserResponse>(`/users/${userId}`);
}

/**
 * Updates a user's information (admin only).
 */
export async function updateUser(userId: number, data: UpdateUserRequest): Promise<ApiResult<UpdateUserResponse>> {
  return api.patch<UpdateUserResponse>(`/users/${userId}`, data);
}

/**
 * Creates a new user (admin only).
 */
export async function createUser(data: CreateUserRequest): Promise<ApiResult<CreateUserResponse>> {
  return api.post<CreateUserResponse>('/users', data);
}

/**
 * Sends a registration link to the user's email.
 */
export async function sendRegistrationLink(userId: number): Promise<ApiResult<void>> {
  return api.post<void>(`/users/${userId}/send-registration-link`);
}
