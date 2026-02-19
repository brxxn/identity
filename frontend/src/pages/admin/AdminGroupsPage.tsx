import { useState, useEffect } from 'react';
import {
  Loader2,
  MoreHorizontal,
  Pencil,
  Plus,
  AlertCircle,
  Users,
  Trash
} from 'lucide-react';
import {
  listGroups,
  createGroup,
  updateGroup,
  listGroupMembers,
  addGroupMember,
  removeGroupMember,
  type Group,
  type CreateGroupRequest,
  type UpdateGroupRequest,
} from '../../api/contollers/admin-groups';
import { listUsers, type User } from '../../api/contollers/admin-users';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogDescription,
} from '@/components/ui/dialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

export default function AdminGroupsPage() {
  const [groups, setGroups] = useState<Group[]>([]);
  const [allUsers, setAllUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [isGroupDialogOpen, setIsGroupDialogOpen] = useState(false);
  const [editingGroup, setEditingGroup] = useState<Group | null>(null);
  const [groupFormData, setGroupFormData] = useState<CreateGroupRequest>({
    slug: '',
    name: '',
    description: '',
  });

  const [isMembersDialogOpen, setIsMembersDialogOpen] = useState(false);
  const [selectedGroup, setSelectedGroup] = useState<Group | null>(null);
  const [groupMembers, setGroupMembers] = useState<User[]>([]);
  const [loadingMembers, setLoadingMembers] = useState(false);
  const [selectedUserToAdd, setSelectedUserToAdd] = useState<string>('');

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    await Promise.all([loadGroups(), loadAllUsers()]);
    setLoading(false);
  };

  const loadGroups = async () => {
    const result = await listGroups();
    if (result.success) {
      setGroups(result.data.groups);
      setError(null);
    } else {
      setError(result.message);
    }
  };

  const loadAllUsers = async () => {
    const result = await listUsers();
    if (result.success) {
      setAllUsers(result.data.users);
    }
  };

  const loadGroupMembers = async (groupId: number) => {
    setLoadingMembers(true);
    const result = await listGroupMembers(groupId);
    if (result.success) {
      setGroupMembers(result.data.members);
    } else {
      setError(result.message);
    }
    setLoadingMembers(false);
  };

  const handleCreateGroup = () => {
    setEditingGroup(null);
    setGroupFormData({
      slug: '',
      name: '',
      description: '',
    });
    setIsGroupDialogOpen(true);
  };

  const handleEditGroup = (group: Group) => {
    if (group.is_managed) {
      setError('Cannot edit managed groups');
      return;
    }
    setEditingGroup(group);
    setGroupFormData({
      slug: group.slug,
      name: group.name,
      description: group.description,
    });
    setIsGroupDialogOpen(true);
  };

  const handleSaveGroup = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    let result;
    if (editingGroup) {
      result = await updateGroup(editingGroup.id, groupFormData as UpdateGroupRequest);
    } else {
      result = await createGroup(groupFormData);
    }

    if (result.success) {
      setIsGroupDialogOpen(false);
      loadGroups();
    } else {
      setError(result.message);
    }
  };

  const handleManageMembers = async (group: Group) => {
    setSelectedGroup(group);
    setIsMembersDialogOpen(true);
    await loadGroupMembers(group.id);
  };

  const handleAddMember = async () => {
    if (!selectedGroup || !selectedUserToAdd) return;

    const userId = parseInt(selectedUserToAdd);
    const result = await addGroupMember(selectedGroup.id, userId);
    if (result.success) {
      setGroupMembers(result.data.members);
      setSelectedUserToAdd('');
    } else {
      setError(result.message);
    }
  };

  const handleRemoveMember = async (userId: number) => {
    if (!selectedGroup) return;

    const result = await removeGroupMember(selectedGroup.id, userId);
    if (result.success) {
      setGroupMembers(result.data.members);
    } else {
      setError(result.message);
    }
  };

  const availableUsers = allUsers.filter(
    (user) => !groupMembers.some((member) => member.id === user.id)
  );

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">Groups</h1>
        <Button onClick={handleCreateGroup}>
          <Plus className="mr-2 h-4 w-4" />
          Create Group
        </Button>
      </div>

      {error && (
        <div className="p-4 rounded-md bg-destructive/15 text-destructive flex items-center gap-2">
          <AlertCircle className="h-4 w-4" />
          <p>{error}</p>
        </div>
      )}

      {loading ? (
        <div className="flex items-center justify-center p-8">
          <Loader2 className="h-6 w-6 animate-spin" />
        </div>
      ) : (
        <div className="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Slug</TableHead>
                <TableHead>Name</TableHead>
                <TableHead>Description</TableHead>
                <TableHead>Managed</TableHead>
                <TableHead className="w-[70px]"></TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {groups.map((group) => (
                <TableRow key={group.id}>
                  <TableCell className="font-medium">{group.slug}</TableCell>
                  <TableCell>{group.name}</TableCell>
                  <TableCell className="text-muted-foreground">{group.description}</TableCell>
                  <TableCell>
                    {group.is_managed ? (
                      <span className="inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-primary text-primary-foreground shadow hover:bg-primary/80">Yes</span>
                    ) : (
                      <span className="text-muted-foreground">No</span>
                    )}
                  </TableCell>
                  <TableCell>
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" className="h-8 w-8 p-0">
                          <span className="sr-only">Open menu</span>
                          <MoreHorizontal className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem onClick={() => handleManageMembers(group)}>
                          <Users className="mr-2 h-4 w-4" />
                          Manage Members
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onClick={() => handleEditGroup(group)}
                          disabled={group.is_managed}
                        >
                          <Pencil className="mr-2 h-4 w-4" />
                          Edit
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}

      {/* Group Edit/Create Dialog */}
      <Dialog open={isGroupDialogOpen} onOpenChange={setIsGroupDialogOpen}>
        <DialogContent className="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle>{editingGroup ? 'Edit Group' : 'Create Group'}</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSaveGroup}>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="slug">Slug</Label>
                <Input
                  id="slug"
                  value={groupFormData.slug}
                  onChange={(e) => setGroupFormData({ ...groupFormData, slug: e.target.value })}
                  required
                  placeholder="lowercase-no-spaces"
                />
                <p className="text-xs text-muted-foreground">Unique identifier for the group.</p>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="name">Name</Label>
                <Input
                  id="name"
                  value={groupFormData.name}
                  onChange={(e) => setGroupFormData({ ...groupFormData, name: e.target.value })}
                  required
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="description">Description</Label>
                <Textarea
                  id="description"
                  value={groupFormData.description}
                  onChange={(e) => setGroupFormData({ ...groupFormData, description: e.target.value })}
                  rows={3}
                />
              </div>
            </div>
            <DialogFooter>
              <Button type="submit">{editingGroup ? 'Save changes' : 'Create group'}</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Group Members Dialog */}
      <Dialog open={isMembersDialogOpen} onOpenChange={setIsMembersDialogOpen}>
        <DialogContent className="sm:max-w-[600px] max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Manage Members: {selectedGroup?.name}</DialogTitle>
            <DialogDescription>
              {selectedGroup?.is_managed ? (
                <span className="text-destructive">
                  This is a managed group. Members cannot be modified directly.
                </span>
              ) : "Add or remove members from this group."}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-6 py-4">
            {/* Add Member Section */}
            {!selectedGroup?.is_managed && (
              <div className="flex gap-2 items-end">
                <div className="grid gap-2 flex-1">
                  <Label>Add Member</Label>
                  <Select
                    value={selectedUserToAdd}
                    onValueChange={setSelectedUserToAdd}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select a user to add..." />
                    </SelectTrigger>
                    <SelectContent>
                      {availableUsers.map((user) => (
                        <SelectItem key={user.id} value={user.id.toString()}>
                          {user.name} (@{user.username})
                        </SelectItem>
                      ))}
                      {availableUsers.length === 0 && (
                        <div className="p-2 text-sm text-muted-foreground">No more users to add</div>
                      )}
                    </SelectContent>
                  </Select>
                </div>
                <Button onClick={handleAddMember} disabled={!selectedUserToAdd}>Add</Button>
              </div>
            )}

            {/* Members List */}
            <div className="space-y-4">
              <h4 className="font-medium leading-none">Current Members</h4>
              {loadingMembers ? (
                <div className="flex justify-center p-4">
                  <Loader2 className="h-4 w-4 animate-spin" />
                </div>
              ) : groupMembers.length > 0 ? (
                <div className="border rounded-md divide-y">
                  {groupMembers.map((member) => (
                    <div key={member.id} className="flex items-center justify-between p-3">
                      <div>
                        <div className="font-medium">{member.name}</div>
                        <div className="text-sm text-muted-foreground">@{member.username}</div>
                      </div>
                      {!selectedGroup?.is_managed && (
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleRemoveMember(member.id)}
                          className="text-destructive hover:bg-destructive/10 hover:text-destructive"
                        >
                          <Trash className="h-4 w-4" />
                        </Button>
                      )}
                    </div>
                  ))}
                </div>
              ) : (
                <div className="text-sm text-muted-foreground text-center p-4 border rounded-md border-dashed">
                  No members in this group.
                </div>
              )}
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
