import axios, { AxiosError, AxiosInstance, AxiosRequestConfig } from "axios";
import { config } from "@/config";
import type { ApiError, ApiResponse } from "@/types";

// Token storage (in-memory for security)
let accessToken: string | null = null;
let refreshToken: string | null = null;

/**
 * Set authentication tokens
 */
export function setTokens(access: string, refresh?: string) {
    accessToken = access;
    if (refresh) refreshToken = refresh;

    // Store in localStorage for persistence (refresh token only)
    if (refresh) {
        localStorage.setItem("aerodb_refresh_token", refresh);
    }
}

/**
 * Clear authentication tokens
 */
export function clearTokens() {
    accessToken = null;
    refreshToken = null;
    localStorage.removeItem("aerodb_refresh_token");
}

/**
 * Get the current access token
 */
export function getAccessToken(): string | null {
    return accessToken;
}

/**
 * Load refresh token from localStorage
 */
export function loadRefreshToken(): string | null {
    if (!refreshToken) {
        refreshToken = localStorage.getItem("aerodb_refresh_token");
    }
    return refreshToken;
}

/**
 * Create axios instance with interceptors
 */
function createApiClient(): AxiosInstance {
    const instance = axios.create({
        baseURL: config.apiUrl,
        timeout: config.timeouts.default,
        headers: {
            "Content-Type": "application/json",
        },
    });

    // Request interceptor - add auth token
    instance.interceptors.request.use(
        (reqConfig) => {
            if (accessToken) {
                reqConfig.headers.Authorization = `Bearer ${accessToken}`;
            }

            // Dev mode logging (never log tokens)
            if (import.meta.env.DEV) {
                console.log(
                    `[API] ${reqConfig.method?.toUpperCase()} ${reqConfig.url}`
                );
            }

            return reqConfig;
        },
        (error) => Promise.reject(error)
    );

    // Response interceptor - handle errors
    instance.interceptors.response.use(
        (response) => response,
        async (error: AxiosError) => {
            const originalRequest = error.config as AxiosRequestConfig & {
                _retry?: boolean;
            };

            // Handle 401 - try to refresh token
            if (error.response?.status === 401 && !originalRequest._retry) {
                originalRequest._retry = true;

                const storedRefresh = loadRefreshToken();
                if (storedRefresh) {
                    try {
                        const response = await axios.post(
                            `${config.apiUrl}${config.endpoints.auth}/refresh`,
                            { refresh_token: storedRefresh }
                        );

                        const { access_token, refresh_token } = response.data;
                        setTokens(access_token, refresh_token);

                        // Retry original request
                        if (originalRequest.headers) {
                            originalRequest.headers.Authorization = `Bearer ${access_token}`;
                        }
                        return instance(originalRequest);
                    } catch {
                        // Refresh failed - clear tokens and reject
                        clearTokens();
                        window.dispatchEvent(new CustomEvent("auth:logout"));
                        return Promise.reject(error);
                    }
                }
            }

            return Promise.reject(error);
        }
    );

    return instance;
}

// Create the API client instance
export const apiClient = createApiClient();

/**
 * Transform axios error to API error
 */
export function toApiError(error: unknown): ApiError {
    if (axios.isAxiosError(error)) {
        const axiosError = error as AxiosError<{ error?: string; message?: string }>;
        return {
            message:
                axiosError.response?.data?.error ||
                axiosError.response?.data?.message ||
                axiosError.message ||
                "An error occurred",
            status: axiosError.response?.status,
            code: axiosError.code,
        };
    }

    if (error instanceof Error) {
        return { message: error.message };
    }

    return { message: "Unknown error" };
}

/**
 * Make a GET request
 */
export async function get<T>(
    url: string,
    params?: Record<string, unknown>
): Promise<ApiResponse<T>> {
    try {
        const response = await apiClient.get<T>(url, { params });
        return { data: response.data, error: null };
    } catch (error) {
        return { data: null, error: toApiError(error) };
    }
}

/**
 * Make a POST request
 */
export async function post<T>(
    url: string,
    data?: unknown
): Promise<ApiResponse<T>> {
    try {
        const response = await apiClient.post<T>(url, data);
        return { data: response.data, error: null };
    } catch (error) {
        return { data: null, error: toApiError(error) };
    }
}

/**
 * Make a PUT request
 */
export async function put<T>(
    url: string,
    data?: unknown
): Promise<ApiResponse<T>> {
    try {
        const response = await apiClient.put<T>(url, data);
        return { data: response.data, error: null };
    } catch (error) {
        return { data: null, error: toApiError(error) };
    }
}

/**
 * Make a DELETE request
 */
export async function del<T>(url: string): Promise<ApiResponse<T>> {
    try {
        const response = await apiClient.delete<T>(url);
        return { data: response.data, error: null };
    } catch (error) {
        return { data: null, error: toApiError(error) };
    }
}

// Export all functions
export const api = {
    get,
    post,
    put,
    delete: del,
    client: apiClient,
    setTokens,
    clearTokens,
    getAccessToken,
};
