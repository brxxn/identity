import api from '../client';
import type { ApiResult } from '../client';

import { clearTokensSync } from '../../store/auth';

export interface User {
  id: number;
  email: string;
  username: string;
  name: string;
  is_suspended: boolean;
  credential_uuid: string;
  is_admin: boolean;
}

/**
 * Fetches the current user's information.
 * @returns The current user data or an error.
 */
export async function getCurrentUser(): Promise<ApiResult<User>> {
  return api.get<User>('/user');
}

export async function logout(): Promise<void> {
  clearTokensSync();
}

import type { Group } from './admin-groups';

export interface GetUserGroupsResponse {
  user: User;
  groups: Group[];
}

/**
 * Fetches the groups the current user is a member of.
 */
export async function getUserGroups(): Promise<ApiResult<GetUserGroupsResponse>> {
  return api.get<GetUserGroupsResponse>('/user/groups');
}
