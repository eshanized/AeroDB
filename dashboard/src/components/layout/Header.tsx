import { useAuth } from "@/lib/auth/AuthContext";
import { Button } from "@/components/ui/button";
import {
    Moon,
    Sun,
    LogOut,
    RefreshCw,
    User,
    Search,
    Keyboard,
    Settings,
    CreditCard,
} from "lucide-react";
import { useState, useEffect } from "react";
import { formatRelativeTime } from "@/lib/utils";
import { useLocation, Link } from "react-router-dom";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
    Avatar,
    AvatarFallback,
    Breadcrumb,
    BreadcrumbItem,
    BreadcrumbLink,
    BreadcrumbList,
    BreadcrumbPage,
    BreadcrumbSeparator,
} from "@/components/ui";

interface HeaderProps {
    title: string;
    subtitle?: string;
    onRefresh?: () => void;
    isRefreshing?: boolean;
    lastFetched?: string;
}

export function Header({
    title: _title,
    subtitle: _subtitle,
    onRefresh,
    isRefreshing,
    lastFetched,
}: HeaderProps) {
    const { user, signOut } = useAuth();
    const [isDark, setIsDark] = useState(true);
    const location = useLocation();

    // Generate breadcrumbs from path
    const pathSegments = location.pathname.split("/").filter((p) => p);

    // Initialize theme from localStorage or system preference
    useEffect(() => {
        const savedTheme = localStorage.getItem("aerodb_theme");
        if (savedTheme) {
            setIsDark(savedTheme === "dark");
        } else {
            setIsDark(window.matchMedia("(prefers-color-scheme: dark)").matches);
        }
    }, []);

    // Apply theme to document
    useEffect(() => {
        if (isDark) {
            document.documentElement.classList.add("dark");
        } else {
            document.documentElement.classList.remove("dark");
        }
        localStorage.setItem("aerodb_theme", isDark ? "dark" : "light");
    }, [isDark]);

    const toggleTheme = () => setIsDark(!isDark);

    return (
        <header className="sticky top-0 z-30 flex h-16 items-center justify-between border-b border-[hsl(var(--border))] bg-[hsl(var(--background))]/95 backdrop-blur supports-[backdrop-filter]:bg-[hsl(var(--background))]/60 px-6">
            {/* Breadcrumbs */}
            <div className="flex flex-col gap-1">
                <Breadcrumb>
                    <BreadcrumbList>
                        <BreadcrumbItem>
                            <BreadcrumbLink asChild>
                                <Link to="/">Home</Link>
                            </BreadcrumbLink>
                        </BreadcrumbItem>
                        {pathSegments.map((segment, index) => {
                            const isLast = index === pathSegments.length - 1;
                            const path = `/${pathSegments.slice(0, index + 1).join("/")}`;
                            return (
                                <div key={path} className="flex items-center">
                                    <BreadcrumbSeparator />
                                    <BreadcrumbItem>
                                        {isLast ? (
                                            <BreadcrumbPage className="capitalize">
                                                {segment}
                                            </BreadcrumbPage>
                                        ) : (
                                            <BreadcrumbLink asChild>
                                                <Link to={path} className="capitalize">
                                                    {segment}
                                                </Link>
                                            </BreadcrumbLink>
                                        )}
                                    </BreadcrumbItem>
                                </div>
                            );
                        })}
                    </BreadcrumbList>
                </Breadcrumb>
                {/* Only show title if it differs from current breadcrumb or adds context */}
                {/* We can rely on breadcrumbs for nav context now, but title is still useful for page specific actions */}
            </div>

            {/* Actions */}
            <div className="flex items-center gap-2">
                {/* Last fetched indicator */}
                {lastFetched && (
                    <span className="text-xs text-[hsl(var(--muted-foreground))] mr-2">
                        Updated {formatRelativeTime(lastFetched)}
                    </span>
                )}

                {/* Refresh button */}
                {onRefresh && (
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={onRefresh}
                        disabled={isRefreshing}
                        className="gap-2"
                    >
                        <RefreshCw
                            className={`h-4 w-4 ${isRefreshing ? "animate-spin" : ""}`}
                        />
                        Refresh
                    </Button>
                )}

                {/* Theme toggle */}
                <Button
                    variant="ghost"
                    size="icon"
                    onClick={toggleTheme}
                    aria-label="Toggle theme"
                >
                    {isDark ? <Sun className="h-5 w-5" /> : <Moon className="h-5 w-5" />}
                </Button>

                {/* Command Palette Trigger */}
                <Button
                    variant="outline"
                    size="sm"
                    className="hidden lg:flex w-full max-w-[200px] justify-between text-muted-foreground mr-2 bg-muted/50"
                    onClick={() => {
                        const event = new KeyboardEvent("keydown", {
                            key: "k",
                            metaKey: true,
                            ctrlKey: true,
                        });
                        document.dispatchEvent(event);
                    }}
                >
                    <span className="flex items-center gap-2">
                        <Search className="h-4 w-4" />
                        Search...
                    </span>
                    <span className="flex items-center gap-1 text-xs border bg-background px-1.5 py-0.5 rounded">
                        <Keyboard className="h-3 w-3" />K
                    </span>
                </Button>

                {/* User menu */}
                {user && (
                    <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                            <Button
                                variant="ghost"
                                className="relative h-8 w-8 rounded-full ml-2"
                            >
                                <Avatar className="h-8 w-8">
                                    <AvatarFallback className="bg-primary text-primary-foreground">
                                        {user.email?.substring(0, 2).toUpperCase()}
                                    </AvatarFallback>
                                </Avatar>
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent className="w-56" align="end" forceMount>
                            <DropdownMenuLabel className="font-normal">
                                <div className="flex flex-col space-y-1">
                                    <p className="text-sm font-medium leading-none">
                                        User
                                    </p>
                                    <p className="text-xs leading-none text-muted-foreground">
                                        {user.email}
                                    </p>
                                </div>
                            </DropdownMenuLabel>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem>
                                <User className="mr-2 h-4 w-4" />
                                <span>Profile</span>
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                <CreditCard className="mr-2 h-4 w-4" />
                                <span>Billing</span>
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                <Settings className="mr-2 h-4 w-4" />
                                <span>Settings</span>
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem onClick={signOut}>
                                <LogOut className="mr-2 h-4 w-4" />
                                <span>Log out</span>
                            </DropdownMenuItem>
                        </DropdownMenuContent>
                    </DropdownMenu>
                )}
            </div>
        </header>
    );
}
