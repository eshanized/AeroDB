import * as React from "react";
import {
    User,
    Database,
    HardDrive,
    Activity,
    Server,
    LogOut,
    Sun,
    Moon,
} from "lucide-react";

import {
    CommandDialog,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
    CommandList,
    CommandSeparator,
    CommandShortcut,
} from "@/components/ui/command";
import { useNavigate } from "react-router-dom";
import { useAuth } from "@/lib/auth/AuthContext";

export function CommandPalette() {
    const [open, setOpen] = React.useState(false);
    const navigate = useNavigate();
    const { signOut } = useAuth();

    React.useEffect(() => {
        const down = (e: KeyboardEvent) => {
            if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
                e.preventDefault();
                setOpen((open) => !open);
            }
        };

        document.addEventListener("keydown", down);
        return () => document.removeEventListener("keydown", down);
    }, []);

    const runCommand = React.useCallback((command: () => unknown) => {
        setOpen(false);
        command();
    }, []);

    const setTheme = (theme: "dark" | "light") => {
        localStorage.setItem("aerodb_theme", theme);
        if (theme === "dark") {
            document.documentElement.classList.add("dark");
        } else {
            document.documentElement.classList.remove("dark");
        }
    };

    return (
        <CommandDialog open={open} onOpenChange={setOpen}>
            <CommandInput placeholder="Type a command or search..." />
            <CommandList>
                <CommandEmpty>No results found.</CommandEmpty>
                <CommandGroup heading="Navigation">
                    <CommandItem onSelect={() => runCommand(() => navigate("/database"))}>
                        <Database className="mr-2 h-4 w-4" />
                        <span>Database</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => navigate("/auth"))}>
                        <User className="mr-2 h-4 w-4" />
                        <span>Authentication</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => navigate("/storage"))}>
                        <HardDrive className="mr-2 h-4 w-4" />
                        <span>Storage</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => navigate("/realtime"))}>
                        <Activity className="mr-2 h-4 w-4" />
                        <span>Realtime</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => navigate("/cluster"))}>
                        <Server className="mr-2 h-4 w-4" />
                        <span>Cluster</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => navigate("/observability"))}>
                        <Activity className="mr-2 h-4 w-4" />
                        <span>Observability</span>
                    </CommandItem>
                </CommandGroup>
                <CommandSeparator />
                <CommandGroup heading="Settings">
                    <CommandItem onSelect={() => runCommand(() => navigate("/auth"))}>
                        <User className="mr-2 h-4 w-4" />
                        <span>Profile</span>
                        <CommandShortcut>⌘P</CommandShortcut>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => setTheme("light"))}>
                        <Sun className="mr-2 h-4 w-4" />
                        <span>Light Mode</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => setTheme("dark"))}>
                        <Moon className="mr-2 h-4 w-4" />
                        <span>Dark Mode</span>
                    </CommandItem>
                    <CommandItem onSelect={() => runCommand(() => signOut())}>
                        <LogOut className="mr-2 h-4 w-4" />
                        <span>Log out</span>
                    </CommandItem>
                </CommandGroup>
            </CommandList>
        </CommandDialog>
    );
}
