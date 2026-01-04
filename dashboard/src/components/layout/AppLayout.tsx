import { useState, useEffect } from "react";
import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { CommandPalette } from "./CommandPalette";
import { cn } from "@/lib/utils";

export function AppLayout() {
    const [isCollapsed, setIsCollapsed] = useState(false);
    const [isMobile, setIsMobile] = useState(false);

    // Handle responsive behavior
    useEffect(() => {
        const checkMobile = () => {
            const mobile = window.innerWidth < 768;
            setIsMobile(mobile);
            if (mobile) {
                setIsCollapsed(true);
            }
        };

        checkMobile();
        window.addEventListener("resize", checkMobile);
        return () => window.removeEventListener("resize", checkMobile);
    }, []);

    // Load collapsed state from localStorage
    useEffect(() => {
        if (!isMobile) {
            const saved = localStorage.getItem("aerodb_sidebar_collapsed");
            if (saved !== null) {
                setIsCollapsed(saved === "true");
            }
        }
    }, [isMobile]);

    const handleToggle = () => {
        setIsCollapsed((prev) => {
            const next = !prev;
            if (!isMobile) {
                localStorage.setItem("aerodb_sidebar_collapsed", String(next));
            }
            return next;
        });
    };

    return (
        <div className="min-h-screen bg-[hsl(var(--background))]">
            <Sidebar isCollapsed={isCollapsed} onToggle={handleToggle} />

            {/* Main content */}
            <main
                className={cn(
                    "min-h-screen transition-all duration-300",
                    isCollapsed ? "ml-16" : "ml-60"
                )}
            >
                <Outlet />
            </main>
            <CommandPalette />
        </div>
    );
}
