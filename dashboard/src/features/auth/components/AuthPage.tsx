import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Header } from "@/components/layout/Header";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { api } from "@/lib/api/client";
import { config } from "@/config";
import { formatRelativeTime } from "@/lib/utils";
import type { User } from "@/types";
import {
    Users,
    Plus,
    Trash2,
    Edit,
    AlertCircle,
    RefreshCw,
    Search,
    Shield,
} from "lucide-react";

export function AuthPage() {
    const [searchQuery, setSearchQuery] = useState("");
    const [_showAddModal, setShowAddModal] = useState(false);
    const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);
    const queryClient = useQueryClient();

    // Fetch users
    const usersQuery = useQuery({
        queryKey: ["users"],
        queryFn: async () => {
            const response = await api.get<{ data: User[] }>(
                `${config.endpoints.rest}/users`,
                { select: "id,email,created_at,last_login,role" }
            );
            if (response.error) throw new Error(response.error.message);
            return response.data?.data || [];
        },
    });

    // Delete user mutation
    const deleteMutation = useMutation({
        mutationFn: async (userId: string) => {
            const response = await api.delete(`${config.endpoints.rest}/users/${userId}`);
            if (response.error) throw new Error(response.error.message);
            return response.data;
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["users"] });
            setDeleteConfirm(null);
        },
    });

    const filteredUsers = usersQuery.data?.filter((user) =>
        user.email.toLowerCase().includes(searchQuery.toLowerCase())
    ) || [];

    return (
        <div className="flex flex-col h-screen">
            <Header
                title="Authentication"
                subtitle="Manage users and sessions"
                onRefresh={() => usersQuery.refetch()}
                isRefreshing={usersQuery.isFetching}
            />

            <div className="flex-1 p-6 overflow-auto">
                <div className="grid grid-cols-12 gap-6">
                    {/* Users section */}
                    <div className="col-span-8">
                        <Card>
                            <CardHeader className="pb-3">
                                <div className="flex items-center justify-between">
                                    <CardTitle className="text-base flex items-center gap-2">
                                        <Users className="h-4 w-4" />
                                        Users
                                    </CardTitle>
                                    <Button size="sm" className="gap-2" onClick={() => setShowAddModal(true)}>
                                        <Plus className="h-4 w-4" />
                                        Add User
                                    </Button>
                                </div>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                {/* Search */}
                                <div className="relative">
                                    <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-[hsl(var(--muted-foreground))]" />
                                    <Input
                                        placeholder="Search users..."
                                        className="pl-8"
                                        value={searchQuery}
                                        onChange={(e) => setSearchQuery(e.target.value)}
                                    />
                                </div>

                                {/* Users table */}
                                {usersQuery.isLoading ? (
                                    <div className="flex items-center justify-center py-12">
                                        <RefreshCw className="h-6 w-6 animate-spin text-[hsl(var(--muted-foreground))]" />
                                    </div>
                                ) : usersQuery.error ? (
                                    <div className="flex flex-col items-center justify-center py-12 text-[hsl(var(--destructive))]">
                                        <AlertCircle className="h-12 w-12 mb-4" />
                                        <p>{(usersQuery.error as Error).message}</p>
                                    </div>
                                ) : (
                                    <div className="border rounded-lg overflow-hidden">
                                        <table className="w-full text-sm">
                                            <thead className="border-b bg-[hsl(var(--muted))]">
                                                <tr>
                                                    <th className="text-left px-4 py-3 font-medium text-[hsl(var(--muted-foreground))]">
                                                        Email
                                                    </th>
                                                    <th className="text-left px-4 py-3 font-medium text-[hsl(var(--muted-foreground))]">
                                                        Role
                                                    </th>
                                                    <th className="text-left px-4 py-3 font-medium text-[hsl(var(--muted-foreground))]">
                                                        Created
                                                    </th>
                                                    <th className="text-left px-4 py-3 font-medium text-[hsl(var(--muted-foreground))]">
                                                        Last Login
                                                    </th>
                                                    <th className="text-right px-4 py-3 font-medium text-[hsl(var(--muted-foreground))]">
                                                        Actions
                                                    </th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {filteredUsers.map((user) => (
                                                    <tr key={user.id} className="border-b hover:bg-[hsl(var(--muted))]/50">
                                                        <td className="px-4 py-3 font-medium">{user.email}</td>
                                                        <td className="px-4 py-3">
                                                            <span className="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs bg-[hsl(var(--primary))]/10 text-[hsl(var(--primary))]">
                                                                <Shield className="h-3 w-3" />
                                                                {user.role || "user"}
                                                            </span>
                                                        </td>
                                                        <td className="px-4 py-3 text-[hsl(var(--muted-foreground))]">
                                                            {formatRelativeTime(user.created_at)}
                                                        </td>
                                                        <td className="px-4 py-3 text-[hsl(var(--muted-foreground))]">
                                                            {user.last_login ? formatRelativeTime(user.last_login) : "Never"}
                                                        </td>
                                                        <td className="px-4 py-3 text-right">
                                                            <div className="flex items-center justify-end gap-1">
                                                                <Button variant="ghost" size="icon" className="h-8 w-8">
                                                                    <Edit className="h-4 w-4" />
                                                                </Button>
                                                                <Button
                                                                    variant="ghost"
                                                                    size="icon"
                                                                    className="h-8 w-8 text-[hsl(var(--destructive))]"
                                                                    onClick={() => setDeleteConfirm(user.id)}
                                                                >
                                                                    <Trash2 className="h-4 w-4" />
                                                                </Button>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                ))}
                                            </tbody>
                                        </table>
                                    </div>
                                )}
                            </CardContent>
                        </Card>
                    </div>

                    {/* Stats sidebar */}
                    <div className="col-span-4 space-y-4">
                        <Card>
                            <CardHeader className="pb-2">
                                <CardTitle className="text-base">Statistics</CardTitle>
                            </CardHeader>
                            <CardContent className="space-y-3">
                                <div className="flex justify-between">
                                    <span className="text-[hsl(var(--muted-foreground))]">Total Users</span>
                                    <span className="font-medium">{usersQuery.data?.length || 0}</span>
                                </div>
                                <div className="flex justify-between">
                                    <span className="text-[hsl(var(--muted-foreground))]">Active Today</span>
                                    <span className="font-medium">-</span>
                                </div>
                                <div className="flex justify-between">
                                    <span className="text-[hsl(var(--muted-foreground))]">Active Sessions</span>
                                    <span className="font-medium">-</span>
                                </div>
                            </CardContent>
                        </Card>

                        <Card>
                            <CardHeader className="pb-2">
                                <CardTitle className="text-base">RLS Policies</CardTitle>
                            </CardHeader>
                            <CardContent>
                                <p className="text-sm text-[hsl(var(--muted-foreground))]">
                                    Row-Level Security policies are configured in the database schema.
                                    View via SQL console.
                                </p>
                            </CardContent>
                        </Card>
                    </div>
                </div>

                {/* Delete confirmation modal */}
                {deleteConfirm && (
                    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
                        <Card className="w-full max-w-md">
                            <CardHeader>
                                <CardTitle className="text-lg flex items-center gap-2 text-[hsl(var(--destructive))]">
                                    <AlertCircle className="h-5 w-5" />
                                    Confirm Delete
                                </CardTitle>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <p>
                                    Are you sure you want to delete this user? This action cannot be undone.
                                </p>
                                <div className="flex justify-end gap-2">
                                    <Button variant="outline" onClick={() => setDeleteConfirm(null)}>
                                        Cancel
                                    </Button>
                                    <Button
                                        variant="destructive"
                                        onClick={() => deleteMutation.mutate(deleteConfirm)}
                                        disabled={deleteMutation.isPending}
                                    >
                                        {deleteMutation.isPending ? "Deleting..." : "Delete"}
                                    </Button>
                                </div>
                            </CardContent>
                        </Card>
                    </div>
                )}
            </div>
        </div>
    );
}
