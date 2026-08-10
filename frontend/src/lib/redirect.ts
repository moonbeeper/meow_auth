import type { AuthUser } from "./auth/auth.svelte";

export const getRedirectUrl = (user: AuthUser | null, url: URL): string | null => {
    const path = url.pathname;
    const isLoggedIn = user != null;

    const unauthedPaths = path.startsWith("/login") || path.startsWith("/signup");
    const iDontCarePaths = path == "/" || path.startsWith("/auth");

    if (isLoggedIn && unauthedPaths) {
        return "/me";
    }

    if (!isLoggedIn && !unauthedPaths && !iDontCarePaths) {
        return `/login?redirect=${encodeURIComponent(path)}`;
    }

    return null;
};
