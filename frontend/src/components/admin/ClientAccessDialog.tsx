import { useState, useEffect, useRef } from 'react';
import {
  Loader2,
  Trash2,
  GripVertical,
  Plus,
  X
} from 'lucide-react';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core';
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
  useSortable
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogDescription,
} from '@/components/ui/dialog';
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/components/ui/tabs';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';

import {
  getClientDetailed,
  updateGroupPermissionOverrides,
  updateGroupRoleOverrides,
  updateUserPermissionOverride,
  deleteUserPermissionOverride,
  updateUserRoleOverride,
  deleteUserRoleOverride,
  type Client,
  type UserPermissionOverride,
  type GroupPermissionOverride,
  type UserAppRoleOverride,
  type GroupAppRoleOverride,
} from '../../api/contollers/admin-clients';
import { listGroups, type Group } from '../../api/contollers/admin-groups';
import { listUsers, type User } from '../../api/contollers/admin-users';

interface ClientAccessDialogProps {
  client: Client | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

// Sortable Item Component
function SortableItem(props: { id: string; children: React.ReactNode }) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
  } = useSortable({ id: props.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  return (
    <div ref={setNodeRef} style={style} {...attributes} className="mb-2">
      <div className="flex items-start gap-2 rounded-md border p-3 bg-card text-card-foreground shadow-sm">
        <div {...listeners} className="cursor-grab text-muted-foreground hover:text-foreground mt-1">
          <GripVertical className="h-5 w-5" />
        </div>
        <div className="flex-1 min-w-0">
          {props.children}
        </div>
      </div>
    </div>
  );
}


export default function ClientAccessDialog({ client, open, onOpenChange }: ClientAccessDialogProps) {
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Reference Data
  const [allGroups, setAllGroups] = useState<Group[]>([]);
  const [allUsers, setAllUsers] = useState<User[]>([]);

  // State
  const [userPermissions, setUserPermissions] = useState<UserPermissionOverride[]>([]);
  const [groupPermissions, setGroupPermissions] = useState<GroupPermissionOverride[]>([]);
  const [userRoles, setUserRoles] = useState<UserAppRoleOverride[]>([]);
  const [groupRoles, setGroupRoles] = useState<GroupAppRoleOverride[]>([]);

  // Initial State for Diffing
  const initialUserPermissions = useRef<UserPermissionOverride[]>([]);
  const initialUserRoles = useRef<UserAppRoleOverride[]>([]);

  // UI State
  const [permissionUserSearch, setPermissionUserSearch] = useState('');
  const [roleUserSearch, setRoleUserSearch] = useState('');

  // Sensors for DnD
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  useEffect(() => {
    if (open && client) {
      loadData();
    } else {
      // Reset state on close
      setGroupPermissions([]);
      setUserPermissions([]);
      setGroupRoles([]);
      setUserRoles([]);
      initialUserPermissions.current = [];
      initialUserRoles.current = [];
      setError(null);
    }
  }, [open, client]);

  const loadData = async () => {
    if (!client) return;
    setLoading(true);
    setError(null);

    try {
      const [clientDataRes, groupsRes, usersRes] = await Promise.all([
        getClientDetailed(client.client_id),
        listGroups(),
        listUsers()
      ]);

      if (!clientDataRes.success) throw new Error(clientDataRes.message);
      if (!groupsRes.success) throw new Error(groupsRes.message);
      if (!usersRes.success) throw new Error(usersRes.message);

      setAllGroups(groupsRes.data.groups);
      setAllUsers(usersRes.data.users);

      // Initialize State
      const gp = clientDataRes.data.group_permission_overrides.sort((a, b) => a.override_priority - b.override_priority);
      const up = clientDataRes.data.user_permission_overrides;
      const gr = clientDataRes.data.group_role_overrides.sort((a, b) => a.override_priority - b.override_priority);
      const ur = clientDataRes.data.user_role_overrides;

      setGroupPermissions(gp);
      setUserPermissions(up);
      setGroupRoles(gr);
      setUserRoles(ur);

      // Save initial state for diffing users (since there's no bulk replace)
      initialUserPermissions.current = JSON.parse(JSON.stringify(up));
      initialUserRoles.current = JSON.parse(JSON.stringify(ur));

    } catch (err: any) {
      setError(err.message || 'Failed to load data');
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!client) return;
    setSaving(true);
    setError(null);

    try {
      // 1. Save Group Overrides (Bulk Replacement)
      // Re-index priorities just in case
      const finalGroupPermissions = groupPermissions.map((g, i) => ({ ...g, override_priority: i }));
      const finalGroupRoles = groupRoles.map((g, i) => ({ ...g, override_priority: i }));

      const p1 = updateGroupPermissionOverrides(client.client_id, finalGroupPermissions);
      const p2 = updateGroupRoleOverrides(client.client_id, finalGroupRoles);

      // 2. Save User Permission Overrides (Diff)
      const initialUP = initialUserPermissions.current;
      const currentUP = userPermissions;

      const userPermProms: Promise<any>[] = [];

      // Removed
      initialUP.forEach(init => {
        if (!currentUP.find(curr => curr.user_id === init.user_id)) {
          userPermProms.push(deleteUserPermissionOverride(client.client_id, init.user_id));
        }
      });
      // Added or Changed
      currentUP.forEach(curr => {
        const init = initialUP.find(i => i.user_id === curr.user_id);
        if (!init || init.granted !== curr.granted) {
          userPermProms.push(updateUserPermissionOverride(client.client_id, curr.user_id, curr.granted));
        }
      });

      // 3. Save User Role Overrides (Diff)
      const initialUR = initialUserRoles.current;
      const currentUR = userRoles;
      const userRoleProms: Promise<any>[] = [];

      // Removed
      initialUR.forEach(init => {
        if (!currentUR.find(curr => curr.user_id === init.user_id && curr.role === init.role)) {
          userRoleProms.push(deleteUserRoleOverride(client.client_id, init.user_id, init.role));
        }
      });
      // Added or Changed
      currentUR.forEach(curr => {
        const init = initialUR.find(i => i.user_id === curr.user_id && i.role === curr.role);
        if (!init || init.granted !== curr.granted) {
          userRoleProms.push(updateUserRoleOverride(client.client_id, curr.user_id, curr.role, curr.granted));
        }
      });

      await Promise.all([p1, p2, ...userPermProms, ...userRoleProms]);

      onOpenChange(false);
    } catch (err: any) {
      console.error(err);
      setError(err.message || "Failed to save changes");
    } finally {
      setSaving(false);
    }
  };

  // --- Group Permissions Handlers ---
  const handleGroupPermissionDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (active.id !== over?.id) {
      setGroupPermissions((items) => {
        const oldIndex = items.findIndex((item) => `group-${item.group_id}` === active.id);
        const newIndex = items.findIndex((item) => `group-${item.group_id}` === over?.id);
        return arrayMove(items, oldIndex, newIndex);
      });
    }
  };

  const handleAddGroupPermission = (groupIdStr: string) => {
    if (!client || !groupIdStr) return;
    const groupId = parseInt(groupIdStr);
    if (groupPermissions.some(g => g.group_id === groupId)) return;

    setGroupPermissions([...groupPermissions, {
      group_id: groupId,
      client_id: client.client_id,
      granted: true,
      override_priority: groupPermissions.length
    }]);
  };

  const handleRemoveGroupPermission = (groupId: number) => {
    setGroupPermissions(groupPermissions.filter(g => g.group_id !== groupId));
  };

  const handleToggleGroupPermission = (groupId: number, granted: boolean) => {
    setGroupPermissions(groupPermissions.map(g => g.group_id === groupId ? { ...g, granted } : g));
  };

  // --- User Permissions Handlers ---
  const handleAddUserPermission = (userId: number) => {
    if (!client) return;
    if (userPermissions.some(u => u.user_id === userId)) return;
    setUserPermissions([...userPermissions, {
      user_id: userId,
      client_id: client.client_id,
      granted: true
    }]);
    setPermissionUserSearch('');
  };

  const handleRemoveUserPermission = (userId: number) => {
    setUserPermissions(userPermissions.filter(u => u.user_id !== userId));
  };

  const handleToggleUserPermission = (userId: number, granted: boolean) => {
    setUserPermissions(userPermissions.map(u => u.user_id === userId ? { ...u, granted } : u));
  };

  // --- Group Roles Handlers ---
  const handleGroupRoleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    // We are dragging GROUPS, not individual roles.
    // The id is `grouprole-${groupId}`
    if (active.id !== over?.id) {
      setGroupRoles((items) => {
        // Create a map of group_id -> first index in list to find "positions" of groups?
        // Actually, since we support multiple roles per group, dragging relies on the visual grouping.
        // If we allow reordering GROUPS, we are reordering blocks of roles.
        // This is complex if `groupRoles` is a flat list of roles.
        // Strategy: Group the flat list by group_id. Reorder the GROUPS. Then flatten back.

        const uniqueGroups = Array.from(new Set(items.map(i => i.group_id)));
        const oldGroupIndex = uniqueGroups.findIndex(gid => `grouprole-${gid}` === active.id);
        const newGroupIndex = uniqueGroups.findIndex(gid => `grouprole-${gid}` === over?.id);

        if (oldGroupIndex === -1 || newGroupIndex === -1) return items;

        const newGroupOrder = arrayMove(uniqueGroups, oldGroupIndex, newGroupIndex);

        // Reconstruct flat list based on new group order
        const newItems: GroupAppRoleOverride[] = [];
        newGroupOrder.forEach(gid => {
          const roles = items.filter(i => i.group_id === gid);
          newItems.push(...roles);
        });
        return newItems;
      });
    }
  };

  const handleAddGroupRole = (groupId: number) => {
    if (!client) return;
    // Add a default role entry for this group
    setGroupRoles([...groupRoles, {
      group_id: groupId,
      client_id: client.client_id,
      role: 'user',
      granted: true,
      override_priority: 0 // Priority is recalculated on save/render
    }]);
  };

  const handleRemoveGroupRoleEntry = (index: number) => {
    const newRoles = [...groupRoles];
    newRoles.splice(index, 1);
    setGroupRoles(newRoles);
  }

  const handleUpdateGroupRoleEntry = (index: number, field: 'role' | 'granted', value: any) => {
    const newRoles = [...groupRoles];
    newRoles[index] = { ...newRoles[index], [field]: value };
    setGroupRoles(newRoles);
  }

  // --- User Roles Handlers ---
  const handleAddUserRole = (userId: number) => {
    if (!client) return;
    setUserRoles([...userRoles, {
      user_id: userId,
      client_id: client.client_id,
      role: 'user',
      granted: true
    }]);
    setRoleUserSearch('');
  };

  const handleRemoveUserRoleEntry = (index: number) => {
    const newRoles = [...userRoles];
    newRoles.splice(index, 1);
    setUserRoles(newRoles);
  }

  const handleUpdateUserRoleEntry = (index: number, field: 'role' | 'granted', value: any) => {
    const newRoles = [...userRoles];
    newRoles[index] = { ...newRoles[index], [field]: value };
    setUserRoles(newRoles);
  }

  // --- Helpers ---
  const getGroupName = (id: number) => allGroups.find(g => g.id === id)?.name || `Group ${id}`;
  const getUserName = (id: number) => {
    const u = allUsers.find(user => user.id === id);
    return u ? `${u.name} (${u.username})` : `User ${id}`;
  };

  const availableGroupsForPermission = allGroups.filter(g => !groupPermissions.some(p => p.group_id === g.id));
  const availableUsersForPermission = allUsers.filter(u =>
    !userPermissions.some(p => p.user_id === u.id) &&
    (u.name.toLowerCase().includes(permissionUserSearch.toLowerCase()) || u.username.toLowerCase().includes(permissionUserSearch.toLowerCase()))
  ).slice(0, 10);

  // For Roles, we allow adding the same group/user multiple times (for multiple roles),
  // BUT the "Add Group/User" main dropdown should probably show all groups/users that we might want to add *another* role to?
  // Or just show all?
  // Let's filter slightly: Show all groups/users?
  // Actually, for "Add Group to Role Overrides", it's fine to pick a group that already has roles, to add another one?
  // OR, we have an "Add Role" button INSIDE the group card, and the main "Add Group" is only for new groups?
  // Plan: Main dropdown = Add Group that doesn't exist yet. Inside Group Card = Add another role.
  const uniqueGroupIdsWithRoles = Array.from(new Set(groupRoles.map(r => r.group_id)));
  const availableGroupsForRoles = allGroups.filter(g => !uniqueGroupIdsWithRoles.includes(g.id));

  const uniqueUserIdsWithRoles = Array.from(new Set(userRoles.map(r => r.user_id)));
  const availableUsersForRoles = allUsers.filter(u =>
    !uniqueUserIdsWithRoles.includes(u.id) &&
    (u.name.toLowerCase().includes(roleUserSearch.toLowerCase()) || u.username.toLowerCase().includes(roleUserSearch.toLowerCase()))
  ).slice(0, 50);

  return (
    <Dialog open={open} onOpenChange={(v) => { if (!v) onOpenChange(false); }}>
      <DialogContent className="sm:max-w-[800px] max-h-[90vh] flex flex-col p-0 gap-0">
        <DialogHeader className="p-6 pb-2">
          <DialogTitle>Manage Access: {client?.app_name}</DialogTitle>
          <DialogDescription>
            Configure access and role overrides. Changes are not saved until you click Save.
          </DialogDescription>
        </DialogHeader>

        {loading ? (
          <div className="flex items-center justify-center p-12">
            <Loader2 className="h-8 w-8 animate-spin" />
          </div>
        ) : error ? (
          <div className="p-6 text-destructive">{error}</div>
        ) : (
          <Tabs defaultValue="permissions" className="flex flex-col flex-1 overflow-hidden">
            <div className="px-6">
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="permissions">App Permissions</TabsTrigger>
                <TabsTrigger value="roles">Role Overrides</TabsTrigger>
              </TabsList>
            </div>

            <TabsContent value="permissions" className="flex-1 flex flex-col mt-4">
              <ScrollArea className="h-full flex-1 px-6 [&>[data-slot=scroll-area-scrollbar]]:hidden [&>[data-slot=scroll-area-viewport]]:max-h-[60vh]">
                <div className="space-y-8 pb-6">

                  {/* Group Permissions */}
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <h3 className="text-sm font-medium">Group Overrides</h3>
                      <span className="text-xs text-muted-foreground">Ordered by Priority (Bottom overrides Top)</span>
                    </div>

                    <DndContext
                      sensors={sensors}
                      collisionDetection={closestCenter}
                      onDragEnd={handleGroupPermissionDragEnd}
                    >
                      <SortableContext
                        items={groupPermissions.map(g => `group-${g.group_id}`)}
                        strategy={verticalListSortingStrategy}
                      >
                        {groupPermissions.map((permission) => (
                          <SortableItem key={`group-${permission.group_id}`} id={`group-${permission.group_id}`}>
                            <div className="flex items-center justify-between">
                              <span className="font-medium text-sm">{getGroupName(permission.group_id)}</span>
                              <div className="flex items-center gap-4">
                                <div className="flex items-center gap-2">
                                  <Switch
                                    checked={permission.granted}
                                    onCheckedChange={(c) => handleToggleGroupPermission(permission.group_id, c)}
                                  />
                                  <Label className="w-12 text-xs font-normal text-muted-foreground">
                                    {permission.granted ? 'Allowed' : 'Denied'}
                                  </Label>
                                </div>
                                <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive hover:bg-destructive/10" onClick={() => handleRemoveGroupPermission(permission.group_id)}>
                                  <Trash2 className="h-4 w-4" />
                                </Button>
                              </div>
                            </div>
                          </SortableItem>
                        ))}
                      </SortableContext>
                    </DndContext>

                    {/* Add Group Input */}
                    <div className="pt-2">
                      <Select onValueChange={handleAddGroupPermission}>
                        <SelectTrigger className="w-full">
                          <SelectValue placeholder="Add Group Override..." />
                        </SelectTrigger>
                        <SelectContent>
                          {availableGroupsForPermission.length === 0 ? (
                            <div className="p-2 text-xs text-muted-foreground">No more groups available</div>
                          ) : (
                            availableGroupsForPermission.map(g => (
                              <SelectItem key={g.id} value={g.id.toString()}>{g.name}</SelectItem>
                            ))
                          )}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>

                  <Separator />

                  {/* User Permissions */}
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <h3 className="text-sm font-medium">User Overrides</h3>
                      <span className="text-xs text-muted-foreground">Highest Priority</span>
                    </div>

                    <div className="grid gap-2">
                      {userPermissions.map(permission => (
                        <div key={permission.user_id} className="flex items-center justify-between rounded-md border p-3 bg-card shadow-sm">
                          <span className="text-sm font-medium">{getUserName(permission.user_id)}</span>
                          <div className="flex items-center gap-4">
                            <div className="flex items-center gap-2">
                              <Switch
                                checked={permission.granted}
                                onCheckedChange={(c) => handleToggleUserPermission(permission.user_id, c)}
                              />
                              <Label className="w-12 text-xs font-normal text-muted-foreground">
                                {permission.granted ? 'Allowed' : 'Denied'}
                              </Label>
                            </div>
                            <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive hover:bg-destructive/10" onClick={() => handleRemoveUserPermission(permission.user_id)}>
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </div>
                        </div>
                      ))}
                    </div>

                    {/* Add User Input */}
                    <div className="pt-2">
                      <Select onValueChange={(val) => handleAddUserPermission(parseInt(val))}>
                        <SelectTrigger className="w-full">
                          <SelectValue placeholder="Search and add user override..." />
                        </SelectTrigger>
                        <SelectContent>
                          <div className="p-2">
                            <Input
                              placeholder="Filter users..."
                              value={permissionUserSearch}
                              onChange={(e) => setPermissionUserSearch(e.target.value)}
                              className="h-8 text-xs mb-2"
                              onKeyDown={(e) => e.stopPropagation()}
                            />
                          </div>
                          {availableUsersForPermission.map(u => (
                            <SelectItem key={u.id} value={u.id.toString()}>
                              {u.name} <span className="text-muted-foreground">({u.username})</span>
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>

                </div>
              </ScrollArea>
            </TabsContent>

            <TabsContent value="roles" className="flex-1 flex flex-col mt-4 overflow-hidden">
              <ScrollArea className="max-h-[60vh] flex-1 px-6 [&>[data-slot=scroll-area-scrollbar]]:hidden [&>[data-slot=scroll-area-viewport]]:max-h-[60vh]">
                <div className="space-y-8 pb-6">

                  {/* Group Roles */}
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <h3 className="text-sm font-medium">Group Role Assignments</h3>
                      <span className="text-xs text-muted-foreground">Priority Order (Bottom Wins)</span>
                    </div>

                    <DndContext
                      sensors={sensors}
                      collisionDetection={closestCenter}
                      onDragEnd={handleGroupRoleDragEnd}
                    >
                      <SortableContext
                        items={uniqueGroupIdsWithRoles.map(gid => `grouprole-${gid}`)}
                        strategy={verticalListSortingStrategy}
                      >
                        {uniqueGroupIdsWithRoles.map(groupId => {
                          // Get all roles for this group
                          const rolesForGroup = groupRoles.filter(r => r.group_id === groupId);
                          return (
                            <SortableItem key={`grouprole-${groupId}`} id={`grouprole-${groupId}`}>
                              <div className="space-y-3">
                                <div className="flex items-center justify-between border-b pb-2">
                                  <span className="font-medium text-sm">{getGroupName(groupId)}</span>
                                  <Button variant="ghost" size="sm" className="h-6 text-xs" onClick={() => handleAddGroupRole(groupId)}>
                                    <Plus className="mr-1 h-3 w-3" /> Add Role
                                  </Button>
                                </div>
                                <div className="space-y-2">
                                  {rolesForGroup.map((role, idx) => {
                                    // Need 'global' index in groupRoles array to update
                                    const realIndex = groupRoles.indexOf(role);
                                    return (
                                      <div key={idx} className="flex items-center gap-2">
                                        <Input
                                          className="h-8 text-sm flex-1"
                                          value={role.role}
                                          onChange={(e) => handleUpdateGroupRoleEntry(realIndex, 'role', e.target.value)}
                                          placeholder="Role Name"
                                        />
                                        <div className="flex items-center gap-2">
                                          <Switch
                                            checked={role.granted}
                                            onCheckedChange={(c) => handleUpdateGroupRoleEntry(realIndex, 'granted', c)}
                                          />
                                          <Label className="text-xs text-muted-foreground w-12">{role.granted ? 'Grant' : 'Revoke'}</Label>
                                        </div>
                                        <Button variant="ghost" size="icon" className="h-8 w-8 text-muted-foreground hover:text-destructive" onClick={() => handleRemoveGroupRoleEntry(realIndex)}>
                                          <X className="h-4 w-4" />
                                        </Button>
                                      </div>
                                    );
                                  })}
                                  {rolesForGroup.length === 0 && (
                                    <div className="text-xs text-muted-foreground italic">No roles (group removed from calculation)</div>
                                  )}
                                </div>
                              </div>
                            </SortableItem>
                          );
                        })}
                      </SortableContext>
                    </DndContext>

                    <div className="pt-2">
                      <Select onValueChange={(val) => handleAddGroupRole(parseInt(val))}>
                        <SelectTrigger className="w-full">
                          <SelectValue placeholder="Add Group to Role Overrides..." />
                        </SelectTrigger>
                        <SelectContent>
                          {availableGroupsForRoles.map(g => (
                            <SelectItem key={g.id} value={g.id.toString()}>{g.name}</SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>

                  <Separator />

                  {/* User Roles */}
                  <div className="space-y-3">
                    <h3 className="text-sm font-medium">User Role Assignments</h3>

                    <div className="space-y-4">
                      {uniqueUserIdsWithRoles.map(userId => {
                        const rolesForUser = userRoles.filter(r => r.user_id === userId);
                        return (
                          <div key={userId} className="rounded-md border p-4 bg-card shadow-sm space-y-3">
                            <div className="flex items-center justify-between border-b pb-2">
                              <span className="font-medium text-sm">{getUserName(userId)}</span>
                              <Button variant="ghost" size="sm" className="h-6 text-xs" onClick={() => handleAddUserRole(userId)}>
                                <Plus className="mr-1 h-3 w-3" /> Add Role
                              </Button>
                            </div>
                            <div className="space-y-2">
                              {rolesForUser.map((role, idx) => {
                                const realIndex = userRoles.indexOf(role);
                                return (
                                  <div key={idx} className="flex items-center gap-2">
                                    <Input
                                      className="h-8 text-sm flex-1"
                                      value={role.role}
                                      onChange={(e) => handleUpdateUserRoleEntry(realIndex, 'role', e.target.value)}
                                      placeholder="Role Name"
                                    />
                                    <div className="flex items-center gap-2">
                                      <Switch
                                        checked={role.granted}
                                        onCheckedChange={(c) => handleUpdateUserRoleEntry(realIndex, 'granted', c)}
                                      />
                                      <Label className="text-xs text-muted-foreground w-12">{role.granted ? 'Grant' : 'Revoke'}</Label>
                                    </div>
                                    <Button variant="ghost" size="icon" className="h-8 w-8 text-muted-foreground hover:text-destructive" onClick={() => handleRemoveUserRoleEntry(realIndex)}>
                                      <X className="h-4 w-4" />
                                    </Button>
                                  </div>
                                );
                              })}
                            </div>
                          </div>
                        );
                      })}
                    </div>

                    <div className="pt-2">
                      <Select onValueChange={(val) => handleAddUserRole(parseInt(val))}>
                        <SelectTrigger className="w-full">
                          <SelectValue placeholder="Search and add user override..." />
                        </SelectTrigger>
                        <SelectContent>
                          <div className="p-2">
                            <Input
                              placeholder="Filter users..."
                              value={roleUserSearch}
                              onChange={(e) => setRoleUserSearch(e.target.value)}
                              className="h-8 text-xs mb-2"
                              onKeyDown={(e) => e.stopPropagation()}
                            />
                          </div>
                          {availableUsersForRoles.map(u => (
                            <SelectItem key={u.id} value={u.id.toString()}>
                              {u.name} <span className="text-muted-foreground">({u.username})</span>
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>

                </div>
              </ScrollArea>
            </TabsContent>
          </Tabs>
        )}

        <DialogFooter className="p-6 pt-4 border-t mt-auto bg-muted/40">
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={saving}>Cancel</Button>
          <Button onClick={handleSave} disabled={saving}>
            {saving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Save Changes
          </Button>
        </DialogFooter>

      </DialogContent>
    </Dialog>
  );
}
