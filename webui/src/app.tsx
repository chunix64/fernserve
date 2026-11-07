import {
  ErrorBoundary,
  lazy,
  LocationProvider,
  Route,
  Router,
} from "preact-iso";

const DefaultLayout = lazy(() => import("@/layouts/default"));

export function App() {
  return (
    <LocationProvider>
      <ErrorBoundary>
        <Router>
          <Route path="/" component={DefaultLayout} />
        </Router>
      </ErrorBoundary>
    </LocationProvider>
  );
}
