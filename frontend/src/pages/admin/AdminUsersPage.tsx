import { useState, useEffect } from 'react';
import {
  Loader2,
  MoreHorizontal,
  Pencil,
  Plus,
  AlertCircle,
  Mail
} from 'lucide-react';
import {
  listUsers,
  createUser,
  updateUser,
  type User,
  type CreateUserRequest,
  type UpdateUserRequest,
  sendRegistrationLink,
} from '../../api/contollers/admin-users';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Checkbox } from '@/components/ui/checkbox';
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
} from '@/components/ui/dialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

export default function AdminUsersPage() {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [userToSendLink, setUserToSendLink] = useState<User | null>(null);
  const [editingUser, setEditingUser] = useState<User | null>(null);
  const [formData, setFormData] = useState<CreateUserRequest>({
    email: '',
    username: '',
    name: '',
    is_suspended: false,
    is_admin: false,
  });

  useEffect(() => {
    loadUsers();
  }, []);

  const loadUsers = async () => {
    setLoading(true);
    const result = await listUsers();
    if (result.success) {
      setUsers(result.data.users);
      setError(null);
    } else {
      setError(result.message);
    }
    setLoading(false);
  };

  const handleCreateUser = () => {
    setEditingUser(null);
    setFormData({
      email: '',
      username: '',
      name: '',
      is_suspended: false,
      is_admin: false,
    });
    setIsDialogOpen(true);
  };

  const handleEditUser = (user: User) => {
    setEditingUser(user);
    setFormData({
      email: user.email,
      username: user.username,
      name: user.name,
      is_suspended: user.is_suspended,
      is_admin: user.is_admin,
    });
    setIsDialogOpen(true);
  };

  const handleSaveUser = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccessMessage(null);

    let result;
    if (editingUser) {
      result = await updateUser(editingUser.id, formData as UpdateUserRequest);
    } else {
      result = await createUser(formData);
    }

    if (result.success) {
      setIsDialogOpen(false);
      loadUsers();
    } else {
      setError(result.message);
    }
  };

  const handleSendRegistrationLink = async () => {
    if (!userToSendLink) return;

    const result = await sendRegistrationLink(userToSendLink.id);
    if (result.success) {
      setSuccessMessage(`Registration link sent to ${userToSendLink.email}`);
      setError(null);
      setUserToSendLink(null);
    } else {
      setError(result.message);
      // We keep the dialog open or close it? 
      // If we close it, the user sees the error on the main page.
      // Let's close it so the user sees the error on the main page where we show errors.
      setUserToSendLink(null);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">Users</h1>
        <Button onClick={handleCreateUser}>
          <Plus className="mr-2 h-4 w-4" />
          Create User
        </Button>
      </div>

      {successMessage && (
        <div className="p-4 rounded-md bg-green-500/15 text-green-600 flex items-center gap-2 border border-green-500/20">
          <Mail className="h-4 w-4" />
          <p>{successMessage}</p>
        </div>
      )}

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
                <TableHead>Username</TableHead>
                <TableHead>Name</TableHead>
                <TableHead>Email</TableHead>
                <TableHead>Admin</TableHead>
                <TableHead>Suspended</TableHead>
                <TableHead className="w-[70px]"></TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {users.map((user) => (
                <TableRow key={user.id}>
                  <TableCell className="font-medium">{user.username}</TableCell>
                  <TableCell>{user.name}</TableCell>
                  <TableCell>{user.email}</TableCell>
                  <TableCell>
                    {user.is_admin ? (
                      <span className="inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-primary text-primary-foreground shadow hover:bg-primary/80">Yes</span>
                    ) : (
                      <span className="text-muted-foreground">No</span>
                    )}
                  </TableCell>
                  <TableCell>
                    {user.is_suspended ? (
                      <span className="inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-destructive text-destructive-foreground shadow hover:bg-destructive/80">Yes</span>
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
                        <DropdownMenuItem onClick={() => handleEditUser(user)}>
                          <Pencil className="mr-2 h-4 w-4" />
                          Edit
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => {
                          setSuccessMessage(null);
                          setError(null);
                          setUserToSendLink(user);
                        }}>
                          <Mail className="mr-2 h-4 w-4" />
                          Send Registration Link
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

      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent className="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle>{editingUser ? 'Edit User' : 'Create User'}</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSaveUser}>
            <div className="grid gap-4 py-4">
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="username" className="text-right">
                  Username
                </Label>
                <Input
                  id="username"
                  value={formData.username}
                  onChange={(e) => setFormData({ ...formData, username: e.target.value })}
                  className="col-span-3"
                  required
                />
              </div>
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="name" className="text-right">
                  Name
                </Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="col-span-3"
                  required
                />
              </div>
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="email" className="text-right">
                  Email
                </Label>
                <Input
                  id="email"
                  type="email"
                  value={formData.email}
                  onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                  className="col-span-3"
                  required
                />
              </div>
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="is_admin" className="text-right">Admin</Label>
                <div className="col-span-3 flex items-center space-x-2">
                  <Checkbox
                    id="is_admin"
                    checked={formData.is_admin}
                    onCheckedChange={(checked) => setFormData({ ...formData, is_admin: checked as boolean })}
                  />
                  <label
                    htmlFor="is_admin"
                    className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                  >
                    Grant admin privileges
                  </label>
                </div>
              </div>
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="is_suspended" className="text-right">Suspended</Label>
                <div className="col-span-3 flex items-center space-x-2">
                  <Checkbox
                    id="is_suspended"
                    checked={formData.is_suspended}
                    onCheckedChange={(checked) => setFormData({ ...formData, is_suspended: checked as boolean })}
                  />
                  <label
                    htmlFor="is_suspended"
                    className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                  >
                    Suspend account
                  </label>
                </div>
              </div>
            </div>
            <DialogFooter>
              <Button type="submit">{editingUser ? 'Save changes' : 'Create user'}</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <Dialog open={!!userToSendLink} onOpenChange={(open) => !open && setUserToSendLink(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Send Registration Link</DialogTitle>
          </DialogHeader>
          <div className="py-4">
            <p>
              Are you sure you want to send a registration link to{' '}
              <span className="font-semibold">{userToSendLink?.email}</span>?
            </p>
            <p className="text-sm text-muted-foreground mt-2">
              This will allow the user to set up their account credentials.
            </p>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setUserToSendLink(null)}>
              Cancel
            </Button>
            <Button onClick={handleSendRegistrationLink}>
              Send Link
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

