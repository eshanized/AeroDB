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
import {
    Avatar,
    AvatarFallback,
    Badge,
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogFooter,
    DialogDescription,
} from "@/components/ui";

export function AuthPage() {
    const [searchQuery, setSearchQuery] = useState("");
    const [showAddModal, setShowAddModal] = useState(false);
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
                                    <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
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
                                        <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
                                    </div>
                                ) : usersQuery.error ? (
                                    <div className="flex flex-col items-center justify-center py-12 text-destructive">
                                        <AlertCircle className="h-12 w-12 mb-4" />
                                        <p>{(usersQuery.error as Error).message}</p>
                                    </div>
                                ) : (
                                    <div className="border rounded-lg overflow-hidden">
                                        <Table>
                                            <TableHeader>
                                                <TableRow>
                                                    <TableHead>Email</TableHead>
                                                    <TableHead>Role</TableHead>
                                                    <TableHead>Created</TableHead>
                                                    <TableHead>Last Login</TableHead>
                                                    <TableHead className="text-right">Actions</TableHead>
                                                </TableRow>
                                            </TableHeader>
                                            <TableBody>
                                                {filteredUsers.map((user) => (
                                                    <TableRow key={user.id}>
                                                        <TableCell className="font-medium">
                                                            <div className="flex items-center gap-3">
                                                                <Avatar className="h-8 w-8">
                                                                    <AvatarFallback className="bg-primary/10 text-primary">
                                                                        {user.email.substring(0, 2).toUpperCase()}
                                                                    </AvatarFallback>
                                                                </Avatar>
                                                                {user.email}
                                                            </div>
                                                        </TableCell>
                                                        <TableCell>
                                                            <Badge variant={user.role === 'admin' ? 'destructive' : 'secondary'} className="gap-1">
                                                                <Shield className="h-3 w-3" />
                                                                {user.role || "user"}
                                                            </Badge>
                                                        </TableCell>
                                                        <TableCell className="text-muted-foreground">
                                                            {formatRelativeTime(user.created_at)}
                                                        </TableCell>
                                                        <TableCell className="text-muted-foreground">
                                                            {user.last_login ? formatRelativeTime(user.last_login) : "Never"}
                                                        </TableCell>
                                                        <TableCell className="text-right">
                                                            <div className="flex items-center justify-end gap-1">
                                                                <Button variant="ghost" size="icon" className="h-8 w-8">
                                                                    <Edit className="h-4 w-4" />
                                                                </Button>
                                                                <Button
                                                                    variant="ghost"
                                                                    size="icon"
                                                                    className="h-8 w-8 text-destructive"
                                                                    onClick={() => setDeleteConfirm(user.id)}
                                                                >
                                                                    <Trash2 className="h-4 w-4" />
                                                                </Button>
                                                            </div>
                                                        </TableCell>
                                                    </TableRow>
                                                ))}
                                            </TableBody>
                                        </Table>
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
                                    <span className="text-muted-foreground">Total Users</span>
                                    <span className="font-medium">{usersQuery.data?.length || 0}</span>
                                </div>
                                <div className="flex justify-between">
                                    <span className="text-muted-foreground">Active Today</span>
                                    <span className="font-medium">-</span>
                                </div>
                                <div className="flex justify-between">
                                    <span className="text-muted-foreground">Active Sessions</span>
                                    <span className="font-medium">-</span>
                                </div>
                            </CardContent>
                        </Card>

                        <Card>
                            <CardHeader className="pb-2">
                                <CardTitle className="text-base">RLS Policies</CardTitle>
                            </CardHeader>
                            <CardContent>
                                <p className="text-sm text-muted-foreground">
                                    Row-Level Security policies are configured in the database schema.
                                    View via SQL console.
                                </p>
                            </CardContent>
                        </Card>
                    </div>
                </div>

                {/* Add User Dialog */}
                <Dialog open={showAddModal} onOpenChange={setShowAddModal}>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>Add New User</DialogTitle>
                            <DialogDescription>
                                Create a new user account. They will receive an email to set their password.
                            </DialogDescription>
                        </DialogHeader>
                        <div className="space-y-4 py-4">
                            <div className="space-y-2">
                                <label className="text-sm font-medium">Email</label>
                                <Input placeholder="user@example.com" type="email" />
                            </div>
                            <div className="space-y-2">
                                <label className="text-sm font-medium">Role</label>
                                <select className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50">
                                    <option value="user">User</option>
                                    <option value="admin">Admin</option>
                                </select>
                            </div>
                        </div>
                        <DialogFooter>
                            <Button variant="outline" onClick={() => setShowAddModal(false)}>
                                Cancel
                            </Button>
                            <Button onClick={() => setShowAddModal(false)}>
                                Create User
                            </Button>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>

                {/* Delete confirmation Dialog */}
                <Dialog open={!!deleteConfirm} onOpenChange={(open) => !open && setDeleteConfirm(null)}>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle className="flex items-center gap-2 text-destructive">
                                <AlertCircle className="h-5 w-5" />
                                Confirm Delete
                            </DialogTitle>
                            <DialogDescription>
                                Are you sure you want to delete this user? This action cannot be undone.
                            </DialogDescription>
                        </DialogHeader>
                        <DialogFooter>
                            <Button variant="outline" onClick={() => setDeleteConfirm(null)}>
                                Cancel
                            </Button>
                            <Button
                                variant="destructive"
                                onClick={() => deleteConfirm && deleteMutation.mutate(deleteConfirm)}
                                disabled={deleteMutation.isPending}
                            >
                                {deleteMutation.isPending ? "Deleting..." : "Delete"}
                            </Button>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            </div>
        </div>
    );
}
