import { useAuth } from "@/lib/auth/AuthContext";
import { Button } from "@/components/ui/button";
import { Moon, Sun, LogOut, RefreshCw, User } from "lucide-react";
import { useState, useEffect } from "react";
import { formatRelativeTime } from "@/lib/utils";

interface HeaderProps {
    title: string;
    subtitle?: string;
    onRefresh?: () => void;
    isRefreshing?: boolean;
    lastFetched?: string;
}

export function Header({
    title,
    subtitle,
    onRefresh,
    isRefreshing,
    lastFetched,
}: HeaderProps) {
    const { user, signOut } = useAuth();
    const [isDark, setIsDark] = useState(true);

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
            {/* Title section */}
            <div className="flex flex-col">
                <h1 className="text-xl font-semibold">{title}</h1>
                {subtitle && (
                    <p className="text-sm text-[hsl(var(--muted-foreground))]">
                        {subtitle}
                    </p>
                )}
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

                {/* User menu */}
                {user && (
                    <div className="flex items-center gap-2 ml-2 pl-2 border-l border-[hsl(var(--border))]">
                        <div className="flex items-center gap-2">
                            <div className="h-8 w-8 rounded-full bg-[hsl(var(--primary))] flex items-center justify-center">
                                <User className="h-4 w-4 text-[hsl(var(--primary-foreground))]" />
                            </div>
                            <span className="text-sm font-medium hidden sm:inline">
                                {user.email}
                            </span>
                        </div>
                        <Button
                            variant="ghost"
                            size="icon"
                            onClick={signOut}
                            aria-label="Sign out"
                        >
                            <LogOut className="h-5 w-5" />
                        </Button>
                    </div>
                )}
            </div>
        </header>
    );
}
