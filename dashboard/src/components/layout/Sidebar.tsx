import { NavLink } from "react-router-dom";
import { cn } from "@/lib/utils";
import {
    Database,
    Users,
    FolderOpen,
    Activity,
    Server,
    BarChart3,
    ChevronLeft,
    Menu,
} from "lucide-react";
import { Button } from "@/components/ui/button";

interface SidebarProps {
    isCollapsed: boolean;
    onToggle: () => void;
}

const navItems = [
    {
        title: "Database",
        href: "/database",
        icon: Database,
        description: "Browse tables, run queries",
    },
    {
        title: "Authentication",
        href: "/auth",
        icon: Users,
        description: "Manage users and sessions",
    },
    {
        title: "Storage",
        href: "/storage",
        icon: FolderOpen,
        description: "File buckets and uploads",
    },
    {
        title: "Real-time",
        href: "/realtime",
        icon: Activity,
        description: "Active subscriptions",
    },
    {
        title: "Cluster",
        href: "/cluster",
        icon: Server,
        description: "Topology and replication",
    },
    {
        title: "Observability",
        href: "/observability",
        icon: BarChart3,
        description: "Logs and metrics",
    },
];

export function Sidebar({ isCollapsed, onToggle }: SidebarProps) {
    return (
        <aside
            className={cn(
                "fixed left-0 top-0 z-40 h-screen border-r border-[hsl(var(--border))] bg-[hsl(var(--card))] transition-all duration-300",
                isCollapsed ? "w-16" : "w-60"
            )}
        >
            {/* Logo and toggle */}
            <div className="flex h-16 items-center justify-between border-b border-[hsl(var(--border))] px-4">
                {!isCollapsed && (
                    <div className="flex items-center gap-2">
                        <div className="h-8 w-8 rounded-lg bg-[hsl(var(--primary))] flex items-center justify-center">
                            <Database className="h-4 w-4 text-[hsl(var(--primary-foreground))]" />
                        </div>
                        <span className="font-semibold text-lg">AeroDB</span>
                    </div>
                )}
                <Button
                    variant="ghost"
                    size="icon"
                    onClick={onToggle}
                    className="shrink-0"
                    aria-label={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
                >
                    {isCollapsed ? (
                        <Menu className="h-5 w-5" />
                    ) : (
                        <ChevronLeft className="h-5 w-5" />
                    )}
                </Button>
            </div>

            {/* Navigation */}
            <nav className="flex flex-col gap-1 p-2">
                {navItems.map((item) => (
                    <NavLink
                        key={item.href}
                        to={item.href}
                        className={({ isActive }) =>
                            cn(
                                "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors",
                                "hover:bg-[hsl(var(--accent))] hover:text-[hsl(var(--accent-foreground))]",
                                isActive
                                    ? "bg-[hsl(var(--accent))] text-[hsl(var(--accent-foreground))]"
                                    : "text-[hsl(var(--muted-foreground))]",
                                isCollapsed && "justify-center px-2"
                            )
                        }
                        title={isCollapsed ? item.title : undefined}
                    >
                        <item.icon className="h-5 w-5 shrink-0" />
                        {!isCollapsed && (
                            <div className="flex flex-col">
                                <span>{item.title}</span>
                                <span className="text-xs text-[hsl(var(--muted-foreground))] font-normal">
                                    {item.description}
                                </span>
                            </div>
                        )}
                    </NavLink>
                ))}
            </nav>

            {/* Footer */}
            {!isCollapsed && (
                <div className="absolute bottom-4 left-4 right-4">
                    <div className="rounded-lg bg-[hsl(var(--muted))] p-3 text-xs text-[hsl(var(--muted-foreground))]">
                        <div className="font-medium mb-1">AeroDB Dashboard</div>
                        <div>Observability without authority</div>
                    </div>
                </div>
            )}
        </aside>
    );
}
