import { useState, useEffect } from 'react';
import {
  type Client,
  type CreateClientRequest,
  type UpdateClientRequest,
  listClients,
  createClient,
  updateClient,
  rotateClientSecret
} from '../../api/contollers/admin-clients';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import {
  Alert,
  AlertTitle,
  AlertDescription,
} from '@/components/ui/alert';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import ClientAccessDialog from '@/components/admin/ClientAccessDialog';
import {
  Loader2,
  MoreHorizontal,
  Pencil,
  Plus,
  Key,
  Copy,
  AlertTriangle,
  Check,
  Shield // New icon Import
} from 'lucide-react';

export default function AdminClientsPage() {
  const [clients, setClients] = useState<Client[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [editingClient, setEditingClient] = useState<Client | null>(null);
  const [formData, setFormData] = useState<CreateClientRequest>({
    app_name: '',
    app_description: '',
    redirect_uris: [],
    is_disabled: false,
    default_allowed: false,
    allow_implicit_flow: false,
    allow_explicit_flow: true,
  });
  const [redirectUrisText, setRedirectUrisText] = useState('');

  const [isSecretDialogOpen, setIsSecretDialogOpen] = useState(false);
  const [newSecret, setNewSecret] = useState<string | null>(null);
  const [secretClient, setSecretClient] = useState<Client | null>(null);
  const [copied, setCopied] = useState(false);

  // New State for Access Dialog
  const [isAccessDialogOpen, setIsAccessDialogOpen] = useState(false);
  const [accessClient, setAccessClient] = useState<Client | null>(null);

  const handleManageAccess = (client: Client) => {
    setAccessClient(client);
    setIsAccessDialogOpen(true);
  };


  useEffect(() => {
    loadClients();
  }, []);

  const loadClients = async () => {
    setLoading(true);
    const result = await listClients();
    if (result.success) {
      setClients(result.data.clients);
      setError(null);
    } else {
      setError(result.message);
    }
    setLoading(false);
  };

  const handleCreateClient = () => {
    setEditingClient(null);
    setFormData({
      app_name: '',
      app_description: '',
      redirect_uris: [],
      is_disabled: false,
      default_allowed: false,
      allow_implicit_flow: false,
      allow_explicit_flow: true,
    });
    setRedirectUrisText('');
    setIsDialogOpen(true);
  };

  const handleEditClient = (client: Client) => {
    if (client.is_managed) {
      setError('Cannot edit managed clients');
      return;
    }
    setEditingClient(client);
    setFormData({
      app_name: client.app_name,
      app_description: client.app_description,
      redirect_uris: client.redirect_uris,
      is_disabled: client.is_disabled,
      default_allowed: client.default_allowed,
      allow_implicit_flow: client.allow_implicit_flow,
      allow_explicit_flow: client.allow_explicit_flow,
    });
    setRedirectUrisText(client.redirect_uris.join('\n'));
    setIsDialogOpen(true);
  };

  const handleSaveClient = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    const uris = redirectUrisText
      .split('\n')
      .map((uri) => uri.trim())
      .filter((uri) => uri.length > 0);

    const data = { ...formData, redirect_uris: uris };

    let result;
    if (editingClient) {
      result = await updateClient(editingClient.client_id, data as UpdateClientRequest);
      if (result.success) {
        setIsDialogOpen(false);
        loadClients();
      } else {
        setError(result.message);
      }
    } else {
      result = await createClient(data);
      if (result.success) {
        setNewSecret(result.data.client_secret);
        setSecretClient(result.data.client);
        setIsDialogOpen(false);
        setIsSecretDialogOpen(true);
        loadClients();
      } else {
        setError(result.message);
      }
    }
  };

  const handleRotateSecret = async (client: Client) => {
    if (client.is_managed) {
      setError('Cannot rotate secrets for managed clients');
      return;
    }

    const result = await rotateClientSecret(client.client_id);
    if (result.success) {
      setNewSecret(result.data.client_secret);
      setSecretClient(result.data.client);
      setIsSecretDialogOpen(true);
      loadClients();
    } else {
      setError(result.message);
    }
  };

  const handleCopyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleCloseDialog = () => {
    setIsDialogOpen(false);
    setEditingClient(null);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">Clients</h1>
        <Button onClick={handleCreateClient}>
          <Plus className="mr-2 h-4 w-4" />
          Create Client
        </Button>
      </div>

      {error && (
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
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
                <TableHead>App Name</TableHead>
                <TableHead>Client ID</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Flows</TableHead>
                <TableHead>Managed</TableHead>
                <TableHead className="w-[70px]"></TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {clients.map((client) => (
                <TableRow key={client.client_id}>
                  <TableCell className="font-medium">{client.app_name}</TableCell>
                  <TableCell>
                    <div className="flex items-center gap-2 font-mono text-xs text-muted-foreground">
                      {client.client_id.substring(0, 16)}...
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-4 w-4"
                        onClick={() => handleCopyToClipboard(client.client_id)}
                      >
                        <Copy className="h-3 w-3" />
                      </Button>
                    </div>
                  </TableCell>
                  <TableCell>
                    {client.is_disabled ? (
                      <Badge variant="destructive">Disabled</Badge>
                    ) : (
                      <Badge variant="secondary" className="bg-green-100 text-green-800 hover:bg-green-200 border-none">Active</Badge>
                    )}
                  </TableCell>
                  <TableCell>
                    <div className="flex gap-1 flex-wrap">
                      {client.allow_explicit_flow && (
                        <Badge variant="outline" className="text-xs">
                          Explicit
                        </Badge>
                      )}
                      {client.allow_implicit_flow && (
                        <Badge variant="outline" className="text-xs">
                          Implicit
                        </Badge>
                      )}
                    </div>
                  </TableCell>
                  <TableCell>
                    {client.is_managed ? (
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
                        <DropdownMenuItem
                          onClick={() => handleEditClient(client)}
                          disabled={client.is_managed}
                        >
                          <Pencil className="mr-2 h-4 w-4" />
                          Edit
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onClick={() => handleManageAccess(client)}
                        >
                          <Shield className="mr-2 h-4 w-4" />
                          Manage Access
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem
                          onClick={() => handleRotateSecret(client)}
                          disabled={client.is_managed}
                          className="text-destructive focus:text-destructive"
                        >
                          <Key className="mr-2 h-4 w-4" />
                          Rotate Secret
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

      {/* Client Edit/Create Dialog */}
      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent className="sm:max-w-[600px] max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{editingClient ? 'Edit OAuth Client' : 'Create OAuth Client'}</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSaveClient}>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="app_name">Application Name</Label>
                <Input
                  id="app_name"
                  value={formData.app_name}
                  onChange={(e) => setFormData({ ...formData, app_name: e.target.value })}
                  required
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="app_description">Application Description</Label>
                <Textarea
                  id="app_description"
                  value={formData.app_description}
                  onChange={(e) => setFormData({ ...formData, app_description: e.target.value })}
                  rows={3}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="redirect_uris">Redirect URIs</Label>
                <Textarea
                  id="redirect_uris"
                  value={redirectUrisText}
                  onChange={(e) => setRedirectUrisText(e.target.value)}
                  rows={5}
                  placeholder={`https://example.com/callback\nhttps://app.example.com/oauth/callback`}
                  className="font-mono text-sm"
                />
                <p className="text-xs text-muted-foreground">One URI per line. These are the allowed callback URLs for OAuth flows.</p>
              </div>

              <div className="space-y-4">
                <h3 className="text-lg font-medium">OAuth Flow Settings</h3>
                <div className="flex items-start space-x-3 space-y-0">
                  <Checkbox
                    id="allow_explicit_flow"
                    checked={formData.allow_explicit_flow}
                    onCheckedChange={(checked) => setFormData({ ...formData, allow_explicit_flow: checked as boolean })}
                  />
                  <div className="grid gap-1.5 leading-none">
                    <Label htmlFor="allow_explicit_flow">
                      Allow Authorization Code Flow (Explicit)
                    </Label>
                    <p className="text-sm text-muted-foreground">
                      Recommended for server-side applications
                    </p>
                  </div>
                </div>
                <div className="flex items-start space-x-3 space-y-0">
                  <Checkbox
                    id="allow_implicit_flow"
                    checked={formData.allow_implicit_flow}
                    onCheckedChange={(checked) => setFormData({ ...formData, allow_implicit_flow: checked as boolean })}
                  />
                  <div className="grid gap-1.5 leading-none">
                    <Label htmlFor="allow_implicit_flow">
                      Allow Implicit Flow
                    </Label>
                    <p className="text-sm text-muted-foreground">
                      For client-side applications (less secure)
                    </p>
                  </div>
                </div>
              </div>

              <div className="space-y-4">
                <h3 className="text-lg font-medium">Access Settings</h3>
                <div className="flex items-start space-x-3 space-y-0">
                  <Checkbox
                    id="default_allowed"
                    checked={formData.default_allowed}
                    onCheckedChange={(checked) => setFormData({ ...formData, default_allowed: checked as boolean })}
                  />
                  <div className="grid gap-1.5 leading-none">
                    <Label htmlFor="default_allowed">
                      Default Allowed
                    </Label>
                    <p className="text-sm text-muted-foreground">
                      Users are automatically granted access without explicit approval
                    </p>
                  </div>
                </div>
                <div className="flex items-start space-x-3 space-y-0">
                  <Checkbox
                    id="is_disabled"
                    checked={formData.is_disabled}
                    onCheckedChange={(checked) => setFormData({ ...formData, is_disabled: checked as boolean })}
                  />
                  <div className="grid gap-1.5 leading-none">
                    <Label htmlFor="is_disabled">
                      Disabled
                    </Label>
                    <p className="text-sm text-muted-foreground">
                      Prevent this client from authenticating
                    </p>
                  </div>
                </div>
              </div>
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={handleCloseDialog}>Cancel</Button>
              <Button type="submit">{editingClient ? 'Save changes' : 'Create Client'}</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Client Secret Dialog */}
      <Dialog open={isSecretDialogOpen} onOpenChange={setIsSecretDialogOpen}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>Client Secret</DialogTitle>
            <Alert variant="destructive" className="mt-4">
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>
                <strong>Important:</strong> Copy this secret now. It will not be shown again!
              </AlertDescription>
            </Alert>
          </DialogHeader>

          {secretClient && (
            <div className="space-y-4 py-4">
              <div className="grid gap-2">
                <Label>Application Name</Label>
                <div className="font-medium">{secretClient.app_name}</div>
              </div>

              <div className="grid gap-2">
                <Label>Client ID</Label>
                <div className="flex items-center space-x-2">
                  <code className="relative rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm flex-1">
                    {secretClient.client_id}
                  </code>
                  <Button
                    variant="outline"
                    size="icon"
                    onClick={() => handleCopyToClipboard(secretClient.client_id)}
                  >
                    {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                  </Button>
                </div>
              </div>

              <div className="grid gap-2">
                <Label>Client Secret</Label>
                <div className="flex items-center space-x-2">
                  <code className="relative rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm flex-1 text-destructive font-bold break-all">
                    {newSecret}
                  </code>
                  <Button
                    variant="outline"
                    size="icon"
                    onClick={() => newSecret && handleCopyToClipboard(newSecret)}
                  >
                    {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                  </Button>
                </div>
              </div>
            </div>
          )}
          <DialogFooter>
            <Button onClick={() => setIsSecretDialogOpen(false)}>Done</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ClientAccessDialog
        client={accessClient}
        open={isAccessDialogOpen}
        onOpenChange={setIsAccessDialogOpen}
      />
    </div>
  );
}
