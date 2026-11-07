import {
  ErrorBoundary,
  lazy,
  LocationProvider,
  Route,
  Router,
} from "preact-iso";
import { useEffect } from "preact/hooks";
import { loadTheme } from "./layouts/themes";
import type { ComponentChildren, ComponentType } from "preact";

const FileBrowser = lazy(() => import("@/pages/FileBrowser"));
const DefaultLayout = lazy(() => import("@/layouts/default"));

function themeEvents() {
  window.addEventListener("load", loadTheme);
  window.addEventListener("storage", loadTheme);

  if (document.readyState === "complete") loadTheme();

  return () => {
    window.removeEventListener("load", loadTheme);
    window.removeEventListener("storage", loadTheme);
  };
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

export function App() {
  useEffect(themeEvents, []);

  const MainPage = withLayout(DefaultLayout, FileBrowser);

  return (
    <LocationProvider>
      <ErrorBoundary>
        <Router>
          {[<Route default component={MainPage} />]}
        </Router>
      </ErrorBoundary>
    </LocationProvider>
  );
}
