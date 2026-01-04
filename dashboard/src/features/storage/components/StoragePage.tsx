import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Header } from "@/components/layout/Header";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/api/client";
import { config } from "@/config";
import { formatBytes, formatRelativeTime } from "@/lib/utils";
import type { Bucket, StorageObject } from "@/types";
import {
    FolderOpen,
    Upload,
    Download,
    Trash2,
    ChevronRight,
    File,
    Image,
    FileText,
    Archive,
    Plus,
    RefreshCw,
    AlertCircle,
} from "lucide-react";

function FileIcon({ contentType }: { contentType: string }) {
    if (contentType.startsWith("image/")) return <Image className="h-5 w-5 text-blue-500" />;
    if (contentType.includes("pdf")) return <FileText className="h-5 w-5 text-red-500" />;
    if (contentType.includes("zip") || contentType.includes("archive"))
        return <Archive className="h-5 w-5 text-amber-500" />;
    return <File className="h-5 w-5 text-[hsl(var(--muted-foreground))]" />;
}

export function StoragePage() {
    const [selectedBucket, setSelectedBucket] = useState<string | null>(null);
    const [currentPath, setCurrentPath] = useState<string[]>([]);

    // Fetch buckets
    const bucketsQuery = useQuery({
        queryKey: ["buckets"],
        queryFn: async () => {
            const response = await api.get<{ data: Bucket[] }>(`${config.endpoints.storage}/bucket`);
            if (response.error) throw new Error(response.error.message);
            return response.data?.data || [];
        },
    });

    // Fetch files in selected bucket
    const filesQuery = useQuery({
        queryKey: ["files", selectedBucket, currentPath.join("/")],
        queryFn: async () => {
            if (!selectedBucket) return [];
            const prefix = currentPath.join("/");
            const response = await api.get<{ data: StorageObject[] }>(
                `${config.endpoints.storage}/object/${selectedBucket}`,
                prefix ? { prefix } : undefined
            );
            if (response.error) throw new Error(response.error.message);
            return response.data?.data || [];
        },
        enabled: !!selectedBucket,
    });

    const handleBucketSelect = (bucketName: string) => {
        setSelectedBucket(bucketName);
        setCurrentPath([]);
    };

    // Navigation is handled inline in the breadcrumb component

    return (
        <div className="flex flex-col h-screen">
            <Header
                title="Storage"
                subtitle={selectedBucket ? `Bucket: ${selectedBucket}` : "File storage management"}
                onRefresh={() => {
                    bucketsQuery.refetch();
                    if (selectedBucket) filesQuery.refetch();
                }}
                isRefreshing={bucketsQuery.isFetching || filesQuery.isFetching}
            />

            <div className="flex-1 p-6 overflow-auto">
                <div className="grid grid-cols-12 gap-6">
                    {/* Buckets sidebar */}
                    <div className="col-span-3">
                        <Card>
                            <CardHeader className="pb-3">
                                <div className="flex items-center justify-between">
                                    <CardTitle className="text-base flex items-center gap-2">
                                        <FolderOpen className="h-4 w-4" />
                                        Buckets
                                    </CardTitle>
                                    <Button variant="ghost" size="icon" className="h-8 w-8">
                                        <Plus className="h-4 w-4" />
                                    </Button>
                                </div>
                            </CardHeader>
                            <CardContent>
                                {bucketsQuery.isLoading ? (
                                    <div className="flex items-center justify-center py-8">
                                        <RefreshCw className="h-5 w-5 animate-spin text-[hsl(var(--muted-foreground))]" />
                                    </div>
                                ) : bucketsQuery.data?.length === 0 ? (
                                    <p className="text-sm text-[hsl(var(--muted-foreground))] text-center py-4">
                                        No buckets yet
                                    </p>
                                ) : (
                                    <div className="space-y-1">
                                        {bucketsQuery.data?.map((bucket) => (
                                            <button
                                                key={bucket.id}
                                                onClick={() => handleBucketSelect(bucket.name)}
                                                className={`w-full text-left px-3 py-2 rounded-md text-sm transition-colors ${selectedBucket === bucket.name
                                                    ? "bg-[hsl(var(--accent))] text-[hsl(var(--accent-foreground))]"
                                                    : "hover:bg-[hsl(var(--accent))]"
                                                    }`}
                                            >
                                                <div className="flex items-center justify-between">
                                                    <div className="flex items-center gap-2">
                                                        <FolderOpen className="h-4 w-4" />
                                                        <span className="font-medium">{bucket.name}</span>
                                                    </div>
                                                    <span className="text-xs text-[hsl(var(--muted-foreground))]">
                                                        {bucket.public ? "Public" : "Private"}
                                                    </span>
                                                </div>
                                                <div className="flex items-center justify-between mt-1 text-xs text-[hsl(var(--muted-foreground))]">
                                                    <span>{bucket.file_count} files</span>
                                                    <span>{formatBytes(bucket.total_size)}</span>
                                                </div>
                                            </button>
                                        ))}
                                    </div>
                                )}
                            </CardContent>
                        </Card>
                    </div>

                    {/* File browser */}
                    <div className="col-span-9">
                        <Card>
                            <CardHeader className="pb-3">
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center gap-2">
                                        {/* Breadcrumb */}
                                        <button
                                            onClick={() => {
                                                setSelectedBucket(null);
                                                setCurrentPath([]);
                                            }}
                                            className="text-sm text-[hsl(var(--muted-foreground))] hover:text-[hsl(var(--foreground))]"
                                        >
                                            Storage
                                        </button>
                                        {selectedBucket && (
                                            <>
                                                <ChevronRight className="h-4 w-4 text-[hsl(var(--muted-foreground))]" />
                                                <button
                                                    onClick={() => setCurrentPath([])}
                                                    className="text-sm text-[hsl(var(--muted-foreground))] hover:text-[hsl(var(--foreground))]"
                                                >
                                                    {selectedBucket}
                                                </button>
                                            </>
                                        )}
                                        {currentPath.map((folder, i) => (
                                            <span key={i} className="flex items-center gap-2">
                                                <ChevronRight className="h-4 w-4 text-[hsl(var(--muted-foreground))]" />
                                                <button
                                                    onClick={() => setCurrentPath(currentPath.slice(0, i + 1))}
                                                    className="text-sm text-[hsl(var(--muted-foreground))] hover:text-[hsl(var(--foreground))]"
                                                >
                                                    {folder}
                                                </button>
                                            </span>
                                        ))}
                                    </div>
                                    {selectedBucket && (
                                        <Button size="sm" className="gap-2">
                                            <Upload className="h-4 w-4" />
                                            Upload
                                        </Button>
                                    )}
                                </div>
                            </CardHeader>
                            <CardContent>
                                {!selectedBucket ? (
                                    <div className="flex flex-col items-center justify-center py-12 text-[hsl(var(--muted-foreground))]">
                                        <FolderOpen className="h-12 w-12 mb-4 opacity-50" />
                                        <p>Select a bucket from the sidebar to browse files</p>
                                    </div>
                                ) : filesQuery.isLoading ? (
                                    <div className="flex items-center justify-center py-12">
                                        <RefreshCw className="h-6 w-6 animate-spin text-[hsl(var(--muted-foreground))]" />
                                    </div>
                                ) : filesQuery.error ? (
                                    <div className="flex flex-col items-center justify-center py-12 text-[hsl(var(--destructive))]">
                                        <AlertCircle className="h-12 w-12 mb-4" />
                                        <p>{(filesQuery.error as Error).message}</p>
                                    </div>
                                ) : filesQuery.data?.length === 0 ? (
                                    <div className="flex flex-col items-center justify-center py-12 text-[hsl(var(--muted-foreground))]">
                                        <FolderOpen className="h-12 w-12 mb-4 opacity-50" />
                                        <p>This folder is empty</p>
                                        <Button variant="outline" className="mt-4 gap-2">
                                            <Upload className="h-4 w-4" />
                                            Upload files
                                        </Button>
                                    </div>
                                ) : (
                                    <div className="border rounded-lg overflow-hidden">
                                        <table className="w-full text-sm">
                                            <thead className="border-b bg-[hsl(var(--muted))]">
                                                <tr>
                                                    <th className="text-left px-4 py-3 font-medium text-[hsl(var(--muted-foreground))]">
                                                        Name
                                                    </th>
                                                    <th className="text-left px-4 py-3 font-medium text-[hsl(var(--muted-foreground))]">
                                                        Size
                                                    </th>
                                                    <th className="text-left px-4 py-3 font-medium text-[hsl(var(--muted-foreground))]">
                                                        Type
                                                    </th>
                                                    <th className="text-left px-4 py-3 font-medium text-[hsl(var(--muted-foreground))]">
                                                        Uploaded
                                                    </th>
                                                    <th className="text-right px-4 py-3 font-medium text-[hsl(var(--muted-foreground))]">
                                                        Actions
                                                    </th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {filesQuery.data?.map((file) => (
                                                    <tr key={file.id} className="border-b hover:bg-[hsl(var(--muted))]/50">
                                                        <td className="px-4 py-3">
                                                            <div className="flex items-center gap-2">
                                                                <FileIcon contentType={file.content_type} />
                                                                <span className="font-medium">{file.path.split("/").pop()}</span>
                                                            </div>
                                                        </td>
                                                        <td className="px-4 py-3 text-[hsl(var(--muted-foreground))]">
                                                            {formatBytes(file.size)}
                                                        </td>
                                                        <td className="px-4 py-3 text-[hsl(var(--muted-foreground))]">
                                                            {file.content_type}
                                                        </td>
                                                        <td className="px-4 py-3 text-[hsl(var(--muted-foreground))]">
                                                            {formatRelativeTime(file.created_at)}
                                                        </td>
                                                        <td className="px-4 py-3 text-right">
                                                            <div className="flex items-center justify-end gap-1">
                                                                <Button variant="ghost" size="icon" className="h-8 w-8">
                                                                    <Download className="h-4 w-4" />
                                                                </Button>
                                                                <Button
                                                                    variant="ghost"
                                                                    size="icon"
                                                                    className="h-8 w-8 text-[hsl(var(--destructive))]"
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
                </div>
            </div>
        </div>
    );
}
