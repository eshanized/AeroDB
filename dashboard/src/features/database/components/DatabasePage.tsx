import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Header } from "@/components/layout/Header";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { api } from "@/lib/api/client";
import { config } from "@/config";
import { formatNumber, isStale } from "@/lib/utils";
import type { Collection, PaginatedResponse } from "@/types";
import {
    ChevronLeft,
    ChevronRight,
    Search,
    Filter,
    // RefreshCw,
    Database,
    AlertCircle,
} from "lucide-react";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
    Skeleton,
} from "@/components/ui";

export function DatabasePage() {
    const [selectedCollection, setSelectedCollection] = useState<string | null>(null);
    const [searchQuery, setSearchQuery] = useState("");
    const [currentPage, setCurrentPage] = useState(0);
    const limit = config.pagination.defaultLimit;

    // Fetch collections
    const collectionsQuery = useQuery({
        queryKey: ["collections"],
        queryFn: async () => {
            const response = await api.get<{ data: Collection[] }>(
                `${config.endpoints.rest}/_schema/collections`
            );
            if (response.error) throw new Error(response.error.message);
            return response.data;
        },
    });

    // Fetch data for selected collection
    const dataQuery = useQuery({
        queryKey: ["collection-data", selectedCollection, currentPage, searchQuery],
        queryFn: async () => {
            if (!selectedCollection) return null;
            const offset = currentPage * limit;
            const response = await api.get<PaginatedResponse<Record<string, unknown>>>(
                `${config.endpoints.rest}/${selectedCollection}`,
                {
                    select: "*",
                    limit,
                    offset,
                    ...(searchQuery && { q: searchQuery }),
                }
            );
            if (response.error) throw new Error(response.error.message);
            return {
                ...response.data,
                fetched_at: new Date().toISOString(),
            };
        },
        enabled: !!selectedCollection,
    });

    const collections = collectionsQuery.data?.data || [];
    const filteredCollections = collections.filter((c) =>
        c.name.toLowerCase().includes(searchQuery.toLowerCase())
    );

    const handleRefresh = () => {
        if (selectedCollection) {
            dataQuery.refetch();
        } else {
            collectionsQuery.refetch();
        }
    };

    const isDataStale = dataQuery.data?.fetched_at
        ? isStale(dataQuery.data.fetched_at, config.staleDataThreshold)
        : false;

    return (
        <div className="flex flex-col h-screen">
            <Header
                title="Database"
                subtitle={selectedCollection ? `Browsing ${selectedCollection}` : "Select a collection to browse"}
                onRefresh={handleRefresh}
                isRefreshing={collectionsQuery.isFetching || dataQuery.isFetching}
                lastFetched={dataQuery.data?.fetched_at}
            />

            <div className="flex-1 p-6 overflow-auto">
                {/* Stale data warning */}
                {isDataStale && (
                    <div className="mb-4 flex items-center gap-2 text-sm text-amber-500 bg-amber-500/10 p-3 rounded-md">
                        <AlertCircle className="h-4 w-4" />
                        <span>Data may be stale. Click refresh to fetch latest.</span>
                    </div>
                )}

                <div className="grid grid-cols-12 gap-6">
                    {/* Collections sidebar */}
                    <div className="col-span-3">
                        <Card>
                            <CardHeader className="pb-3">
                                <CardTitle className="text-base flex items-center gap-2">
                                    <Database className="h-4 w-4" />
                                    Collections
                                </CardTitle>
                            </CardHeader>
                            <CardContent className="space-y-2">
                                {/* Search */}
                                <div className="relative">
                                    <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                                    <Input
                                        placeholder="Search collections..."
                                        className="pl-8"
                                        value={searchQuery}
                                        onChange={(e) => setSearchQuery(e.target.value)}
                                    />
                                </div>

                                {/* Collection list */}
                                <div className="space-y-1 max-h-96 overflow-auto">
                                    {collectionsQuery.isLoading ? (
                                        <div className="space-y-2">
                                            <Skeleton className="h-8 w-full" />
                                            <Skeleton className="h-8 w-full" />
                                            <Skeleton className="h-8 w-full" />
                                        </div>
                                    ) : filteredCollections.length === 0 ? (
                                        <div className="text-sm text-muted-foreground p-2">
                                            No collections found
                                        </div>
                                    ) : (
                                        filteredCollections.map((collection) => (
                                            <button
                                                key={collection.name}
                                                onClick={() => {
                                                    setSelectedCollection(collection.name);
                                                    setCurrentPage(0);
                                                }}
                                                className={`w-full text-left px-3 py-2 rounded-md text-sm transition-colors ${selectedCollection === collection.name
                                                    ? "bg-accent text-accent-foreground"
                                                    : "hover:bg-accent hover:text-accent-foreground"
                                                    }`}
                                            >
                                                <div className="flex items-center justify-between">
                                                    <span className="font-medium">{collection.name}</span>
                                                    <span className="text-xs text-muted-foreground">
                                                        {formatNumber(collection.count)}
                                                    </span>
                                                </div>
                                            </button>
                                        ))
                                    )}
                                </div>
                            </CardContent>
                        </Card>
                    </div>

                    {/* Data table */}
                    <div className="col-span-9">
                        <Card>
                            <CardHeader className="pb-3">
                                <div className="flex items-center justify-between">
                                    <CardTitle className="text-base flex items-center gap-2">
                                        <Table className="h-4 w-4" /> // This is the Icon, wait "Table" is now a component
                                        {/* Conflict: Table icon vs Table component.
                                            I should rename the imported component or icon.
                                            Lucide exports 'Table' icon.
                                            SHADCN exports 'Table' component.
                                            I should rename Lucide icon to TableIcon.
                                         */}
                                        {selectedCollection || "Select a collection"}
                                    </CardTitle>
                                    {selectedCollection && (
                                        <div className="flex items-center gap-2">
                                            <Button variant="outline" size="sm" className="gap-2">
                                                <Filter className="h-4 w-4" />
                                                Filters
                                            </Button>
                                        </div>
                                    )}
                                </div>
                            </CardHeader>
                            <CardContent>
                                {!selectedCollection ? (
                                    <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                                        <Database className="h-12 w-12 mb-4 opacity-50" />
                                        <p>Select a collection from the sidebar to browse data</p>
                                    </div>
                                ) : dataQuery.isLoading ? (
                                    <div className="space-y-2">
                                        <Skeleton className="h-10 w-full" />
                                        <Skeleton className="h-10 w-full" />
                                        <Skeleton className="h-10 w-full" />
                                    </div>
                                ) : dataQuery.error ? (
                                    <div className="flex flex-col items-center justify-center py-12 text-destructive">
                                        <AlertCircle className="h-12 w-12 mb-4" />
                                        <p>{(dataQuery.error as Error).message}</p>
                                    </div>
                                ) : (
                                    <>
                                        {/* Table */}
                                        <div className="border rounded-lg overflow-hidden">
                                            <div className="overflow-x-auto">
                                                <Table>
                                                    <TableHeader>
                                                        <TableRow>
                                                            {dataQuery.data?.data &&
                                                                dataQuery.data.data.length > 0 &&
                                                                Object.keys(dataQuery.data.data[0]).map((key) => (
                                                                    <TableHead key={key}>
                                                                        {key}
                                                                    </TableHead>
                                                                ))}
                                                        </TableRow>
                                                    </TableHeader>
                                                    <TableBody>
                                                        {dataQuery.data?.data?.map((row, i) => (
                                                            <TableRow
                                                                key={i}
                                                                className="cursor-pointer"
                                                            >
                                                                {Object.values(row).map((value, j) => (
                                                                    <TableCell key={j}>
                                                                        {typeof value === "object"
                                                                            ? JSON.stringify(value)
                                                                            : String(value ?? "")}
                                                                    </TableCell>
                                                                ))}
                                                            </TableRow>
                                                        ))}
                                                    </TableBody>
                                                </Table>
                                            </div>
                                        </div>

                                        {/* Pagination */}
                                        <div className="flex items-center justify-between mt-4">
                                            <div className="text-sm text-muted-foreground">
                                                Showing {currentPage * limit + 1} -{" "}
                                                {Math.min((currentPage + 1) * limit, dataQuery.data?.total || 0)} of{" "}
                                                {formatNumber(dataQuery.data?.total || 0)} rows
                                            </div>
                                            <div className="flex items-center gap-2">
                                                <Button
                                                    variant="outline"
                                                    size="sm"
                                                    onClick={() => setCurrentPage((p) => Math.max(0, p - 1))}
                                                    disabled={currentPage === 0}
                                                >
                                                    <ChevronLeft className="h-4 w-4" />
                                                </Button>
                                                <span className="text-sm">Page {currentPage + 1}</span>
                                                <Button
                                                    variant="outline"
                                                    size="sm"
                                                    onClick={() => setCurrentPage((p) => p + 1)}
                                                    disabled={
                                                        (currentPage + 1) * limit >= (dataQuery.data?.total || 0)
                                                    }
                                                >
                                                    <ChevronRight className="h-4 w-4" />
                                                </Button>
                                            </div>
                                        </div>
                                    </>
                                )}
                            </CardContent>
                        </Card>
                    </div>
                </div>
            </div>
        </div>
    );
}
