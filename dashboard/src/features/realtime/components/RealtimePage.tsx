import { useState, useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { Header } from "@/components/layout/Header";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/api/client";
import { config } from "@/config";
import { formatRelativeTime } from "@/lib/utils";
import type { Subscription, RealtimeEvent } from "@/types";
import {
    Activity,
    Radio,
    Zap,
    Circle,
    RefreshCw,
    AlertCircle,
    Filter,
    Play,
    Pause,
    Trash2,
} from "lucide-react";

export function RealtimePage() {
    const [isLive, setIsLive] = useState(false);
    const [events, setEvents] = useState<RealtimeEvent[]>([]);
    const [filterType, setFilterType] = useState<string | null>(null);
    const wsRef = useRef<WebSocket | null>(null);

    // Fetch active subscriptions
    const subscriptionsQuery = useQuery({
        queryKey: ["subscriptions"],
        queryFn: async () => {
            const response = await api.get<{ data: Subscription[] }>(
                `${config.endpoints.realtime}/subscriptions`
            );
            if (response.error) throw new Error(response.error.message);
            return response.data?.data || [];
        },
    });

    // WebSocket connection for live events
    useEffect(() => {
        if (!isLive) {
            wsRef.current?.close();
            return;
        }

        const ws = new WebSocket(config.wsUrl);

        ws.onopen = () => {
            ws.send(JSON.stringify({ type: "subscribe", channel: "*" }));
        };

        ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data) as RealtimeEvent;
                setEvents((prev) => [data, ...prev].slice(0, 100)); // Keep last 100
            } catch {
                console.error("Failed to parse WebSocket message");
            }
        };

        ws.onerror = () => {
            console.error("WebSocket error");
            setIsLive(false);
        };

        wsRef.current = ws;

        return () => {
            ws.close();
        };
    }, [isLive]);

    const filteredEvents = filterType
        ? events.filter((e) => e.type === filterType)
        : events;

    const getEventColor = (type: string) => {
        switch (type) {
            case "INSERT":
                return "text-green-500 bg-green-500/10";
            case "UPDATE":
                return "text-blue-500 bg-blue-500/10";
            case "DELETE":
                return "text-red-500 bg-red-500/10";
            default:
                return "text-[hsl(var(--muted-foreground))] bg-[hsl(var(--muted))]";
        }
    };

    return (
        <div className="flex flex-col h-screen">
            <Header
                title="Real-time"
                subtitle="Active subscriptions and live events"
                onRefresh={() => subscriptionsQuery.refetch()}
                isRefreshing={subscriptionsQuery.isFetching}
            />

            <div className="flex-1 p-6 overflow-auto">
                <div className="grid grid-cols-12 gap-6">
                    {/* Subscriptions */}
                    <div className="col-span-5">
                        <Card>
                            <CardHeader className="pb-3">
                                <div className="flex items-center justify-between">
                                    <CardTitle className="text-base flex items-center gap-2">
                                        <Radio className="h-4 w-4" />
                                        Active Subscriptions
                                    </CardTitle>
                                    <div className="flex items-center gap-2">
                                        <Circle
                                            className={`h-2 w-2 ${subscriptionsQuery.data?.length
                                                    ? "fill-green-500 text-green-500"
                                                    : "fill-[hsl(var(--muted-foreground))] text-[hsl(var(--muted-foreground))]"
                                                }`}
                                        />
                                        <span className="text-sm text-[hsl(var(--muted-foreground))]">
                                            {subscriptionsQuery.data?.length || 0} active
                                        </span>
                                    </div>
                                </div>
                            </CardHeader>
                            <CardContent>
                                {subscriptionsQuery.isLoading ? (
                                    <div className="flex items-center justify-center py-8">
                                        <RefreshCw className="h-5 w-5 animate-spin text-[hsl(var(--muted-foreground))]" />
                                    </div>
                                ) : subscriptionsQuery.data?.length === 0 ? (
                                    <div className="text-center py-8 text-[hsl(var(--muted-foreground))]">
                                        <Radio className="h-8 w-8 mx-auto mb-2 opacity-50" />
                                        <p>No active subscriptions</p>
                                    </div>
                                ) : (
                                    <div className="space-y-2">
                                        {subscriptionsQuery.data?.map((sub) => (
                                            <div
                                                key={sub.id}
                                                className="p-3 rounded-lg border bg-[hsl(var(--muted))]/50"
                                            >
                                                <div className="flex items-center justify-between mb-1">
                                                    <span className="font-medium text-sm">{sub.channel}</span>
                                                    <span className="text-xs text-[hsl(var(--muted-foreground))]">
                                                        {formatRelativeTime(sub.connected_at)}
                                                    </span>
                                                </div>
                                                <div className="text-xs text-[hsl(var(--muted-foreground))]">
                                                    User: {sub.user_id.slice(0, 8)}...
                                                    {sub.filter && <span className="ml-2">Filter: {sub.filter}</span>}
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                )}
                            </CardContent>
                        </Card>
                    </div>

                    {/* Event log */}
                    <div className="col-span-7">
                        <Card>
                            <CardHeader className="pb-3">
                                <div className="flex items-center justify-between">
                                    <CardTitle className="text-base flex items-center gap-2">
                                        <Zap className="h-4 w-4" />
                                        Event Log
                                    </CardTitle>
                                    <div className="flex items-center gap-2">
                                        {/* Filter buttons */}
                                        <div className="flex items-center gap-1 border rounded-md p-1">
                                            {["INSERT", "UPDATE", "DELETE"].map((type) => (
                                                <Button
                                                    key={type}
                                                    variant={filterType === type ? "secondary" : "ghost"}
                                                    size="sm"
                                                    className="h-7 px-2 text-xs"
                                                    onClick={() =>
                                                        setFilterType(filterType === type ? null : type)
                                                    }
                                                >
                                                    {type}
                                                </Button>
                                            ))}
                                        </div>
                                        {/* Live toggle */}
                                        <Button
                                            variant={isLive ? "destructive" : "default"}
                                            size="sm"
                                            className="gap-2"
                                            onClick={() => setIsLive(!isLive)}
                                        >
                                            {isLive ? (
                                                <>
                                                    <Pause className="h-4 w-4" />
                                                    Stop
                                                </>
                                            ) : (
                                                <>
                                                    <Play className="h-4 w-4" />
                                                    Live
                                                </>
                                            )}
                                        </Button>
                                        {/* Clear */}
                                        <Button
                                            variant="outline"
                                            size="icon"
                                            className="h-8 w-8"
                                            onClick={() => setEvents([])}
                                        >
                                            <Trash2 className="h-4 w-4" />
                                        </Button>
                                    </div>
                                </div>
                            </CardHeader>
                            <CardContent>
                                {filteredEvents.length === 0 ? (
                                    <div className="text-center py-12 text-[hsl(var(--muted-foreground))]">
                                        <Activity className="h-8 w-8 mx-auto mb-2 opacity-50" />
                                        <p>
                                            {isLive
                                                ? "Waiting for events..."
                                                : "Click Live to start streaming events"}
                                        </p>
                                    </div>
                                ) : (
                                    <div className="space-y-2 max-h-[500px] overflow-auto">
                                        {filteredEvents.map((event, i) => (
                                            <div
                                                key={i}
                                                className="p-3 rounded-lg border bg-[hsl(var(--muted))]/50"
                                            >
                                                <div className="flex items-center justify-between mb-2">
                                                    <div className="flex items-center gap-2">
                                                        <span
                                                            className={`px-2 py-0.5 rounded text-xs font-medium ${getEventColor(
                                                                event.type
                                                            )}`}
                                                        >
                                                            {event.type}
                                                        </span>
                                                        <span className="text-sm font-medium">
                                                            {event.collection}
                                                        </span>
                                                    </div>
                                                    <span className="text-xs text-[hsl(var(--muted-foreground))]">
                                                        {formatRelativeTime(event.timestamp)}
                                                    </span>
                                                </div>
                                                <pre className="text-xs text-[hsl(var(--muted-foreground))] overflow-x-auto">
                                                    {JSON.stringify(event.new || event.old, null, 2)}
                                                </pre>
                                            </div>
                                        ))}
                                    </div>
                                )}
                            </CardContent>
                        </Card>
                    </div>
                </div>
            </div>
        </div>
    );
}
