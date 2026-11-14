import {
  ErrorBoundary,
  lazy,
  LocationProvider,
  Route,
  Router,
} from "preact-iso";
import { useEffect } from "preact/hooks";
import { themeEvents } from "./layouts/themes";
import type { ComponentChildren, ComponentType } from "preact";

const DefaultLayout = lazy(() => import("@/layouts/DefaultLayout"));
const AdminLayout = lazy(() => import("@/layouts/AdminLayout"));

const FileBrowser = lazy(() => import("@/pages/FileBrowser"));
const AdminHome = lazy(() => import("@/pages/AdminHome"));

export function App() {
  useEffect(themeEvents, []);

  const MainPage = withLayout(DefaultLayout, FileBrowser);
  const AdminPage = withLayout(AdminLayout, AdminHome);

  return (
    <LocationProvider>
      <ErrorBoundary>
        <Router>
          <Route path="/dev/admin" component={AdminPage} />
          <Route default component={MainPage} />
        </Router>
      </ErrorBoundary>
    </LocationProvider>
  );
}

function withLayout(
  Layout: ComponentType<{ children?: ComponentChildren }>,
  Page: ComponentType,
) {
  return function wrapperPage() {
    return (
      <Layout>
        <Page />
      </Layout>
    );
  };
}
