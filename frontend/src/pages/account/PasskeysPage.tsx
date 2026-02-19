import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

export default function PasskeysPage() {
  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium">Passkeys</h3>
        <p className="text-sm text-muted-foreground">
          Manage your registered passkeys and security keys.
        </p>
      </div>
      <Card>
        <CardHeader>
          <CardTitle>Your Passkeys</CardTitle>
          <CardDescription>Passkeys allow you to sign in securely without a password.</CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">Passkey management is coming soon.</p>
        </CardContent>
      </Card>
    </div>
  );
}
