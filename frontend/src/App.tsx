import './App.css'
import { Redirect, Route, Switch } from 'wouter'
import PasskeyLogin from './pages/login/PasskeyLogin'
import PasskeyRegister from './pages/register/PasskeyRegister'
import ProtectedRoute from './components/ProtectedRoute'
import AdminRoute from './components/AdminRoute'
import ProfilePage from './pages/account/ProfilePage'
import AppsPage from './pages/account/AppsPage'
import GroupsPage from './pages/account/GroupsPage'
import PasskeysPage from './pages/account/PasskeysPage'
import DangerZonePage from './pages/account/DangerZonePage'
import AdminUsersPage from './pages/admin/AdminUsersPage'
import AdminGroupsPage from './pages/admin/AdminGroupsPage'
import AdminClientsPage from './pages/admin/AdminClientsPage'
import AuthorizePage from './pages/oauth/AuthorizePage'
import Layout from './components/Layout'

function App() {

  return (
    <Switch>
      <Route path="/auth/login/passkey" component={PasskeyLogin} />
      <Route path="/auth/register/passkey" component={PasskeyRegister} />

      <Route path="/oauth/authorize">
        {() => (
          <ProtectedRoute>
            <AuthorizePage />
          </ProtectedRoute>
        )}
      </Route>

      {/* Catch-all route for authenticated pages */}
      <Route>
        <Layout>
          <Switch>
            <Route path="/account/profile">
              {() => <ProtectedRoute><ProfilePage /></ProtectedRoute>}
            </Route>
            <Route path="/account/apps">
              {() => <ProtectedRoute><AppsPage /></ProtectedRoute>}
            </Route>
            <Route path="/account/groups">
              {() => <ProtectedRoute><GroupsPage /></ProtectedRoute>}
            </Route>
            <Route path="/account/passkeys">
              {() => <ProtectedRoute><PasskeysPage /></ProtectedRoute>}
            </Route>
            <Route path="/account/danger-zone">
              {() => <ProtectedRoute><DangerZonePage /></ProtectedRoute>}
            </Route>

            <Route path="/admin/users">
              {() => <AdminRoute><AdminUsersPage /></AdminRoute>}
            </Route>
            <Route path="/admin/groups">
              {() => <AdminRoute><AdminGroupsPage /></AdminRoute>}
            </Route>
            <Route path="/admin/clients">
              {() => <AdminRoute><AdminClientsPage /></AdminRoute>}
            </Route>

            {/* Default redirect to profile */}
            <Redirect to="/account/profile" />
          </Switch>
        </Layout>
      </Route>
    </Switch>
  )
}

export default App
