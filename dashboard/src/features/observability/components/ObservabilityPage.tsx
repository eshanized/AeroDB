import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Header } from "@/components/layout/Header";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { api } from "@/lib/api/client";
import { config } from "@/config";
import { formatRelativeTime, formatNumber, formatDuration } from "@/lib/utils";
import type { LogEntry, Metric } from "@/types";
import {
    BarChart3,
    FileText,
    Search,
    Download,
    RefreshCw,
    AlertCircle,
    AlertTriangle,
    Info,
    Bug,
    Filter,
    Clock,
    Activity,
    Database,
    HardDrive,
} from "lucide-react";
import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
} from "recharts";

export function ObservabilityPage() {
    const [logLevel, setLogLevel] = useState<string | null>(null);
    const [logSearch, setLogSearch] = useState("");
    const [timeRange, setTimeRange] = useState("1h");

    // Fetch logs
    const logsQuery = useQuery({
        queryKey: ["logs", logLevel, timeRange],
        queryFn: async () => {
            const params: Record<string, string> = {};
            if (logLevel) params.level = logLevel;

            const response = await api.get<{ data: LogEntry[] }>(
                `${config.endpoints.observability}/logs`,
                params
            );
            if (response.error) throw new Error(response.error.message);
            return response.data?.data || [];
        },
    });

    // Fetch metrics
    const metricsQuery = useQuery({
        queryKey: ["metrics", timeRange],
        queryFn: async () => {
            const response = await api.get<{
                qps: Metric[];
                latency: { p50: number; p95: number; p99: number };
                error_rate: number;
                connections: number;
                storage_used: number;
            }>(`${config.endpoints.observability}/metrics`);
            if (response.error) throw new Error(response.error.message);
            return response.data;
        },
    });

    const filteredLogs = logsQuery.data?.filter((log) =>
        log.message.toLowerCase().includes(logSearch.toLowerCase())
    );

    const getLevelIcon = (level: string) => {
        switch (level) {
            case "ERROR":
                return <AlertCircle className="h-4 w-4 text-red-500" />;
            case "WARN":
                return <AlertTriangle className="h-4 w-4 text-amber-500" />;
            case "INFO":
                return <Info className="h-4 w-4 text-blue-500" />;
            case "DEBUG":
                return <Bug className="h-4 w-4 text-[hsl(var(--muted-foreground))]" />;
            default:
                return null;
        }
    };

    const getLevelColor = (level: string) => {
        switch (level) {
            case "ERROR":
                return "text-red-500 bg-red-500/10";
            case "WARN":
                return "text-amber-500 bg-amber-500/10";
            case "INFO":
                return "text-blue-500 bg-blue-500/10";
            case "DEBUG":
                return "text-[hsl(var(--muted-foreground))] bg-[hsl(var(--muted))]";
            default:
                return "";
        }
    };

    // Sample chart data (would come from API in real implementation)
    const chartData = metricsQuery.data?.qps || [
        { time: "00:00", qps: 120 },
        { time: "00:05", qps: 150 },
        { time: "00:10", qps: 180 },
        { time: "00:15", qps: 140 },
        { time: "00:20", qps: 200 },
        { time: "00:25", qps: 220 },
        { time: "00:30", qps: 190 },
    ];

    return (
        <div className="flex flex-col h-screen">
            <Header
                title="Observability"
                subtitle="Logs, metrics, and system health"
                onRefresh={() => {
                    logsQuery.refetch();
                    metricsQuery.refetch();
                }}
                isRefreshing={logsQuery.isFetching || metricsQuery.isFetching}
            />

            <div className="flex-1 p-6 overflow-auto">
                <div className="space-y-6">
                    {/* Metrics overview */}
                    <div className="grid grid-cols-4 gap-4">
                        <Card>
                            <CardContent className="pt-6">
                                <div className="flex items-center justify-between">
                                    <div>
                                        <p className="text-sm text-[hsl(var(--muted-foreground))]">
                                            Queries / sec
                                        </p>
                                        <p className="text-2xl font-bold">
                                            {formatNumber(metricsQuery.data?.qps?.length ?
                                                (metricsQuery.data.qps[metricsQuery.data.qps.length - 1] as unknown as { value: number }).value || 0 :
                                                185
                                            )}
                                        </p>
                                    </div>
                                    <Activity className="h-8 w-8 text-[hsl(var(--primary))] opacity-50" />
                                </div>
                            </CardContent>
                        </Card>

                        <Card>
                            <CardContent className="pt-6">
                                <div className="flex items-center justify-between">
                                    <div>
                                        <p className="text-sm text-[hsl(var(--muted-foreground))]">
                                            p95 Latency
                                        </p>
                                        <p className="text-2xl font-bold">
                                            {formatDuration(metricsQuery.data?.latency?.p95 || 45)}
                                        </p>
                                    </div>
                                    <Clock className="h-8 w-8 text-amber-500 opacity-50" />
                                </div>
                            </CardContent>
                        </Card>

                        <Card>
                            <CardContent className="pt-6">
                                <div className="flex items-center justify-between">
                                    <div>
                                        <p className="text-sm text-[hsl(var(--muted-foreground))]">
                                            Error Rate
                                        </p>
                                        <p className="text-2xl font-bold">
                                            {(metricsQuery.data?.error_rate || 0.2).toFixed(2)}%
                                        </p>
                                    </div>
                                    <AlertCircle className="h-8 w-8 text-red-500 opacity-50" />
                                </div>
                            </CardContent>
                        </Card>

                        <Card>
                            <CardContent className="pt-6">
                                <div className="flex items-center justify-between">
                                    <div>
                                        <p className="text-sm text-[hsl(var(--muted-foreground))]">
                                            Connections
                                        </p>
                                        <p className="text-2xl font-bold">
                                            {formatNumber(metricsQuery.data?.connections || 42)}
                                        </p>
                                    </div>
                                    <Database className="h-8 w-8 text-green-500 opacity-50" />
                                </div>
                            </CardContent>
                        </Card>
                    </div>

                    {/* Charts */}
                    <Card>
                        <CardHeader className="pb-2">
                            <div className="flex items-center justify-between">
                                <CardTitle className="text-base flex items-center gap-2">
                                    <BarChart3 className="h-4 w-4" />
                                    Queries per Second
                                </CardTitle>
                                <div className="flex items-center gap-2">
                                    {["1h", "6h", "24h", "7d"].map((range) => (
                                        <Button
                                            key={range}
                                            variant={timeRange === range ? "secondary" : "ghost"}
                                            size="sm"
                                            onClick={() => setTimeRange(range)}
                                        >
                                            {range}
                                        </Button>
                                    ))}
                                </div>
                            </div>
                        </CardHeader>
                        <CardContent>
                            <div className="h-64">
                                <ResponsiveContainer width="100%" height="100%">
                                    <LineChart data={chartData}>
                                        <CartesianGrid strokeDasharray="3 3" stroke="hsl(var(--border))" />
                                        <XAxis dataKey="time" stroke="hsl(var(--muted-foreground))" />
                                        <YAxis stroke="hsl(var(--muted-foreground))" />
                                        <Tooltip
                                            contentStyle={{
                                                backgroundColor: "hsl(var(--card))",
                                                border: "1px solid hsl(var(--border))",
                                                borderRadius: "8px",
                                            }}
                                        />
                                        <Line
                                            type="monotone"
                                            dataKey="qps"
                                            stroke="hsl(var(--primary))"
                                            strokeWidth={2}
                                            dot={false}
                                        />
                                    </LineChart>
                                </ResponsiveContainer>
                            </div>
                        </CardContent>
                    </Card>

                    {/* Logs */}
                    <Card>
                        <CardHeader className="pb-3">
                            <div className="flex items-center justify-between">
                                <CardTitle className="text-base flex items-center gap-2">
                                    <FileText className="h-4 w-4" />
                                    Logs
                                </CardTitle>
                                <div className="flex items-center gap-2">
                                    {/* Level filter */}
                                    <div className="flex items-center gap-1 border rounded-md p-1">
                                        {["ERROR", "WARN", "INFO", "DEBUG"].map((level) => (
                                            <Button
                                                key={level}
                                                variant={logLevel === level ? "secondary" : "ghost"}
                                                size="sm"
                                                className="h-7 px-2 text-xs"
                                                onClick={() =>
                                                    setLogLevel(logLevel === level ? null : level)
                                                }
                                            >
                                                {level}
                                            </Button>
                                        ))}
                                    </div>
                                    {/* Export */}
                                    <Button variant="outline" size="sm" className="gap-2">
                                        <Download className="h-4 w-4" />
                                        Export
                                    </Button>
                                </div>
                            </div>
                        </CardHeader>
                        <CardContent className="space-y-3">
                            {/* Search */}
                            <div className="relative">
                                <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-[hsl(var(--muted-foreground))]" />
                                <Input
                                    placeholder="Search logs..."
                                    className="pl-8"
                                    value={logSearch}
                                    onChange={(e) => setLogSearch(e.target.value)}
                                />
                            </div>

                            {/* Log entries */}
                            {logsQuery.isLoading ? (
                                <div className="flex items-center justify-center py-8">
                                    <RefreshCw className="h-5 w-5 animate-spin text-[hsl(var(--muted-foreground))]" />
                                </div>
                            ) : (
                                <div className="space-y-2 max-h-80 overflow-auto">
                                    {(filteredLogs || []).slice(0, 50).map((log) => (
                                        <div
                                            key={log.id}
                                            className="p-3 rounded-lg border text-sm font-mono"
                                        >
                                            <div className="flex items-start gap-2">
                                                <span className="text-[hsl(var(--muted-foreground))] shrink-0">
                                                    {new Date(log.timestamp).toLocaleTimeString()}
                                                </span>
                                                <span
                                                    className={`px-1.5 py-0.5 rounded text-xs shrink-0 ${getLevelColor(
                                                        log.level
                                                    )}`}
                                                >
                                                    {log.level}
                                                </span>
                                                <span className="text-[hsl(var(--muted-foreground))] shrink-0">
                                                    [{log.module}]
                                                </span>
                                                <span className="break-all">{log.message}</span>
                                            </div>
                                        </div>
                                    ))}
                                    {(!filteredLogs || filteredLogs.length === 0) && (
                                        <div className="text-center py-8 text-[hsl(var(--muted-foreground))]">
                                            <FileText className="h-8 w-8 mx-auto mb-2 opacity-50" />
                                            <p>No logs matching filters</p>
                                        </div>
                                    )}
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </div>
            </div>
        </div>
    );
}
