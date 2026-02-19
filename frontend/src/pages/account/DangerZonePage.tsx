import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { AlertCircle } from 'lucide-react';

export default function DangerZonePage() {
  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-destructive">Danger Zone</h3>
        <p className="text-sm text-muted-foreground">
          Irreversible and destructive actions.
        </p>
      </div>

      <div className="rounded-md border border-destructive/50 border-l-4 border-l-destructive bg-destructive/10 p-4">
        <div className="flex items-center gap-2 text-destructive">
          <AlertCircle className="h-5 w-5" />
          <p className="font-semibold">Proceed with caution</p>
        </div>
        <p className="mt-2 text-sm text-destructive/90">
          These actions are irreversible. Please be certain.
        </p>
      </div>

      <Card className="border-destructive/20">
        <CardHeader>
          <CardTitle>Sign out everywhere</CardTitle>
          <CardDescription>This will sign you out of all active sessions on all devices.</CardDescription>
        </CardHeader>
        <CardContent>
          <Button variant="destructive" disabled>
            Sign out everywhere
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
