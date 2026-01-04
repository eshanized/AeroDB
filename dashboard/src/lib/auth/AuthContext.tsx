import {
    createContext,
    useContext,
    useEffect,
    useState,
    useCallback,
} from "react";
import type { ReactNode } from "react";
import { api, setTokens, clearTokens, loadRefreshToken } from "@/lib/api/client";
import { config } from "@/config";
import type { User, Session, AuthResponse, ApiError } from "@/types";

interface AuthState {
    user: User | null;
    session: Session | null;
    isLoading: boolean;
    isAuthenticated: boolean;
}

interface AuthContextValue extends AuthState {
    signIn: (email: string, password: string) => Promise<AuthResponse>;
    signUp: (email: string, password: string) => Promise<AuthResponse>;
    signOut: () => Promise<void>;
    refreshSession: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

interface AuthProviderProps {
    children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
    const [state, setState] = useState<AuthState>({
        user: null,
        session: null,
        isLoading: true,
        isAuthenticated: false,
    });

    // Initialize auth state on mount
    useEffect(() => {
        const initAuth = async () => {
            const refreshTokenValue = loadRefreshToken();

            if (refreshTokenValue) {
                try {
                    const response = await api.post<{
                        access_token: string;
                        refresh_token: string;
                        user: User;
                    }>(`${config.endpoints.auth}/refresh`, {
                        refresh_token: refreshTokenValue,
                    });

                    if (response.data) {
                        setTokens(response.data.access_token, response.data.refresh_token);
                        setState({
                            user: response.data.user,
                            session: {
                                access_token: response.data.access_token,
                                refresh_token: response.data.refresh_token,
                                user: response.data.user,
                            },
                            isLoading: false,
                            isAuthenticated: true,
                        });
                        return;
                    }
                } catch {
                    clearTokens();
                }
            }

            setState((prev) => ({ ...prev, isLoading: false }));
        };

        initAuth();

        // Listen for logout events (e.g., from 401 interceptor)
        const handleLogout = () => {
            clearTokens();
            setState({
                user: null,
                session: null,
                isLoading: false,
                isAuthenticated: false,
            });
        };

        window.addEventListener("auth:logout", handleLogout);

        // Multi-tab sync
        const handleStorageChange = (e: StorageEvent) => {
            if (e.key === "aerodb_refresh_token" && !e.newValue) {
                handleLogout();
            }
        };

        window.addEventListener("storage", handleStorageChange);

        return () => {
            window.removeEventListener("auth:logout", handleLogout);
            window.removeEventListener("storage", handleStorageChange);
        };
    }, []);

    const signIn = useCallback(
        async (email: string, password: string): Promise<AuthResponse> => {
            const response = await api.post<{
                access_token: string;
                refresh_token: string;
                user: User;
            }>(`${config.endpoints.auth}/login`, { email, password });

            if (response.error) {
                return { data: null, error: response.error as ApiError };
            }

            if (response.data) {
                setTokens(response.data.access_token, response.data.refresh_token);
                const session: Session = {
                    access_token: response.data.access_token,
                    refresh_token: response.data.refresh_token,
                    user: response.data.user,
                };

                setState({
                    user: response.data.user,
                    session,
                    isLoading: false,
                    isAuthenticated: true,
                });

                return { data: { user: response.data.user, session }, error: null };
            }

            return { data: null, error: { message: "Unknown error" } };
        },
        []
    );

    const signUp = useCallback(
        async (email: string, password: string): Promise<AuthResponse> => {
            const response = await api.post<{
                access_token: string;
                refresh_token: string;
                user: User;
            }>(`${config.endpoints.auth}/signup`, { email, password });

            if (response.error) {
                return { data: null, error: response.error as ApiError };
            }

            if (response.data) {
                setTokens(response.data.access_token, response.data.refresh_token);
                const session: Session = {
                    access_token: response.data.access_token,
                    refresh_token: response.data.refresh_token,
                    user: response.data.user,
                };

                setState({
                    user: response.data.user,
                    session,
                    isLoading: false,
                    isAuthenticated: true,
                });

                return { data: { user: response.data.user, session }, error: null };
            }

            return { data: null, error: { message: "Unknown error" } };
        },
        []
    );

    const signOut = useCallback(async () => {
        try {
            await api.post(`${config.endpoints.auth}/logout`);
        } finally {
            clearTokens();
            setState({
                user: null,
                session: null,
                isLoading: false,
                isAuthenticated: false,
            });
        }
    }, []);

    const refreshSession = useCallback(async () => {
        const refreshTokenValue = loadRefreshToken();

        if (!refreshTokenValue) {
            throw new Error("No refresh token available");
        }

        const response = await api.post<{
            access_token: string;
            refresh_token: string;
            user: User;
        }>(`${config.endpoints.auth}/refresh`, {
            refresh_token: refreshTokenValue,
        });

        if (response.data) {
            setTokens(response.data.access_token, response.data.refresh_token);
            setState({
                user: response.data.user,
                session: {
                    access_token: response.data.access_token,
                    refresh_token: response.data.refresh_token,
                    user: response.data.user,
                },
                isLoading: false,
                isAuthenticated: true,
            });
        }
    }, []);

    const value: AuthContextValue = {
        ...state,
        signIn,
        signUp,
        signOut,
        refreshSession,
    };

    return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
    const context = useContext(AuthContext);
    if (!context) {
        throw new Error("useAuth must be used within an AuthProvider");
    }
    return context;
}
