import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Header } from "@/components/layout/Header";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { api } from "@/lib/api/client";
import { config } from "@/config";
import { formatNumber, formatDuration } from "@/lib/utils";
import type { LogEntry, Metric } from "@/types";
import {
    BarChart3,
    FileText,
    Search,
    Download,
    AlertCircle,
    Clock,
    Activity,
    Database,
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
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
    Badge,
    Skeleton,
} from "@/components/ui";

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

    const getLevelBadgeVariant = (level: string) => {
        switch (level) {
            case "ERROR":
                return "destructive";
            case "WARN":
                return "default";
            case "INFO":
                return "secondary";
            case "DEBUG":
                return "outline";
            default:
                return "outline";
        }
    };

    const getLevelClassName = (level: string) => {
        switch (level) {
            case "WARN":
                return "bg-amber-500/15 text-amber-600 hover:bg-amber-500/25 border-amber-500/50";
            case "INFO":
                return "bg-blue-500/15 text-blue-600 hover:bg-blue-500/25 border-blue-500/50";
            default:
                return "";
        }
    }

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
                                        <p className="text-sm text-muted-foreground">
                                            Queries / sec
                                        </p>
                                        <p className="text-2xl font-bold">
                                            {formatNumber(metricsQuery.data?.qps?.length ?
                                                (metricsQuery.data.qps[metricsQuery.data.qps.length - 1] as unknown as { value: number }).value || 0 :
                                                185
                                            )}
                                        </p>
                                    </div>
                                    <Activity className="h-8 w-8 text-primary opacity-50" />
                                </div>
                            </CardContent>
                        </Card>

                        <Card>
                            <CardContent className="pt-6">
                                <div className="flex items-center justify-between">
                                    <div>
                                        <p className="text-sm text-muted-foreground">
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
                                        <p className="text-sm text-muted-foreground">
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
                                        <p className="text-sm text-muted-foreground">
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
                                <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                                <Input
                                    placeholder="Search logs..."
                                    className="pl-8"
                                    value={logSearch}
                                    onChange={(e) => setLogSearch(e.target.value)}
                                />
                            </div>

                            {/* Log entries */}
                            {logsQuery.isLoading ? (
                                <div className="space-y-2">
                                    <Skeleton className="h-10 w-full" />
                                    <Skeleton className="h-10 w-full" />
                                    <Skeleton className="h-10 w-full" />
                                </div>
                            ) : (
                                <div className="border rounded-lg overflow-hidden">
                                    <Table>
                                        <TableHeader>
                                            <TableRow>
                                                <TableHead className="w-[180px]">Timestamp</TableHead>
                                                <TableHead className="w-[100px]">Level</TableHead>
                                                <TableHead className="w-[150px]">Module</TableHead>
                                                <TableHead>Message</TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            {(filteredLogs || []).slice(0, 50).map((log) => (
                                                <TableRow key={log.id} className="font-mono text-xs">
                                                    <TableCell className="whitespace-nowrap text-muted-foreground">
                                                        {new Date(log.timestamp).toLocaleTimeString()}
                                                    </TableCell>
                                                    <TableCell>
                                                        <Badge
                                                            variant={getLevelBadgeVariant(log.level) as any}
                                                            className={getLevelClassName(log.level)}
                                                        >
                                                            {log.level}
                                                        </Badge>
                                                    </TableCell>
                                                    <TableCell className="text-muted-foreground">
                                                        [{log.module}]
                                                    </TableCell>
                                                    <TableCell className="break-all">
                                                        {log.message}
                                                    </TableCell>
                                                </TableRow>
                                            ))}
                                            {(!filteredLogs || filteredLogs.length === 0) && (
                                                <TableRow>
                                                    <TableCell colSpan={4} className="h-24 text-center">
                                                        <div className="flex flex-col items-center justify-center text-muted-foreground">
                                                            <FileText className="h-8 w-8 mb-2 opacity-50" />
                                                            <p>No logs matching filters</p>
                                                        </div>
                                                    </TableCell>
                                                </TableRow>
                                            )}
                                        </TableBody>
                                    </Table>
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </div>
            </div>
        </div>
    );
}
