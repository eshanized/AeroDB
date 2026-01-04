import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AuthProvider, useAuth } from "@/lib/auth/AuthContext";
import { AppLayout } from "@/components/layout";
import { LoginPage } from "@/features/auth/components/LoginPage";
import { DatabasePage } from "@/features/database/components/DatabasePage";
import { AuthPage } from "@/features/auth/components/AuthPage";
import { StoragePage } from "@/features/storage/components/StoragePage";
import { RealtimePage } from "@/features/realtime/components/RealtimePage";
import { ClusterPage } from "@/features/cluster/components/ClusterPage";
import { ObservabilityPage } from "@/features/observability/components/ObservabilityPage";
import { Toaster, TooltipProvider } from "@/components/ui";

// Create React Query client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
      staleTime: 30000, // 30 seconds
    },
  },
});

// Protected route wrapper
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-[hsl(var(--background))]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-[hsl(var(--primary))]"></div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
}

// App routes
function AppRoutes() {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-[hsl(var(--background))]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-[hsl(var(--primary))]"></div>
      </div>
    );
  }

  return (
    <Routes>
      {/* Public routes */}
      <Route
        path="/login"
        element={
          isAuthenticated ? <Navigate to="/database" replace /> : <LoginPage />
        }
      />

      {/* Protected routes */}
      <Route
        element={
          <ProtectedRoute>
            <AppLayout />
          </ProtectedRoute>
        }
      >
        <Route index element={<Navigate to="/database" replace />} />
        <Route path="/database" element={<DatabasePage />} />
        <Route path="/auth" element={<AuthPage />} />
        <Route path="/storage" element={<StoragePage />} />
        <Route path="/realtime" element={<RealtimePage />} />
        <Route path="/cluster" element={<ClusterPage />} />
        <Route path="/observability" element={<ObservabilityPage />} />
      </Route>

      {/* Fallback */}
      <Route path="*" element={<Navigate to="/database" replace />} />
    </Routes>
  );
}

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <TooltipProvider>
          <BrowserRouter>
            <AppRoutes />
            <Toaster />
          </BrowserRouter>
        </TooltipProvider>
      </AuthProvider>
    </QueryClientProvider>
  );
}

export default App;
