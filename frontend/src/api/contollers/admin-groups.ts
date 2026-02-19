import api from '../client';
import type { ApiResult } from '../client';
import type { User } from './admin-users';

export interface Group {
  id: number;
  slug: string;
  name: string;
  description: string;
  is_managed: boolean;
}

export interface ListGroupsResponse {
  groups: Group[];
}

export interface ListGroupMembersResponse {
  group: Group;
  members: User[];
}

export interface CreateGroupRequest {
  slug: string;
  name: string;
  description: string;
}

export interface CreateGroupResponse {
  group: Group;
}

export interface UpdateGroupRequest {
  slug: string;
  name: string;
  description: string;
}

export interface UpdateGroupResponse {
  group: Group;
}

export interface AddGroupMemberResponse {
  group: Group;
  targeted_user: User;
  members: User[];
}

export type RemoveGroupMemberResponse = AddGroupMemberResponse;

/**
 * Lists all groups in the system (admin only).
 */
export async function listGroups(): Promise<ApiResult<ListGroupsResponse>> {
  return api.get<ListGroupsResponse>('/groups');
}

/**
 * Lists all members of a specific group (admin only).
 */
export async function listGroupMembers(groupId: number): Promise<ApiResult<ListGroupMembersResponse>> {
  return api.get<ListGroupMembersResponse>(`/groups/${groupId}/members`);
}

/**
 * Creates a new group (admin only).
 */
export async function createGroup(data: CreateGroupRequest): Promise<ApiResult<CreateGroupResponse>> {
  return api.post<CreateGroupResponse>('/groups', data);
}

/**
 * Updates a group's information (admin only).
 */
export async function updateGroup(groupId: number, data: UpdateGroupRequest): Promise<ApiResult<UpdateGroupResponse>> {
  return api.patch<UpdateGroupResponse>(`/groups/${groupId}`, data);
}

/**
 * Adds a user to a group (admin only).
 */
export async function addGroupMember(groupId: number, userId: number): Promise<ApiResult<AddGroupMemberResponse>> {
  return api.put<AddGroupMemberResponse>(`/groups/${groupId}/members/${userId}`, undefined);
}

/**
 * Removes a user from a group (admin only).
 */
export async function removeGroupMember(groupId: number, userId: number): Promise<ApiResult<RemoveGroupMemberResponse>> {
  return api.del<RemoveGroupMemberResponse>(`/groups/${groupId}/members/${userId}`);
}
