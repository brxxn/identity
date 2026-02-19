import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

export default function AppsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium">Apps</h3>
        <p className="text-sm text-muted-foreground">
          Manage your connected applications and integrations.
        </p>
      </div>
      <Card>
        <CardHeader>
          <CardTitle>Connected Applications</CardTitle>
          <CardDescription>You have no connected applications.</CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">No apps found.</p>
        </CardContent>
      </Card>
    </div>
  );
}
