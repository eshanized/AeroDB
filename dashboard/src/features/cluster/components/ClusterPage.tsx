import { useQuery } from "@tanstack/react-query";
import { Header } from "@/components/layout/Header";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { api } from "@/lib/api/client";
import { config } from "@/config";
import { formatRelativeTime } from "@/lib/utils";
import type { ClusterNode } from "@/types";
import {
    Server,
    Crown,
    Copy,
    Circle,
    RefreshCw,
    AlertCircle,
    ArrowRight,
} from "lucide-react";

export function ClusterPage() {
    // Fetch cluster nodes
    const nodesQuery = useQuery({
        queryKey: ["cluster-nodes"],
        queryFn: async () => {
            const response = await api.get<{ data: ClusterNode[] }>(
                `${config.endpoints.control}/cluster/nodes`
            );
            if (response.error) throw new Error(response.error.message);
            return response.data?.data || [];
        },
    });

    const authority = nodesQuery.data?.find((n) => n.role === "authority");
    const replicas = nodesQuery.data?.filter((n) => n.role === "replica") || [];

    const getStatusColor = (status: string) => {
        switch (status) {
            case "healthy":
                return "text-green-500 fill-green-500";
            case "degraded":
                return "text-amber-500 fill-amber-500";
            case "offline":
                return "text-red-500 fill-red-500";
            default:
                return "text-[hsl(var(--muted-foreground))]";
        }
    };

    const getStatusBg = (status: string) => {
        switch (status) {
            case "healthy":
                return "border-green-500/50 bg-green-500/5";
            case "degraded":
                return "border-amber-500/50 bg-amber-500/5";
            case "offline":
                return "border-red-500/50 bg-red-500/5";
            default:
                return "";
        }
    };

    return (
        <div className="flex flex-col h-screen">
            <Header
                title="Cluster"
                subtitle="Topology and replication status"
                onRefresh={() => nodesQuery.refetch()}
                isRefreshing={nodesQuery.isFetching}
            />

            <div className="flex-1 p-6 overflow-auto">
                {nodesQuery.isLoading ? (
                    <div className="flex items-center justify-center py-24">
                        <RefreshCw className="h-8 w-8 animate-spin text-[hsl(var(--muted-foreground))]" />
                    </div>
                ) : nodesQuery.error ? (
                    <div className="flex flex-col items-center justify-center py-24 text-[hsl(var(--destructive))]">
                        <AlertCircle className="h-12 w-12 mb-4" />
                        <p>{(nodesQuery.error as Error).message}</p>
                    </div>
                ) : (
                    <div className="space-y-8">
                        {/* Cluster topology visualization */}
                        <Card>
                            <CardHeader>
                                <CardTitle className="text-base">Cluster Topology</CardTitle>
                            </CardHeader>
                            <CardContent>
                                <div className="flex items-center justify-center gap-8 py-8">
                                    {/* Authority node */}
                                    {authority && (
                                        <div
                                            className={`p-6 rounded-xl border-2 ${getStatusBg(
                                                authority.status
                                            )}`}
                                        >
                                            <div className="flex items-center gap-2 mb-3">
                                                <Crown className="h-5 w-5 text-amber-500" />
                                                <span className="font-semibold">Authority</span>
                                            </div>
                                            <div className="flex items-center gap-2 mb-2">
                                                <Server className="h-4 w-4 text-[hsl(var(--muted-foreground))]" />
                                                <span className="text-sm font-mono">
                                                    {authority.host}:{authority.port}
                                                </span>
                                            </div>
                                            <div className="flex items-center gap-2">
                                                <Circle className={`h-3 w-3 ${getStatusColor(authority.status)}`} />
                                                <span className="text-sm capitalize">{authority.status}</span>
                                            </div>
                                        </div>
                                    )}

                                    {/* Replication arrows */}
                                    {replicas.length > 0 && (
                                        <div className="flex flex-col items-center gap-2">
                                            {replicas.map((_, i) => (
                                                <ArrowRight
                                                    key={i}
                                                    className="h-6 w-6 text-[hsl(var(--muted-foreground))]"
                                                />
                                            ))}
                                        </div>
                                    )}

                                    {/* Replica nodes */}
                                    <div className="flex flex-col gap-4">
                                        {replicas.map((replica) => (
                                            <div
                                                key={replica.id}
                                                className={`p-4 rounded-xl border-2 ${getStatusBg(
                                                    replica.status
                                                )}`}
                                            >
                                                <div className="flex items-center gap-2 mb-2">
                                                    <Copy className="h-4 w-4 text-blue-500" />
                                                    <span className="font-medium">Replica</span>
                                                </div>
                                                <div className="flex items-center gap-2 mb-1">
                                                    <Server className="h-4 w-4 text-[hsl(var(--muted-foreground))]" />
                                                    <span className="text-sm font-mono">
                                                        {replica.host}:{replica.port}
                                                    </span>
                                                </div>
                                                <div className="flex items-center justify-between">
                                                    <div className="flex items-center gap-2">
                                                        <Circle className={`h-3 w-3 ${getStatusColor(replica.status)}`} />
                                                        <span className="text-sm capitalize">{replica.status}</span>
                                                    </div>
                                                    {replica.lag_ms !== undefined && (
                                                        <span className="text-xs text-[hsl(var(--muted-foreground))]">
                                                            {replica.lag_ms}ms lag
                                                        </span>
                                                    )}
                                                </div>
                                            </div>
                                        ))}
                                        {replicas.length === 0 && (
                                            <div className="p-4 rounded-xl border-2 border-dashed text-[hsl(var(--muted-foreground))]">
                                                <p className="text-sm">No replicas configured</p>
                                            </div>
                                        )}
                                    </div>
                                </div>
                            </CardContent>
                        </Card>

                        {/* Node details */}
                        <div className="grid grid-cols-3 gap-4">
                            {nodesQuery.data?.map((node) => (
                                <Card key={node.id}>
                                    <CardHeader className="pb-2">
                                        <div className="flex items-center justify-between">
                                            <CardTitle className="text-sm flex items-center gap-2">
                                                {node.role === "authority" ? (
                                                    <Crown className="h-4 w-4 text-amber-500" />
                                                ) : (
                                                    <Copy className="h-4 w-4 text-blue-500" />
                                                )}
                                                {node.role === "authority" ? "Authority" : "Replica"}
                                            </CardTitle>
                                            <Circle className={`h-3 w-3 ${getStatusColor(node.status)}`} />
                                        </div>
                                    </CardHeader>
                                    <CardContent className="space-y-2 text-sm">
                                        <div className="flex justify-between">
                                            <span className="text-[hsl(var(--muted-foreground))]">Host</span>
                                            <span className="font-mono">{node.host}</span>
                                        </div>
                                        <div className="flex justify-between">
                                            <span className="text-[hsl(var(--muted-foreground))]">Port</span>
                                            <span className="font-mono">{node.port}</span>
                                        </div>
                                        <div className="flex justify-between">
                                            <span className="text-[hsl(var(--muted-foreground))]">Status</span>
                                            <span className="capitalize">{node.status}</span>
                                        </div>
                                        {node.lag_ms !== undefined && (
                                            <div className="flex justify-between">
                                                <span className="text-[hsl(var(--muted-foreground))]">Lag</span>
                                                <span>{node.lag_ms}ms</span>
                                            </div>
                                        )}
                                        <div className="flex justify-between">
                                            <span className="text-[hsl(var(--muted-foreground))]">
                                                Last Heartbeat
                                            </span>
                                            <span>{formatRelativeTime(node.last_heartbeat)}</span>
                                        </div>
                                    </CardContent>
                                </Card>
                            ))}
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
